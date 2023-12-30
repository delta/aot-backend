use super::auth::session::AuthUser;
use super::{PgPool, RedisPool};
use crate::api::error;
use crate::models::UpdateUser;
use actix_web::error::{ErrorBadRequest, ErrorConflict, ErrorNotFound};
use actix_web::web::{self, Data, Json, Path};
use actix_web::{Responder, Result};
use serde::{Deserialize,Serialize};

pub mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/profile/{id}").route(web::patch().to(update_user)))
        .service(web::resource("/register").route(web::post().to(register)))
        .service(web::resource("/{id}/stats").route(web::get().to(get_user_stats)))
        .service(web::resource("/profile/{id}").route(web::get().to(get_user_profile)));
}

#[derive(Clone, Deserialize)]
pub struct InputUser {
    name: String,
    phone: String,
    username: String,
    password: String,
}
#[derive(Serialize)]
struct UserProfileResponse {
    user_id: i32,
    name: String,
    trophies: i32,
    artifacts: i32,
    defenses_won: i32,
    avatar_id: i32,
}
#[derive(Debug, Serialize)]
struct ErrorResponse {
    message: String,
}
#[derive(Debug, Serialize)]
struct SuccessResponse {
    message: String,
}


async fn register(
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    input_user: Json<InputUser>,
) -> Result<impl Responder> {
    let mut conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let user = input_user.clone();
    if user.username.len() < 6 {
        return Err(ErrorBadRequest(
            "Username should contain atleast 6 characters",
        ));
    }
    if user.password.len() < 6 {
        return Err(ErrorBadRequest(
            "Password should contain atleast 6 characters",
        ));
    }
    let duplicates = web::block(move || util::get_duplicate_users(&mut conn, &user))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    for duplicate in duplicates {
        if duplicate.phone == input_user.phone && duplicate.is_verified {
            return Err(ErrorConflict("Phone number already exists"));
        } else if duplicate.username == input_user.username {
            return Err(ErrorConflict("Username already exists"));
        }
    }
    web::block(move || {
        let mut conn = pg_pool.get()?;
        let redis_conn = redis_pool.get()?;
        util::add_user(&mut conn, redis_conn, &input_user)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok("Successfully Registered")
}

async fn update_user(
    player_id: Path<i32>,
    user_details: Json<UpdateUser>,
    pool: Data<PgPool>,
) -> Result<impl Responder, Error> {
    let player_id = player_id.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::fetch_user(&mut conn, player_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

    if let Some(user) = user {
        web::block(move || {
            let mut conn = pool.get()?;
            util::update_user(&mut conn, player_id, &user_details)
        })
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

        let success_response = SuccessResponse {
            message: "Update profile success".to_string(),
        };
        Ok(Json(success_response))
    } else {
        let error_response = ErrorResponse {
            message: "Player not found".to_string(),
        };
        Ok(ErrorNotFound(Json(error_response)))
    }
}

async fn get_user_stats(user_id: Path<i32>, pool: Data<PgPool>) -> Result<impl Responder> {
    let user_id = user_id.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::fetch_user(&mut conn, user_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(user) = user {
        let response = web::block(move || {
            let mut conn = pool.get()?;
            let attack_game = util::fetch_attack_game(&mut conn, user_id)?;
            let defense_game = util::fetch_defense_game(&mut conn, user_id)?;
            let users = util::fetch_all_user(&mut conn)?;
            util::make_response(&user, &attack_game, &defense_game, &users)
        })
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
        Ok(Json(response))
    } else {
        Err(ErrorNotFound("User not found"))
    }
}
async fn get_user_profile(
    user_id: Path<i32>,
    pool: Data<PgPool>,
) -> Result<impl Responder, Error> {
    let user_id = user_id.into_inner();
    let mut conn = pool.get().map_err(error::handle_error)?;

    let user = web::block(move || util::fetch_user(&mut conn, user_id))
        .await?
        .map_err(error::handle_error)?;

    if let Some(user) = user {
        let response = UserProfileResponse {
            user_id: user.id,
            name: user.name,
            trophies: user.trophies,
            artifacts: user.artifacts,
            attacks_won: user.attacks_won,
            defenses_won: user.defenses_won,
            avatar_id: user.avatar_id,
        };

        Ok(Json(response))
    } else {
        let error_response = ErrorResponse {
            message: "Player not found".to_string(),
        };
        Ok(ErrorNotFound(Json(error_response)))
    }
}





