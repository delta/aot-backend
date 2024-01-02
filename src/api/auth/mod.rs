use self::pragyan::PragyanMessage;
use super::{PgPool, RedisPool};
use crate::api::error;
use actix_session::Session;
use actix_web::error::ErrorUnauthorized;
use actix_web::web::{self, Data, Json};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};

mod pragyan;
pub mod session;
mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)));
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
    pub avatar_id: i32,
    pub attacks_won: i32,
    pub defenses_won: i32,
    pub trophies: i32,
    pub artifacts: i32,
    pub email: String,
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
                        avatar_id: user.avatar_id,
                        attacks_won: user.attacks_won,
                        defenses_won: user.defenses_won,
                        trophies: user.trophies,
                        artifacts: user.artifacts,
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
