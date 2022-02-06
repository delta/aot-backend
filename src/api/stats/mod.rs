use crate::api::error;
use actix_web::error::ErrorNotFound;
use actix_web::web::{self, Data, Json, Path};
use actix_web::{Responder, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

mod util;

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn get_user_stats(user_id: Path<i32>, pool: Data<Pool>) -> Result<impl Responder> {
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
