use crate::models::*;
use actix_web::error::{ErrorBadRequest, ErrorInternalServerError};
use actix_web::web::{self, Data, Json};
use actix_web::{Responder, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;

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
    let conn = &*pool
        .get()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let map = util::fetch_map_layout(conn, 2)?;
    let response = util::get_details_from_map_layout(map, conn)?;

    Ok(Json(response))
}

async fn set_base_details(
    map_spaces: Json<Vec<NewMapSpaces>>,
    pool: Data<Pool>,
) -> Result<impl Responder> {
    let conn = &*pool
        .get()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let map_spaces = map_spaces.into_inner();
    let map = util::fetch_map_layout(conn, 2)?;
    let blocks = util::fetch_blocks(conn)?;

    if validate::is_valid_update_layout(&map_spaces, &map, &blocks) {
        util::put_base_details(&map_spaces, &map, conn)?;
        Ok("Updated successfully")
    } else {
        Err(ErrorBadRequest("Invalid Map Layout"))
    }
}

async fn confirm_base_details(
    map_spaces: Json<Vec<NewMapSpaces>>,
    pool: Data<Pool>,
) -> Result<impl Responder> {
    let conn = &*pool
        .get()
        .map_err(|_| ErrorInternalServerError("Internal Server Error"))?;
    let map_spaces = map_spaces.into_inner();
    let blocks = util::fetch_blocks(conn)?;
    let map = util::fetch_map_layout(conn, 2)?;
    let road_id = util::get_road_id(conn)?;
    let mut level_constraints = util::get_level_constraints(conn, map.level_id)?;

    if validate::is_valid_update_layout(&map_spaces, &map, &blocks)
        && validate::is_valid_save_layout(&map_spaces, road_id, &mut level_constraints, &blocks)
    {
        util::put_base_details(&map_spaces, &map, conn)?;
        util::set_map_valid(conn, map.id)?;
        Ok("Saved successfully")
    } else {
        Err(ErrorBadRequest("Invalid Map Layout"))
    }
}
