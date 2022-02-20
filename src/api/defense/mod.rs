use super::auth::session;
use crate::api::error;
use crate::constants::ROAD_ID;
use crate::models::*;
use actix_session::Session;
use actix_web::error::ErrorBadRequest;
use actix_web::web::{self, Data, Json};
use actix_web::{Responder, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use std::collections::HashMap;

mod util;
mod validate;

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::put().to(set_base_details))
            .route(web::get().to(get_base_details)),
    )
    .service(web::resource("/save").route(web::put().to(confirm_base_details)))
    .service(web::resource("/{defender_id}/history").route(web::get().to(defense_history)))
    .service(web::resource("/top").route(web::get().to(get_top_defenses)))
    .data(web::JsonConfig::default().limit(1024 * 1024));
}

async fn get_base_details(pool: Data<Pool>, session: Session) -> Result<impl Responder> {
    let defender_id = session::get_current_user(&session)?;
    let response = web::block(move || {
        let conn = pool.get()?;
        let map = util::fetch_map_layout(&conn, &defender_id)?;
        util::get_details_from_map_layout(&conn, map)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(response))
}

async fn set_base_details(
    map_spaces: Json<Vec<NewMapSpaces>>,
    pool: Data<Pool>,
    session: Session,
) -> Result<impl Responder> {
    let defender_id = session::get_current_user(&session)?;
    let map_spaces = map_spaces.into_inner();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks) = web::block(move || {
        Ok((
            util::fetch_map_layout(&conn, &defender_id)?,
            util::fetch_blocks(&conn)?,
        )) as anyhow::Result<(MapLayout, Vec<BlockType>)>
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    if validate::is_valid_update_layout(&map_spaces, &map, &blocks) {
        web::block(move || {
            let conn = pool.get()?;
            util::set_map_invalid(&conn, map.id)?;
            util::put_base_details(&map_spaces, &map, &conn)
        })
        .await
        .map_err(|err| error::handle_error(err.into()))?;
        Ok("Updated successfully")
    } else {
        Err(ErrorBadRequest("Invalid map layout"))
    }
}

async fn confirm_base_details(
    map_spaces: Json<Vec<NewMapSpaces>>,
    pool: Data<Pool>,
    session: Session,
) -> Result<impl Responder> {
    let defender_id = session::get_current_user(&session)?;
    let map_spaces = map_spaces.into_inner();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks, mut level_constraints) = web::block(move || {
        let map = util::fetch_map_layout(&conn, &defender_id)?;
        Ok((
            map.clone(),
            util::fetch_blocks(&conn)?,
            util::get_level_constraints(&conn, map.level_id)?,
        )) as anyhow::Result<(MapLayout, Vec<BlockType>, HashMap<i32, i32>)>
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    if validate::is_valid_update_layout(&map_spaces, &map, &blocks)
        && validate::is_valid_save_layout(&map_spaces, ROAD_ID, &mut level_constraints, &blocks)
    {
        web::block(move || {
            let conn = pool.get()?;
            util::put_base_details(&map_spaces, &map, &conn)?;
            util::set_map_valid(&conn, map.id)
        })
        .await
        .map_err(|err| error::handle_error(err.into()))?;
        Ok("Saved successfully")
    } else {
        Err(ErrorBadRequest("Invalid map layout"))
    }
}

async fn defense_history(
    defender_id: web::Path<i32>,
    pool: web::Data<Pool>,
    session: Session,
) -> Result<impl Responder> {
    let user_id = session::get_current_user(&session)?;
    let defender_id = defender_id.0;
    let response = web::block(move || {
        let conn = pool.get()?;
        util::fetch_defense_history(defender_id, user_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn get_top_defenses(pool: web::Data<Pool>, session: Session) -> Result<impl Responder> {
    let user_id = session::get_current_user(&session)?;
    let response = web::block(move || {
        let conn = pool.get()?;
        util::fetch_top_defenses(user_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}
