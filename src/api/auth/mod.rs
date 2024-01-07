use self::authentication_token::AuthenticationToken;
use self::pragyan::PragyanMessage;
use super::{PgPool, RedisPool};
use crate::api::error;
use crate::constants::OTP_LIMIT;
use actix_session::Session;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized};
use actix_web::web::{self, Data, Json, Query};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use oauth2::reqwest::http_client;
use oauth2::TokenResponse;
use oauth2::{AuthorizationCode, CsrfToken, Scope};
use pwhash::bcrypt;
use reqwest::header::{AUTHORIZATION, COOKIE, LOCATION};
// use reqwest::header::LOCATION;
use serde::{Deserialize, Serialize};
use std::env;
pub mod authentication_token;
mod pragyan;
pub mod session;
mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/gauth2/login").route(web::get().to(google_login)))
        .service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)))
        .service(web::resource("/gauth2/callback").route(web::get().to(login_callback)))
        .service(web::resource("/autherization/health-check").route(web::get().to(health_check)));
    }

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: i32,
    pub username: String,
    pub name: String,
    pub overall_rating: i32,
    pub avatar: i32,
    pub highest_rating: i32,
    pub email: String,
}

#[derive(Debug, Deserialize)]
pub struct QueryCode {
    pub state: String,
    pub code: String,
}

#[derive(Debug, Serialize)]
pub struct GoogleLoginResponse {
    pub csrf_state: String,
}

#[derive(Serialize, Deserialize)]
pub struct UserInfoFromGoogle {
    name: String,
    email: String,
    picture: String,
}

#[derive(Serialize)]
pub struct CallbackResponse {
    expiry_time: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub id: i32,
    pub iat: usize,
    pub exp: usize,
}

async fn health_check(user: AuthenticationToken, pg_pool: Data<PgPool>) -> Result<impl Responder> {
    let user_id = user.id;

    let mut pg_conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    let user = web::block(move || util::get_user_by_user_id(&mut pg_conn, &user_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(user))
}
async fn google_login() -> impl Responder {
    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_token) = util::client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    //TODO: Store the CSRF token somewhere so we can verify it in the callback.

    // Redirect the user to the authorization URL sent in the below json response.
    HttpResponse::Found()
        .append_header((LOCATION, authorize_url.to_string()))
        .append_header(("GOOGLE_CSRF_TOKEN", csrf_token.secret().to_string()))
        .finish()
}

async fn login_callback(
    params: Query<QueryCode>,
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    //extracting the authorization code from the query parameters in the callback url
    let code = AuthorizationCode::new(params.code.clone());

    //TODO: Verify the CSRF state returned by Google matches the one we generated before proceeding.
    let state = params.state.clone();
    if state.is_empty() {
        return Err(ErrorBadRequest("Invalid state"));
    }

    //exchanging the authorization code for the access token
    let token_result = util::client().exchange_code(code).request(http_client);
    let access_token = match token_result {
        Ok(token_result) => token_result.access_token().secret().clone(),
        Err(e) => return Err(ErrorInternalServerError(e.to_string())),
    };
    let url = "https://www.googleapis.com/oauth2/v3/userinfo"; //url for getting user info from google

    //exchanging the access token for the user info
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await;
    //TODO: use map instead of nested match error handling
    let userinfo: UserInfoFromGoogle = match response {
        Ok(response) => match response.json().await {
            Ok(json_response_from_google) => json_response_from_google,
            Err(e) => return Err(ErrorInternalServerError(e.to_string())),
        },
        Err(_) => {
            return Err(ErrorInternalServerError(
                "Error in getting user info from google",
            ))
        }
    };
    let email = userinfo.email;
    let name = userinfo.name;
    //checking if the user exists in db else creating a new user
    let user = web::block(move || {
        let mut conn = pg_pool.get()?;
        let mut redis_conn = redis_pool.get()?;
        util::get_oauth_user(&mut conn, &mut redis_conn, &email, &name)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    //generating jwt token and cookie
    let (token, cookie, expiring_time) = util::generate_jwt_token_and_cookie(user.id).unwrap();

    let frontend_origin = env::var("FRONTEND_URL").expect("Frontend origin must be set!");

    //the user will be redirected to the frontend_origin with the cookie and jwt in the header.
    Ok(HttpResponse::Found()
        .append_header((LOCATION, frontend_origin))
        .append_header((COOKIE, cookie.to_string()))
        .append_header((AUTHORIZATION, token))
        .append_header(("expiry_time", expiring_time.to_string()))
        .finish())
}

async fn login(
    request: web::Json<LoginRequest>,
    session: Session,
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let username = request.username.clone();
    let mut pg_conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    let user = web::block(move || util::get_user_by_username(&mut pg_conn, &username))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(user) = user {
        if !user.is_pragyan {
            if bcrypt::verify(&request.password, &user.password) {
                session::set(&session, user.id, user.is_verified).map_err(error::handle_error)?;
                if user.is_verified {
                    return Ok(Json(LoginResponse {
                        user_id: user.id,
                        username: user.username,
                        name: user.name,
                        overall_rating: user.overall_rating,
                        avatar: user.avatar,
                        highest_rating: user.highest_rating,
                        email: user.email,
                    }));
                }
                // Account not verified
                return Err(ErrorUnauthorized("App account not verified"));
            } else {
                return Err(ErrorUnauthorized("Invalid Credentials"));
            }
        }
    } else {
        return Err(ErrorUnauthorized("Invalid Credentials"));
    }

    let LoginRequest { username, password } = request.into_inner();
    // Pragyan users need to login with email
    let email = username.to_lowercase();
    let pragyan_auth = pragyan::auth(email, password)
        .await
        .map_err(error::handle_error)?;
    match pragyan_auth.status_code {
        200 => {
            if let PragyanMessage::Success(pragyan_user) = pragyan_auth.message {
                let name = pragyan_user.user_fullname.clone();
                let user = web::block(move || {
                    let mut conn = pg_pool.get()?;
                    let mut redis_conn = redis_pool.get()?;
                    let email = username.clone();
                    util::get_pragyan_user(&mut conn, &mut redis_conn, &email, &name)
                })
                .await?
                .map_err(|err| error::handle_error(err.into()))?;
                session::set(&session, user.id, true).map_err(error::handle_error)?;
                Ok(Json(LoginResponse {
                    user_id: user.id,
                    username: user.username,
                    name: pragyan_user.user_fullname,
                    overall_rating: user.overall_rating,
                    avatar: user.avatar,
                    highest_rating: user.highest_rating,
                    email: user.email,
                }))
            } else {
                Err(anyhow::anyhow!(
                    "Unexpected error in Pragyan auth: {:?}",
                    pragyan_auth
                ))
                .map_err(|err| error::handle_error(err.into()))?
            }
        }
        203 => Err(ErrorUnauthorized("Pragyan account not verified")),
        _ => Err(ErrorUnauthorized(
            "Invalid username/Pragyan email or password",
        )),
    }
}

async fn logout(session: Session) -> impl Responder {
    session.clear();
    HttpResponse::NoContent().finish()
}
