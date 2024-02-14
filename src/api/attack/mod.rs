use self::util::{get_valid_road_paths, AttackResponse, GameLog, NewAttack, ResultResponse};
use super::auth::session::AuthUser;
use super::defense::util::{
    AttackBaseResponse, DefenseResponse, MineTypeResponseWithoutBlockId, SimulationBaseResponse,
};
use super::user::util::fetch_user;
use super::{error, PgPool, RedisPool};
use crate::api;
use crate::api::attack::socket::{ResultType, SocketRequest};
use crate::api::attack::util::ShortestPathResponse;
use crate::api::defense::util::calculate_shortest_paths;
use crate::api::util::HistoryboardQuery;
use crate::constants::MAX_BOMBS_PER_ATTACK;
use crate::models::{AttackerType, LevelsFixture, User};
use crate::simulation::blocks::{Coords, SourceDest};
use crate::validator::state::State;
use crate::validator::util::{BombType, BuildingDetails, DefenderDetails, MineDetails};
use actix_rt;
use actix_web::error::ErrorBadRequest;
use actix_web::web::{Data, Json};
use actix_web::{web, Error, HttpRequest, HttpResponse, Responder, Result};
use std::collections::{HashMap, HashSet};

use crate::validator::game_handler;
use actix_ws::Message;
use futures_util::stream::StreamExt;
use std::time::Instant;

mod rating;
pub mod socket;
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

    // Check if attacker is already in a game
    if let Ok(Some(_)) = util::get_game_id_from_redis(attacker_id, redis_conn) {
        return Err(ErrorBadRequest("Only one attack is allowed at a time"));
    }

    //Generate random opponent id
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

    //Fetch base details and shortest paths data
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

    web::block(move || calculate_shortest_paths(&mut conn, map_id) as anyhow::Result<()>)
        .await?
        .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let shortest_paths = web::block(move || {
        Ok(util::get_shortest_paths_for_attack(&mut conn, map_id)?)
            as anyhow::Result<Vec<ShortestPathResponse>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let user_details =
        web::block(move || Ok(fetch_user(&mut conn, opponent_id)?) as anyhow::Result<Option<User>>)
            .await?
            .map_err(|err| error::handle_error(err.into()))?;
    //
    //Generate attack token to validate the /attack/start
    let attack_token = util::encode_attack_token(attacker_id, opponent_id).unwrap();
    let response: AttackResponse = AttackResponse {
        user: user_details,
        max_bombs: MAX_BOMBS_PER_ATTACK,
        base: AttackBaseResponse {
            map_spaces: opponent_base.map_spaces,
            defender_types: opponent_base.defender_types,
            blocks: opponent_base.blocks,
            mine_types: opponent_base
                .mine_types
                .iter()
                .map(|mine_type| MineTypeResponseWithoutBlockId {
                    id: mine_type.id,
                    name: mine_type.name.clone(),
                    damage: mine_type.damage,
                    cost: mine_type.cost,
                    level: mine_type.level,
                    radius: mine_type.radius,
                })
                .collect(),
        },
        shortest_paths,
        attack_token,
        attacker_types: opponent_base.attacker_types,
        bomb_types: opponent_base.bomb_types,
    };

    Ok(Json(response))
}

async fn socket_handler(
    pool: web::Data<PgPool>,
    redis_pool: Data<RedisPool>,
    req: HttpRequest,
    body: web::Payload,
) -> Result<HttpResponse, Error> {
    let query_params = req.query_string().split('&').collect::<Vec<&str>>();
    let user_token = query_params[0].split('=').collect::<Vec<&str>>()[1];
    let attack_token = query_params[1].split('=').collect::<Vec<&str>>()[1];

    let attacker_id = util::decode_user_token(user_token).unwrap();
    let attack_token_data = util::decode_attack_token(attack_token).unwrap();

    if attacker_id != attack_token_data.attacker_id {
        return Err(ErrorBadRequest("User not authorised"));
    }

    let defender_id = attack_token_data.defender_id;

    if attacker_id == defender_id {
        return Err(ErrorBadRequest("Can't attack yourself"));
    }

    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    if let Ok(Some(_)) = util::get_game_id_from_redis(attacker_id, redis_conn) {
        return Err(ErrorBadRequest("Only one attack is allowed at a time"));
    }

    //Fetch map_id of the defender
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let map = web::block(move || {
        let map = util::get_map_id(&defender_id, &mut conn)?;
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

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    //Create game
    let game_id = web::block(move || {
        Ok(util::add_game(attacker_id, defender_id, map_id, &mut conn)?) as anyhow::Result<i32>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    //Fetch base details and shortest paths data
    //Fetch defender details, fetch defender details

    //Store the game id in redis
    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    if util::add_game_id_to_redis(attacker_id, defender_id, game_id, redis_conn).is_err() {
        return Err(ErrorBadRequest("Internal Server Error"));
    }

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let defenders = web::block(move || {
        Ok(util::get_defenders(&mut conn, map_id)?) as anyhow::Result<Vec<DefenderDetails>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let mines = web::block(move || {
        Ok(util::get_mines(&mut conn, map_id)?) as anyhow::Result<Vec<MineDetails>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let buildings = web::block(move || {
        Ok(util::get_buildings(&mut conn, map_id)?) as anyhow::Result<Vec<BuildingDetails>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let roads = web::block(move || {
        Ok(get_valid_road_paths(map_id, &mut conn)?) as anyhow::Result<HashSet<(i32, i32)>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let bomb_types =
        web::block(move || Ok(util::get_bomb_types(&mut conn)?) as anyhow::Result<Vec<BombType>>)
            .await?
            .map_err(|err| error::handle_error(err.into()))?;

    let (response, session, mut msg_stream) = actix_ws::handle(&req, body)?;

    let mut session_clone = session.clone();

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let attacker_type = web::block(move || {
        Ok(util::get_attacker_types(&mut conn)?) as anyhow::Result<HashMap<i32, AttackerType>>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let attacker_user_details =
        web::block(move || Ok(fetch_user(&mut conn, attacker_id)?) as anyhow::Result<Option<User>>)
            .await?
            .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let defender_user_details =
        web::block(move || Ok(fetch_user(&mut conn, defender_id)?) as anyhow::Result<Option<User>>)
            .await?
            .map_err(|err| error::handle_error(err.into()))?;

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;

    let defender_base_details = web::block(move || {
        Ok(util::get_opponent_base_details(defender_id, &mut conn)?)
            as anyhow::Result<DefenseResponse>
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    if attacker_user_details.is_none() || defender_user_details.is_none() {
        return Err(ErrorBadRequest("Internal Server Error"));
    }

    let game_log: GameLog = GameLog {
        attacker: attacker_user_details.unwrap(),
        defender: defender_user_details.unwrap(),
        base: SimulationBaseResponse {
            map_spaces: defender_base_details.map_spaces,
            defender_types: defender_base_details.defender_types,
            blocks: defender_base_details.blocks,
            mine_types: defender_base_details.mine_types,
            attacker_types: defender_base_details.attacker_types,
            bomb_types: defender_base_details.bomb_types,
        },
        events: Vec::new(),
        result: ResultResponse {
            damage_done: 0,
            artifacts_collected: 0,
            bombs_used: 0,
            attackers_used: 0,
            is_attacker_alive: true,
            new_attacker_trophies: 0,
            new_defender_trophies: 0,
            old_attacker_trophies: 0,
            old_defender_trophies: 0,
        },
    };

    actix_rt::spawn(async move {
        let mut game_state = State::new(attacker_id, defender_id, defenders, mines, buildings);

        let mut game_logs = &mut game_log.clone();

        let mut conn = pool
            .get()
            .map_err(|err| error::handle_error(err.into()))
            .unwrap();
        let shortest_path = &shortest_paths.clone();
        let roads = &roads.clone();
        let bomb_types = &bomb_types.clone();
        let attacker_type = &attacker_type.clone();

        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session_clone.pong(&bytes).await.is_err() {
                        return;
                    }
                }
                Message::Text(s) => {
                    println!("Received JSON message: {}", s);
                    if let Ok(socket_request) = serde_json::from_str::<SocketRequest>(&s) {
                        println!("Parsed JSON message: {:?}", socket_request);
                        let response_result = game_handler(
                            attacker_type,
                            socket_request,
                            &mut game_state,
                            shortest_path,
                            roads,
                            bomb_types,
                            &mut game_logs,
                        );
                        match response_result {
                            Some(Ok(response)) => {
                                if let Ok(response_json) = serde_json::to_string(&response) {
                                    if response.result_type == ResultType::GameOver {
                                        println!("Game over. Terminating the socket...");
                                        if session_clone.text(response_json).await.is_err() {
                                            return;
                                        }
                                        let _ = session_clone.clone().close(None).await;
                                        let _ = util::terminate_game(&mut game_logs, &mut conn);
                                    }
                                } else {
                                    println!("Error serializing JSON");
                                    if session_clone.text("Error serializing JSON").await.is_err() {
                                        return;
                                    }
                                }
                            }
                            Some(Err(err)) => {
                                println!("Error handling game: {:?}", err);
                            }
                            None => {
                                // Handle the case where game_handler returned None (e.g., ActionType::PlaceAttacker)
                                // Add appropriate logic here based on the requirements.
                                println!("All fine");
                            }
                        }
                    } else {
                        println!("Error parsing JSON");
                        if session_clone.text("Error parsing JSON").await.is_err() {
                            return;
                        }
                    }
                }
                Message::Close(s) => {
                    println!("Received close: {:?}", s);
                    let _ = util::terminate_game(&mut game_logs, &mut conn);
                    break;
                }
                _ => (),
            }
        }

        let _ = session_clone.close(None).await;
    });

    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    actix_rt::spawn(util::timeout_task(
        session,
        Instant::now(),
        redis_conn,
        attacker_id,
        defender_id,
    ));
    Ok(response)
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
