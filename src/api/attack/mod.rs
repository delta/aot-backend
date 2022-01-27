use std::collections::HashSet;

use actix_web::{web, HttpResponse, Responder, Result};
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;

mod util;
mod validate;

use crate::models::LevelsFixture;

use super::error;
use util::NewAttack;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/list").route(web::get().to(list_leaderboard)));
    cfg.service(web::resource("").route(web::post().to(create_attack)));
    cfg.service(web::resource("/history").route(web::get().to(attack_history)));
}

type DbPool = Pool<ConnectionManager<PgConnection>>;

async fn create_attack(
    new_attack: web::Json<NewAttack>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    // TODO: get attacker_id from session
    let attacker_id = 1;

    if !util::is_attack_allowed_now() {
        return Ok(HttpResponse::BadRequest().body("Attack not allowed"));
    }

    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let defender_id = new_attack.defender_id;
    let (level, map_id, valid_road_paths, valid_emp_ids, is_attack_allowed) =
        web::block(move || {
            let level = util::get_current_levels_fixture(&conn)?;
            let map_id = util::get_map_id(&defender_id, &level.id, &conn)?;
            let is_attack_allowed = util::is_attack_valid(attacker_id, defender_id, &conn)?;
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
        return Ok(HttpResponse::BadRequest().body("Invalid attack"));
    }

    if !validate::is_attack_valid(
        &new_attack,
        valid_road_paths,
        valid_emp_ids,
        &level.no_of_bombs,
    ) {
        return Ok(HttpResponse::BadRequest().body("Invalid attack path"));
    }
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    web::block(move || {
        Ok(util::insert_attack(
            attacker_id,
            &new_attack,
            map_id,
            &conn,
        )?) as anyhow::Result<()>
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;

    // TODO: Simulate attack, generate csv, send csv response

    Ok(HttpResponse::Ok().body("Added Attack"))
}

async fn attack_history(pool: web::Data<DbPool>) -> Result<impl Responder> {
    // TODO: get attacker_id from session
    let attacker_id = 1;
    let response = web::block(move || {
        let conn = pool.get()?;
        util::get_attack_history(attacker_id, &conn)
    })
    .await
    .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(web::Json(response))
}

async fn list_leaderboard() -> impl Responder {
    HttpResponse::Ok().body("Todo")
}
