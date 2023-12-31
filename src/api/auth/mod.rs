use self::pragyan::PragyanMessage;
use super::{PgPool, RedisPool};
use crate::api::error;
use actix_session::Session;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::web::{self, Data, Json, Query};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use oauth2::reqwest::http_client;
use oauth2::{basic::BasicClient, TokenResponse};
use oauth2::{
    AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, RedirectUrl, Scope, TokenUrl,
};
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};
// use std::env;
mod pragyan;
pub mod session;
mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/gauth2/login").route(web::get().to(google_login)))
        .service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)))
        .service(web::resource("/gauth2/callback").route(web::get().to(login_callback)));
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
    pub authorize_url: String,
    pub csrf_state: String,
}

fn client() -> BasicClient {
    let google_client_id = ClientId::new(
        "684397563262-1mp4uefnhlb6kbpobl5rdbnd336avkps.apps.googleusercontent.com".to_string(),
    );
    let google_client_secret = ClientSecret::new("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".to_string());
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.

    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8000/user/gauth2/callback".to_string())
            .expect("Invalid redirect URL"),
    )
}

async fn google_login() -> impl Responder {
    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, csrf_token) = client()
        .authorize_url(CsrfToken::new_random)
        .add_scope(Scope::new("email".to_string()))
        .add_scope(Scope::new("openid".to_string()))
        .add_scope(Scope::new("profile".to_string()))
        .url();

    return Json(GoogleLoginResponse {
        authorize_url: authorize_url.to_string(),
        csrf_state: csrf_token.secret().clone().to_string(),
    });
}

async fn login_callback(params: Query<QueryCode>) -> impl Responder {
    let code = AuthorizationCode::new(params.code.clone());
    let token_result = client().exchange_code(code.clone()).request(http_client);
    Json(CallbackResponse {
        autherization_code: code.secret().clone().to_string(),
        access_token: token_result
            .unwrap()
            .access_token()
            .secret()
            .clone()
            .to_string(),
    })
}

#[derive(Serialize)]
pub struct CallbackResponse {
    pub access_token: String,
    pub autherization_code: String,
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
