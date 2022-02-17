use super::auth::session;
use crate::api::error;
use crate::models::UpdateUser;
use actix_session::Session;
use actix_web::error::{ErrorConflict, ErrorNotFound};
use actix_web::web::{self, Data, Json, Path};
use actix_web::{Responder, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use serde::Deserialize;

mod util;

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::patch().to(update_user)))
        .service(web::resource("/register").route(web::post().to(register)))
        .service(web::resource("/{id}/stats").route(web::get().to(get_user_stats)));
}

#[derive(Clone, Deserialize)]
pub struct InputUser {
    name: String,
    phone: String,
    username: String,
    password: String,
}

async fn register(pool: Data<Pool>, input_user: Json<InputUser>) -> Result<impl Responder> {
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = input_user.clone();
    let duplicates = web::block(move || util::get_duplicate_users(&conn, &user))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    for duplicate in duplicates {
        if duplicate.phone == input_user.phone && duplicate.is_verified {
            return Err(ErrorConflict("Phone number already exists"));
        } else if duplicate.username == input_user.username {
            return Err(ErrorConflict("Username already exists"));
        }
    }
    web::block(move || {
        let conn = pool.get()?;
        util::add_user(&conn, &input_user)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok("Successfully Registered")
}

async fn update_user(
    user: Json<UpdateUser>,
    pool: Data<Pool>,
    session: Session,
) -> Result<impl Responder> {
    let user_id = session::get_current_user(&session)?;
    let username = user.username.clone();
    if let Some(username) = username {
        let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
        let duplicate = web::block(move || util::get_duplicate_username(&conn, &username))
            .await
            .map_err(|err| error::handle_error(err.into()))?;
        if duplicate.is_some() {
            return Err(ErrorConflict("Username already exists"));
        }
    }
    web::block(move || {
        let conn = pool.get()?;
        util::update_user(&conn, user_id, &user)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    Ok("User updated successfully")
}

async fn get_user_stats(user_id: Path<i32>, pool: Data<Pool>) -> Result<impl Responder> {
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::fetch_user(&conn, user_id.0))
        .await
        .map_err(|_| ErrorNotFound("USER NOT FOUND"))?;
    let response = web::block(move || {
        let conn = pool.get()?;
        let attack_game = util::fetch_attack_game(&conn, user_id.0)?;
        let defense_game = util::fetch_defense_game(&conn, user_id.0)?;
        let users = util::fetch_all_user(&conn)?;
        util::make_response(&user, &attack_game, &defense_game, &users)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(Json(response))
}
