use super::auth::session::AuthUser;
use super::PgPool;
use crate::api::error;
use crate::models::*;
use actix_web::error::{ErrorBadRequest, ErrorForbidden, ErrorNotFound};
use actix_web::web::{self, Data, Json};
use actix_web::{Responder, Result};
use serde::Deserialize;
use std::collections::HashMap;

mod util;
mod validate;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::put().to(set_base_details))
            .route(web::get().to(get_user_base_details)),
    )
    .service(web::resource("/top").route(web::get().to(get_top_defenses)))
    .service(web::resource("/save").route(web::put().to(confirm_base_details)))
    .service(web::resource("/game/{id}").route(web::get().to(get_game_base_details)))
    .service(web::resource("/{defender_id}").route(web::get().to(get_other_base_details)))
    .service(web::resource("/{defender_id}/history").route(web::get().to(defense_history)))
    .app_data(Data::new(web::JsonConfig::default().limit(1024 * 1024)));
}

#[derive(Deserialize)]
pub struct MapSpacesEntry {
    pub blk_type: i32,
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub rotation: i32,
    pub building_type: i32,
}

async fn get_user_base_details(pool: Data<PgPool>, user: AuthUser) -> Result<impl Responder> {
    let defender_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        let map = util::fetch_map_layout(&mut conn, &defender_id)?;
        util::get_details_from_map_layout(&mut conn, map)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(response))
}

async fn get_other_base_details(
    defender_id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let defender_id = defender_id.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let defender_exists = web::block(move || util::defender_exists(defender_id, &mut conn))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if !defender_exists {
        return Err(ErrorNotFound("Player not found"));
    }

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let map = web::block(move || util::fetch_map_layout(&mut conn, &defender_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

    if !map.is_valid {
        return Err(ErrorBadRequest("Invalid Base"));
    }

    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::get_details_from_map_layout(&mut conn, map)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(response))
}

async fn get_game_base_details(
    game_id: web::Path<i32>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let game_id = game_id.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let map = web::block(move || util::fetch_map_layout_from_game(&mut conn, game_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

    if map.is_none() {
        return Err(ErrorNotFound("Game not found"));
    }

    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::get_details_from_map_layout(&mut conn, map.unwrap())
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(Json(response))
}

async fn set_base_details(
    map_spaces: Json<Vec<MapSpacesEntry>>,
    pool: Data<PgPool>,
    user: AuthUser,
) -> Result<impl Responder> {
    let defender_id = user.0;

    if !util::is_defense_allowed_now() {
        return Err(ErrorForbidden("Cannot edit base now"));
    }

    let map_spaces = map_spaces.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks) = web::block(move || {
        Ok((
            util::fetch_map_layout(&mut conn, &defender_id)?,
            util::fetch_blocks(&mut conn)?,
        )) as anyhow::Result<(MapLayout, Vec<BlockType>)>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    validate::is_valid_update_layout(&map_spaces, &blocks)?;

    web::block(move || {
        let mut conn = pool.get()?;
        util::set_map_invalid(&mut conn, map.id)?;
        util::put_base_details(&map_spaces, &map, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok("Updated successfully")
}

async fn confirm_base_details(
    map_spaces: Json<Vec<MapSpacesEntry>>,
    pool: Data<PgPool>,
    user: AuthUser,
) -> Result<impl Responder> {
    let defender_id = user.0;

    if !util::is_defense_allowed_now() {
        return Err(ErrorForbidden("Cannot edit base now"));
    }

    let map_spaces = map_spaces.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks, mut level_constraints) = web::block(move || {
        let map = util::fetch_map_layout(&mut conn, &defender_id)?;
        Ok((
            map.clone(),
            util::fetch_blocks(&mut conn)?,
            util::get_level_constraints(&mut conn, map.level_id)?,
        )) as anyhow::Result<(MapLayout, Vec<BlockType>, HashMap<i32, i32>)>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    validate::is_valid_save_layout(&map_spaces, &mut level_constraints, &blocks)?;

    web::block(move || {
        let mut conn = pool.get()?;
        util::put_base_details(&map_spaces, &map, &mut conn)?;
        util::set_map_valid(&mut conn, map.id)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok("Saved successfully")
}

async fn defense_history(
    defender_id: web::Path<i32>,
    pool: web::Data<PgPool>,
    user: AuthUser,
) -> Result<impl Responder> {
    let user_id = user.0;
    let defender_id = defender_id.into_inner();
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_defense_history(defender_id, user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn get_top_defenses(pool: web::Data<PgPool>, user: AuthUser) -> Result<impl Responder> {
    let user_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_top_defenses(user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

