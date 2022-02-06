use crate::api::error;
use crate::constants::ROAD_ID;
use crate::models::*;
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
    .data(web::JsonConfig::default().limit(1024 * 1024));
}

// TODO: Get player id from session

async fn get_base_details(pool: Data<Pool>) -> Result<impl Responder> {
    let response = web::block(move || {
        let conn = pool.get()?;
        let map = util::fetch_map_layout(&conn, 2)?;
        util::get_details_from_map_layout(&conn, map)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(response))
}

async fn set_base_details(
    map_spaces: Json<Vec<NewMapSpaces>>,
    pool: Data<Pool>,
) -> Result<impl Responder> {
    let map_spaces = map_spaces.into_inner();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks) = web::block(move || {
        Ok((
            util::fetch_map_layout(&conn, 2)?,
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
) -> Result<impl Responder> {
    let map_spaces = map_spaces.into_inner();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks, mut level_constraints) = web::block(move || {
        let map = util::fetch_map_layout(&conn, 2)?;
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
