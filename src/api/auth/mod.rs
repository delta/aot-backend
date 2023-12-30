use super::{PgPool, RedisPool};
use crate::api::error;
use actix_session::Session;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::{self, Data, Json, Query};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use oauth2::reqwest::http_client;
use oauth2::{AuthorizationCode, CsrfToken, Scope};
use oauth2::{PkceCodeChallenge, PkceCodeVerifier, TokenResponse};
use reqwest::header::{AUTHORIZATION, LOCATION};
use serde::{Deserialize, Serialize};
use std::env;
pub mod authentication_token;
pub mod session;
mod util;

use self::pragyan::PragyanMessage;
use actix_web::error::ErrorUnauthorized;

mod pragyan;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::get().to(logout)))
        .service(web::resource("/oauth2-login").route(web::get().to(oauth2_login)))
        .service(web::resource("/login/callback").route(web::get().to(login_callback)));
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub user_id: i32,
    pub username: String,
    pub name: String,
    pub avatar_id: i32,
    pub attacks_won: i32,
    pub defenses_won: i32,
    pub trophies: i32,
    pub artifacts: i32,
    pub email: String,
}
#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
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
            return Ok(Json(LoginResponse {
                user_id: user.id,
                username: user.username,
                name: user.name,
                avatar_id: user.avatar_id,
                attacks_won: user.attacks_won,
                defenses_won: user.defenses_won,
                trophies: user.trophies,
                artifacts: user.artifacts,
                email: user.email,
            }));
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
                    name: user.name,
                    avatar_id: user.avatar_id,
                    attacks_won: user.attacks_won,
                    defenses_won: user.defenses_won,
                    trophies: user.trophies,
                    artifacts: user.artifacts,
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

async fn oauth2_login(session: Session) -> impl Responder {
    //generate pkce code verifier and challenge
    let (pkce_code_challenge, pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Store the PKCE code verifier in the user's session.
    session
        .insert("pkce_code_verifier", pkce_code_verifier)
        .expect("Failed to insert PKCE code verifier in session");

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_token) = util::client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    // Store the CSRF token in the user's session.
    session
        .insert("csrf_token", csrf_token.clone())
        .expect("Failed to insert CSRF token in session");

    // Redirect the user to the authorization URL sent in the below json response.
    HttpResponse::Found()
        .append_header((LOCATION, authorize_url.to_string()))
        .append_header(("GOOGLE_CSRF_TOKEN", csrf_token.secret().to_string()))
        .finish()
}

async fn login_callback(
    session: Session,
    params: Query<QueryCode>,
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    //extracting the authorization code from the query parameters in the callback url
    let code = AuthorizationCode::new(params.code.clone());

    //extracting the csrf token from the query parameters in the callback url
    let state = params.state.clone();
    if state.is_empty() {
        return Err(ErrorBadRequest("Invalid state"));
    }

    //extracting the csrf token from the session
    let state_from_session = session
        .get::<CsrfToken>("csrf_token")
        .map_err(|_| ErrorInternalServerError("Error in getting csrf token from session"))?
        .ok_or(ErrorInternalServerError(
            "Error in getting csrf token from session",
        ))?;

    //check if both the csrf token in the query parameters and the csrf token in the session are same
    if state != *state_from_session.secret() {
        return Err(ErrorBadRequest("Invalid state"));
    }

    let pkce_verifier = session
        .get::<PkceCodeVerifier>("pkce_code_verifier")
        .map_err(|_| ErrorInternalServerError("Error in getting pkce code verifier from session"))?
        .ok_or(ErrorInternalServerError(
            "Error in getting pkce code verifier from session",
        ))?;

    //exchanging the authorization code for the access token
    let token_result = util::client()
        .exchange_code(code)
        .set_pkce_verifier(pkce_verifier)
        .request(http_client);
    let access_token = match token_result {
        Ok(token_result) => token_result.access_token().secret().clone(),
        Err(e) => return Err(ErrorInternalServerError(e.to_string())),
    };
    let url =
        env::var("GOOGLE_OAUTH_USER_INFO_URL").expect("GOOGLE_OAUTH_USER_INFO_URL must be set"); //url for getting user info from google

    //exchanging the access token for the user info
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await;

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
    let (token, expiring_time) = util::generate_jwt_token(user.id).unwrap();

    let frontend_origin = env::var("FRONTEND_URL").expect("Frontend origin must be set!");

    session
        .insert("token", token.clone())
        .expect("Failed to insert token in session");

    //the user will be redirected to the frontend_origin with the cookie and jwt in the header.
    Ok(HttpResponse::Found()
        .append_header((LOCATION, frontend_origin + "/auth"))
        .append_header((AUTHORIZATION, token))
        .append_header(("expiry_time", expiring_time))
        .json(Json(LoginResponse {
            user_id: user.id,
            username: user.username,
            name: user.name,
            avatar_id: user.avatar_id,
            attacks_won: user.attacks_won,
            defenses_won: user.defenses_won,
            trophies: user.trophies,
            artifacts: user.artifacts,
            email: user.email,
        })))
}
