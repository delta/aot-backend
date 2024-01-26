use super::{PgPool, RedisPool};
use crate::api::error;
use actix_session::Session;
use actix_web::error::ErrorInternalServerError;
use actix_web::web::{self, Data, Json};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use oauth2::reqwest::http_client;
use oauth2::AuthorizationCode;
use oauth2::TokenResponse;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::env;
pub mod authentication_token;
pub mod session;
mod util;

use self::authentication_token::AuthenticationToken;
use self::pragyan::PragyanMessage;
use actix_web::error::ErrorUnauthorized;

mod pragyan;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)))
        .service(web::resource("/oauth2-login").route(web::post().to(oauth2_login)))
        .service(web::resource("/get-user").route(web::get().to(get_user)));
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

#[derive(Deserialize)]
struct OauthLoginRequest {
    code: String,
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

#[derive(Deserialize, Serialize)]
struct LogoutRequest {
    user_id: i32,
}

async fn logout(
    user: AuthenticationToken,
    session: Session,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let user_id = user.id;
    // get redis connection from redis pool
    let mut redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    // delete user id from redis db
    redis_conn
        .del(user_id)
        .map_err(|err| error::handle_error(err.into()))?;

    // clear the session cookie
    session.clear();
    Ok(HttpResponse::NoContent().finish())
}

async fn get_user(user: AuthenticationToken, pool: Data<PgPool>) -> Result<impl Responder> {
    let mut pool_conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user_id = user.id;
    let user = web::block(move || util::fetch_user_from_db(&mut pool_conn, user_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

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
}
async fn oauth2_login(
    session: Session,
    request: web::Json<OauthLoginRequest>,
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    //getting the auth code from request body
    let code = AuthorizationCode::new(request.code.clone());

    //exchanging the authorization code for the access token
    let token_result = util::client().exchange_code(code).request(http_client);
    let access_token = token_result
        .map_err(|err| error::handle_error(err.into()))?
        .access_token()
        .secret()
        .clone();

    let url =
        env::var("GOOGLE_OAUTH_USER_INFO_URL").expect("GOOGLE_OAUTH_USER_INFO_URL must be set"); //url for getting user info from google

    //exchanging the access token for the user info
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("Authorization", format!("Bearer {access_token}"))
        .send()
        .await;

    let userinfo: UserInfoFromGoogle = response
        .map_err(|err| error::handle_error(err.into()))?
        .json()
        .await
        .map_err(|err| error::handle_error(err.into()))?;

    let email = userinfo.email;
    let name = userinfo.name;

    //checking if the user exists in db else creating a new user
    let user = web::block(move || {
        let mut conn = pg_pool.get()?;
        util::get_oauth_user(&mut conn, &email, &name)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    //get redis connection from redis pool
    let mut redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    // check if user is logged in on any other device by checking redis db
    let is_loggedin: Result<i32, redis::RedisError> = redis_conn.exists(user.id);
    match is_loggedin {
        Ok(i) => {
            if i == 1 {
                return Err(ErrorUnauthorized(
                    "User already logged in on another device",
                ));
            } else {
                log::info!("User not logged in on any other device");
            }
        }
        Err(_) => {
            return Err(ErrorInternalServerError(
                "Error in checking user id in redis",
            ))
        }
    };
    // if not add ther user id to redis db
    redis_conn
        .set(user.id, 0)
        .map_err(|err| error::handle_error(err.into()))?;

    //generating jwt token
    let (token, expiring_time) = util::generate_jwt_token(user.id).unwrap();

    // insert the jwt token in the session cookie
    session
        .insert("token", token.clone())
        .expect("Failed to insert token in session");

    Ok(HttpResponse::Found()
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
