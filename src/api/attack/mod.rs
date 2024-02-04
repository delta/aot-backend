use self::util::{AttackResponse, NewAttack};
use super::auth::session::AuthUser;
use super::defense::util::DefenseResponse;
use super::{error, PgPool, RedisPool};
use crate::api;
use crate::api::socket::Socket;
use crate::api::util::HistoryboardQuery;
use crate::models::{AttackerType, LevelsFixture};
use crate::simulation::blocks::{Coords, SourceDest};
use actix_web::error::ErrorBadRequest;
use actix_web::web::{Data, Json};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, Result};
use actix_web_actors::ws;
use std::collections::{HashMap, HashSet};

mod rating;
pub mod util;
mod validate;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(init_attack)))
        .service(web::resource("/start").route(web::get().to(socket_handler)))
        .service(web::resource("/history").route(web::get().to(attack_history)))
        .service(web::resource("/top").route(web::get().to(get_top_attacks)))
        .service(web::resource("/testbase").route(web::post().to(test_base)));
}

// /attack (Get)
// Get user id from auth user *
// Find random opponent id *
// Get base of opponent id (no mine)
// Get shortest path of base id
// Response:
// Jwt using user id, opponent id
// Base, shortest paths of opponent

async fn init_attack(
    pool: web::Data<PgPool>,
    redis_pool: Data<RedisPool>,
    user: AuthUser,
) -> Result<impl Responder> {
    let attacker_id = user.0;
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    if let Ok(Some(_)) = util::get_from_redis(attacker_id, redis_conn) {
        return Err(ErrorBadRequest("Only one attack is allowed at a time"));
    }

    let random_opponent_id = web::block(move || {
        Ok(util::get_random_opponent_id(attacker_id, &mut conn)?) as anyhow::Result<Option<i32>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    if random_opponent_id.is_none() {
        return Err(ErrorBadRequest("No opponent found"));
    }

    let opponent_id = random_opponent_id.unwrap();

    // let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    // let (is_attack_allowed) =
    //     web::block(move || {
    //         let is_attack_allowed = util::is_attack_allowed(attacker_id, opponent_id, &mut conn)?;
    //         Ok((
    //             is_attack_allowed,
    //         ))
    //             as anyhow::Result<(
    //                 bool,
    //             )>
    //     })
    //     .await?
    //     .map_err(|err| error::handle_error(err.into()))?;

    // if !is_attack_allowed {
    //     return Err(ErrorBadRequest("Attack not allowed"));
    // }

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let opponent_base = web::block(move || {
        Ok(util::get_opponent_base_details(opponent_id, &mut conn)?)
            as anyhow::Result<DefenseResponse>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let map = web::block(move || {
        let map = util::get_map_id(&opponent_id, &mut conn)?;
        Ok(map) as anyhow::Result<Option<i32>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let map_id = if let Some(map) = map {
        map
    } else {
        return Err(ErrorBadRequest("Invalid base"));
    };

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let shortest_paths = web::block(move || {
        Ok(util::get_shortest_paths(&mut conn, map_id)?)
            as anyhow::Result<HashMap<SourceDest, Coords>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let attack_token = util::generate_attack_token(attacker_id, opponent_id).unwrap();
    let response: AttackResponse = AttackResponse {
        base: opponent_base,
        shortest_paths,
        attack_token,
    };

    Ok(Json(response))
}

async fn socket_handler(
    pool: web::Data<PgPool>,
    redis_pool: Data<RedisPool>,
    req: HttpRequest,
    stream: web::Payload,
    user: AuthUser,
) -> Result<HttpResponse, Error> {
    println!("User id: {}", user.0);
    let user_id_str = req
        .query_string()
        .strip_prefix("user_id=")
        .ok_or_else(|| ErrorBadRequest("'user_id' not found in the query string"))?;
    let user_id: i32 = user_id_str
        .parse()
        .map_err(|_| ErrorBadRequest("Failed to parse 'user_id' into an integer"))?;

    // //Create new game
    // let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    // let game_id = web::block(move || {
    //     Ok(util::add_game(attacker_id, opponent_id, map_id, &mut conn)?) as anyhow::Result<i32>
    // })
    // .await?
    // .map_err(|err| error::handle_error(err.into()))?;

    // //Store the game id in redis
    // let redis_conn = redis_pool.get().map_err(|err| error::handle_error(err.into()))?;
    // if let Err(_) = util::add_to_redis(attacker_id, game_id, redis_conn) {
    //     return Err(ErrorBadRequest("Internal Server Error"));
    // }

    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    if let Ok(Some(game_id)) = util::get_from_redis(user_id, redis_conn) {
        ws::start(Socket { game_id }, &req, stream)
    } else {
        return Err(ErrorBadRequest("No game found for the user"));
    }
}

// async fn create_attack(
//     new_attack: web::Json<NewAttack>,
//     pool: web::Data<PgPool>,
//     user: AuthUser,
// ) -> Result<impl Responder> {
//     let attacker_id = user.0;
//     let attackers = new_attack.attackers.clone();

//     let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
//     let defender_id = new_attack.defender_id;
//     let (level, map) = web::block(move || {
//         let level = api::util::get_current_levels_fixture(&mut conn)?;
//         let map = util::get_map_id(&defender_id, &level.id, &mut conn)?;
//         Ok((level, map)) as anyhow::Result<(LevelsFixture, Option<i32>)>
//     })
//     .await?
//     .map_err(|err| error::handle_error(err.into()))?;

//     let map_id = if let Some(map) = map {
//         map
//     } else {
//         return Err(ErrorBadRequest("Invalid base"));
//     };

//     let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
//     let (valid_road_paths, valid_emp_ids, is_attack_allowed, attacker_types) =
//         web::block(move || {
//             let is_attack_allowed = util::is_attack_allowed(attacker_id, defender_id, &mut conn)?;
//             let valid_emp_ids: HashSet<i32> = util::get_valid_emp_ids(&mut conn)?;
//             let valid_road_paths = util::get_valid_road_paths(map_id, &mut conn)?;
//             let attacker_types = util::get_attacker_types(&mut conn)?;
//             Ok((
//                 valid_road_paths,
//                 valid_emp_ids,
//                 is_attack_allowed,
//                 attacker_types,
//             ))
//                 as anyhow::Result<(
//                     HashSet<(i32, i32)>,
//                     HashSet<i32>,
//                     bool,
//                     HashMap<i32, AttackerType>,
//                 )>
//         })
//         .await?
//         .map_err(|err| error::handle_error(err.into()))?;

//     if !is_attack_allowed {
//         return Err(ErrorBadRequest("Attack not allowed"));
//     }

//     validate::is_attack_valid(
//         &new_attack,
//         valid_road_paths,
//         valid_emp_ids,
//         &level.no_of_bombs,
//         &level.no_of_attackers,
//         &attacker_types,
//     )
//     .map_err(ErrorBadRequest)?;

//     let file_content = web::block(move || {
//         let mut conn = pool.get()?;
//         let game_id = util::add_game(attacker_id, &new_attack, map_id, &mut conn)?;
//         let sim_result = util::run_simulation(game_id, map_id, attackers, &mut conn);
//         match sim_result {
//             Ok(file_content) => Ok(file_content),
//             Err(_) => {
//                 remove_game(game_id, &mut conn)?;
//                 Err(anyhow::anyhow!(
//                     "Failed to run simulation for game {}",
//                     game_id
//                 ))
//             }
//         }
//     })
//     .await?
//     .map_err(|err| error::handle_error(err.into()))?;

//     Ok(HttpResponse::Ok().body(file_content))
// }

async fn attack_history(
    pool: web::Data<PgPool>,
    user: AuthUser,
    query: web::Query<HistoryboardQuery>,
) -> Result<impl Responder> {
    let user_id = user.0;
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    if page <= 0 || limit <= 0 {
        return Err(ErrorBadRequest("Invalid query params"));
    }
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_attack_history(user_id, page, limit, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn get_top_attacks(pool: web::Data<PgPool>, user: AuthUser) -> Result<impl Responder> {
    let user_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_top_attacks(user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn test_base(
    new_attack: web::Json<NewAttack>,
    pool: web::Data<PgPool>,
    user: AuthUser,
) -> Result<impl Responder> {
    let player_id = user.0;
    if new_attack.defender_id != player_id {
        return Err(ErrorBadRequest("Player not authorised"));
    }

    let attackers = new_attack.attackers.clone();

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (level, map) = web::block(move || {
        let level = api::util::get_current_levels_fixture(&mut conn)?;
        let map = util::get_map_id(&player_id, &mut conn)?;
        Ok((level, map)) as anyhow::Result<(LevelsFixture, Option<i32>)>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let map_id = if let Some(map) = map {
        map
    } else {
        return Err(ErrorBadRequest("Invalid base"));
    };

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let (valid_road_paths, valid_emp_ids, attacker_types) = web::block(move || {
        let valid_emp_ids: HashSet<i32> = util::get_valid_emp_ids(&mut conn)?;
        let valid_road_paths = util::get_valid_road_paths(map_id, &mut conn)?;
        let attacker_types = util::get_attacker_types(&mut conn)?;
        Ok((valid_road_paths, valid_emp_ids, attacker_types))
            as anyhow::Result<(
                HashSet<(i32, i32)>,
                HashSet<i32>,
                HashMap<i32, AttackerType>,
            )>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    validate::is_attack_valid(
        &new_attack,
        valid_road_paths,
        valid_emp_ids,
        &level.no_of_bombs,
        &level.no_of_attackers,
        &attacker_types,
    )
    .map_err(ErrorBadRequest)?;

    let file_content = web::block(move || {
        let mut conn = pool.get()?;
        let sim_result = util::run_test_base_simulation(map_id, attackers, &mut conn);
        match sim_result {
            Ok(file_content) => Ok(file_content),
            Err(_) => Err(anyhow::anyhow!("Failed to run test base simulation")),
        }
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(HttpResponse::Ok().body(file_content))
}
