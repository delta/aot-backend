use self::pragyan::PragyanMessage;
use super::{PgPool, RedisPool};
use crate::api::error;
use actix_session::Session;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{self, Data, Json};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use oauth2::basic::BasicClient;
use pwhash::bcrypt;
use reqwest::header::LOCATION;
use serde::{Deserialize, Serialize};

use oauth2::{
    AuthUrl, ClientId, ClientSecret, CsrfToken, PkceCodeChallenge, RedirectUrl, RevocationUrl,
    Scope, TokenUrl,
};
use std::env;
mod pragyan;
pub mod session;
mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)))
        .service(web::resource("/gauth2/login").route(web::post().to(google_login)));
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

async fn google_login() -> impl Responder {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_CLIENT_ID").expect("Missing the GOOGLE_CLIENT_ID environment variable."),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_CLIENT_SECRET")
            .expect("Missing the GOOGLE_CLIENT_SECRET environment variable."),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the config for the Google OAuth2 process.
    let client = BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    // This example will be running its own server at localhost:8080.
    // See below for the server implementation.
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8080".to_string()).expect("Invalid redirect URL"),
    )
    // Google supports OAuth 2.0 Token Revocation (RFC-7009)
    .set_revocation_uri(
        RevocationUrl::new("https://oauth2.googleapis.com/revoke".to_string())
            .expect("Invalid revocation endpoint URL"),
    );

    // Google supports Proof Key for Code Exchange (PKCE - https://oauth.net/2/pkce/).
    // Create a PKCE code verifier and SHA-256 encode it as a code challenge.
    let (pkce_code_challenge, _pkce_code_verifier) = PkceCodeChallenge::new_random_sha256();

    // Generate the authorization URL to which we'll redirect the user.
    let (authorize_url, _csrf_state) = client
        .authorize_url(CsrfToken::new_random)
        // This example is requesting access to the "calendar" features and the user's profile.
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/calendar".to_string(),
        ))
        .add_scope(Scope::new(
            "https://www.googleapis.com/auth/plus.me".to_string(),
        ))
        .set_pkce_challenge(pkce_code_challenge)
        .url();

    let mut response = HttpResponse::Found();
    response.append_header((LOCATION, authorize_url.to_string()));
    response.body("");
    response.finish()
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
