use super::error;
use crate::models::LevelsFixture;
use actix_web::error::ErrorBadRequest;
use actix_web::{web, HttpResponse, Responder, Result};
use anyhow::Context;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use std::collections::HashSet;
use util::NewAttack;

mod util;
mod validate;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::post().to(create_attack)))
        .service(web::resource("/{attacker_id}/history").route(web::get().to(attack_history)))
        .service(web::resource("/top").route(web::get().to(get_top_attacks)));
}

type DbPool = Pool<ConnectionManager<PgConnection>>;

async fn create_attack(
    new_attack: web::Json<NewAttack>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    // TODO: get attacker_id from session
    let attacker_id = 1;

    if !util::is_attack_allowed_now() {
        return Err(ErrorBadRequest("Attack not allowed"));
    }

    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let defender_id = new_attack.defender_id;
    let (level, map_id, valid_road_paths, valid_emp_ids, is_attack_allowed) =
        web::block(move || {
            let level = util::get_current_levels_fixture(&conn)?;
            let map_id = util::get_map_id(&defender_id, &level.id, &conn)?;
            let is_attack_allowed = util::is_attack_allowed(attacker_id, defender_id, &conn)?;
            let valid_emp_ids: HashSet<i32> = util::get_valid_emp_ids(&conn)?;
            let valid_road_paths = util::get_valid_road_paths(map_id, &conn)?;
            Ok((
                level,
                map_id,
                valid_road_paths,
                valid_emp_ids,
                is_attack_allowed,
            ))
                as anyhow::Result<(LevelsFixture, i32, HashSet<(i32, i32)>, HashSet<i32>, bool)>
        })
        .await
        .map_err(|err| error::handle_error(err.into()))?;

    if !is_attack_allowed {
        return Err(ErrorBadRequest("Invalid attack"));
    }

    if !validate::is_attack_valid(
        &new_attack,
        valid_road_paths,
        valid_emp_ids,
        &level.no_of_bombs,
    ) {
        return Err(ErrorBadRequest("Invalid attack path"));
    }
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let game_id = web::block(move || util::insert_attack(attacker_id, &new_attack, map_id, &conn))
        .await
        .map_err(|err| error::handle_error(err.into()))?;

    let file_content = web::block(move || {
        let conn = pool.get()?;
        util::run_simulation(game_id, &conn)
            .with_context(|| format!("Failed to run simulation for game {}", game_id))
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(HttpResponse::Ok().body(file_content))
}

async fn attack_history(
    attacker_id: web::Path<i32>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    // TODO: get user_id from session
    let user_id = 1;
    let attacker_id = attacker_id.0;
    let response = web::block(move || {
        let conn = pool.get()?;
        util::fetch_attack_history(attacker_id, user_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn get_top_attacks(pool: web::Data<DbPool>) -> Result<impl Responder> {
    // TODO: get user_id from session
    let user_id = 1;
    let response = web::block(move || {
        let conn = pool.get()?;
        util::fetch_top_attacks(user_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}
