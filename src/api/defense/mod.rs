use super::auth::session::AuthUser;
use super::user::util::fetch_user;
use super::PgPool;
use crate::api::error;
use crate::api::util::HistoryboardQuery;
use crate::models::*;
use actix_web::error::{ErrorBadRequest, ErrorNotFound};
use actix_web::web::{self, Data, Json};
use actix_web::{Responder, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// use super::RedisPool;   //Uncomment to check for user under attack//

mod shortest_path;
mod util;
mod validate;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::put().to(set_base_details))
            .route(web::get().to(get_user_base_details)),
    )
    .service(web::resource("/top").route(web::get().to(get_top_defenses)))
    .service(web::resource("/transfer").route(web::post().to(post_transfer_artifacts)))
    .service(web::resource("/save").route(web::put().to(confirm_base_details)))
    .service(web::resource("/game/{id}").route(web::get().to(get_game_base_details)))
    .service(web::resource("/history").route(web::get().to(defense_history)))
    .service(web::resource("/{defender_id}").route(web::get().to(get_other_base_details)))
    .app_data(Data::new(web::JsonConfig::default().limit(1024 * 1024)));
}

#[derive(Deserialize)]
pub struct MapSpacesEntry {
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub block_type_id: i32,
}

#[derive(Deserialize)]
pub struct TransferArtifactEntry {
    pub artifacts_differ: i32,
    pub map_space_id: i32,
}

#[derive(Serialize)]
pub struct TransferArtifactResponse {
    pub building_map_space_id: i32,
    pub artifacts_in_building: i32,
    pub bank_map_space_id: i32,
    pub artifacts_in_bank: i32,
}

async fn post_transfer_artifacts(
    transfer: Json<TransferArtifactEntry>,
    pg_pool: Data<PgPool>,
    // redis_pool: Data<RedisPool>,   //Uncomment to check for user under attack//
    user: AuthUser,
) -> Result<impl Responder> {
    let user_id = user.0;
    let bank_block_type_id = 5; //To be altered as per the bank block type id
    let transfer = transfer.into_inner();

    // let is_defender = match util::check_user_under_attack(&redis_pool, &user_id) {       //Uncomment to check for user under attack//
    //     Ok(result) => result,
    //     Err(e) => {
    //         eprintln!("Error checking if user is under attack: {}", e);
    //         false // or handle the error in another way
    //     }
    // };

    // if !is_defender {   //Uncomment to check for user under attack//

    let mut conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let existing_artifacts_in_building = web::block(move || {
        util::fetch_existing_artifacts_in_building(
            &mut conn,
            &user_id,
            &transfer.map_space_id,
            &bank_block_type_id,
        )
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let artifacts_in_bank = web::block(move || {
        util::fetch_existing_artifacts_in_bank(
            &mut conn,
            &user_id,
            &existing_artifacts_in_building.bank_map_space_id,
        )
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let block_capacity = web::block(move || {
        util::fetch_block_capacity(&mut conn, &existing_artifacts_in_building.block_type_id)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let bank_map_space_id = existing_artifacts_in_building.bank_map_space_id;
    let new_building_artifact_count =
        existing_artifacts_in_building.artifact_count + transfer.artifacts_differ;
    let new_bank_artifact_count = artifacts_in_bank.artifact_count - transfer.artifacts_differ;

    if bank_map_space_id == transfer.map_space_id {
        return Err(ErrorBadRequest("Cannot transfer to the same building"));
    } else if transfer.artifacts_differ > artifacts_in_bank.artifact_count {
        return Err(ErrorBadRequest("Not enough artifacts in the bank"));
    } else if existing_artifacts_in_building.block_type_id != 0 {
        if block_capacity
            < transfer.artifacts_differ + existing_artifacts_in_building.artifact_count
        {
            return Err(ErrorBadRequest("Building capacity not sufficient"));
        } else if transfer.artifacts_differ + existing_artifacts_in_building.artifact_count < 0 {
            return Err(ErrorBadRequest("Not enough artifacts in the building"));
        } else {
            //Transfer Artifacts
            let new_building_artifact_count =
                existing_artifacts_in_building.artifact_count + transfer.artifacts_differ;
            let new_bank_artifact_count =
                artifacts_in_bank.artifact_count - transfer.artifacts_differ;
            web::block(move || {
                let mut conn = pg_pool.get()?;
                util::transfer_artifacts_building(
                    &mut conn,
                    &transfer.map_space_id,
                    &bank_map_space_id,
                    &new_building_artifact_count,
                    &new_bank_artifact_count,
                )
            })
            .await?
            .map_err(|err| error::handle_error(err.into()))?;
        }
    } else {
        if transfer.artifacts_differ < 0 {
            return Err(ErrorBadRequest("No artifacts in the building"));
        }
        //Create artifact record and transfer them to it
        let pg_pool_clone = pg_pool.clone();
        web::block(move || {
            let mut conn = pg_pool_clone.get()?;
            util::create_artifact_record(&mut conn, &transfer.map_space_id, &0)
        })
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

        web::block(move || {
            let mut conn = pg_pool.get()?;
            util::transfer_artifacts_building(
                &mut conn,
                &transfer.map_space_id,
                &bank_map_space_id,
                &new_building_artifact_count,
                &new_bank_artifact_count,
            )
        })
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    }
    Ok(web::Json(TransferArtifactResponse {
        building_map_space_id: transfer.map_space_id,
        artifacts_in_building: new_building_artifact_count,
        bank_map_space_id,
        artifacts_in_bank: new_bank_artifact_count,
    }))
    // }         //Uncomment to check for user under attack//
    // else{
    //     return Err(ErrorBadRequest(
    //         "Cannot transfer while under attack",
    //     ));
    // }
}

async fn get_user_base_details(pool: Data<PgPool>, user: AuthUser) -> Result<impl Responder> {
    let defender_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        let user = fetch_user(&mut conn, defender_id)?;
        let map = util::fetch_map_layout(&mut conn, &defender_id)?;
        util::get_details_from_map_layout(&mut conn, map, user)
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
        util::get_map_details_for_attack(&mut conn, map)
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
        util::get_details_from_map_layout(&mut conn, map.unwrap(), None)
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

    let map_spaces = map_spaces.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks, buildings) = web::block(move || {
        Ok((
            util::fetch_map_layout(&mut conn, &defender_id)?,
            util::fetch_blocks(&mut conn)?,
            util::fetch_buildings(&mut conn)?,
        )) as anyhow::Result<(MapLayout, HashMap<i32, BlockType>, Vec<BuildingType>)>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    validate::is_valid_update_layout(&map_spaces, &blocks, &buildings)?;

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

    let map_spaces = map_spaces.into_inner();
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (map, blocks, mut level_constraints, buildings) = web::block(move || {
        let map = util::fetch_map_layout(&mut conn, &defender_id)?;
        Ok((
            map.clone(),
            util::fetch_blocks(&mut conn)?,
            util::get_level_constraints(&mut conn, map.level_id)?,
            util::fetch_buildings(&mut conn)?,
        ))
            as anyhow::Result<(
                MapLayout,
                HashMap<i32, BlockType>,
                HashMap<i32, i32>,
                Vec<BuildingType>,
            )>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    validate::is_valid_save_layout(&map_spaces, &mut level_constraints, &blocks, &buildings)?;

    web::block(move || {
        let mut conn = pool.get()?;
        util::put_base_details(&map_spaces, &map, &mut conn)?;
        util::calculate_shortest_paths(&mut conn, map.id, &buildings)?;
        util::set_map_valid(&mut conn, map.id)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok("Saved successfully")
}

async fn defense_history(
    user: AuthUser,
    query: web::Query<HistoryboardQuery>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let user_id = user.0;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    if page <= 0 || limit <= 0 {
        return Err(ErrorBadRequest("Invalid query params"));
    }
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_defense_historyboard(user_id, page, limit, &mut conn)
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
