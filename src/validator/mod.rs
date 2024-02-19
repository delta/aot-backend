use std::collections::{HashMap, HashSet};

use crate::{
    api::attack::{
        socket::{
            self, ActionType, BuildingResponse, DefenderResponse, GameStateResponse, ResultType,
            SocketRequest, SocketResponse,
        },
        util::{Direction, EventResponse, GameLog},
    },
    models::AttackerType,
    simulation::blocks::{Coords, SourceDest},
};
use anyhow::{Ok, Result};
use r2d2::event;

use self::{
    state::State,
    util::{Attacker, BombType, BuildingDetails, DefenderDetails, DefenderReturnType, MineDetails},
};

pub mod error;
pub mod state;
pub mod util;

pub fn game_handler(
    attacker_type: &HashMap<i32, AttackerType>,
    socket_request: SocketRequest,
    _game_state: &mut State,
    _shortest_path: &HashMap<SourceDest, Coords>,
    _roads: &HashSet<(i32, i32)>,
    _bomb_types: &Vec<BombType>,
    mut _game_log: &mut GameLog,
) -> Option<Result<SocketResponse>> {
    let defender_trigger_result: DefenderReturnType;
    let exploded_mines_result: Vec<MineDetails>;
    let buildings_damaged_result: Vec<BuildingResponse>;

    // _game_log.result.artifacts_collected += 1; //To check if modif is working as mutex

    match socket_request.action_type {
        ActionType::PlaceAttacker => {
            let mut event_response = EventResponse {
                attacker_id: None,
                bomb_id: None,
                coords: Coords { x: 0, y: 0 },
                direction: Direction::Up,
                is_bomb: false,
            };

            dotenv::dotenv().ok();

            if socket_request.frame_number == 1 {
                let bomb_max_count = std::env::var("BOMBS_MAX_COUNT")
                    .unwrap_or("0".to_string())
                    .parse::<i32>()
                    .unwrap_or(0);
                for bomb_type in _bomb_types {
                    if let Some(bomb_id) = socket_request.bomb_id {
                        if bomb_type.id == bomb_id {
                            _game_state.set_bombs(bomb_type.clone(), bomb_max_count);
                        }
                    }
                }

                // _game_state.set_mines(mine_positions);
                event_response.bomb_id = socket_request.bomb_id;
            }

            if let Some(attacker_id) = socket_request.attacker_id {
                let attacker: AttackerType = attacker_type.get(&attacker_id).unwrap().clone();
                _game_state.place_attacker(Attacker {
                    id: attacker.id,
                    path_in_current_frame: Vec::new(),
                    attacker_pos: socket_request.start_position.unwrap(),
                    attacker_health: attacker.max_health,
                    attacker_speed: attacker.speed,
                    bombs: Vec::new(),
                    trigger_defender: false,
                });

                event_response.attacker_id = Some(attacker_id);
                event_response.coords = socket_request.start_position.unwrap();
            }

            _game_state.update_frame_number(socket_request.frame_number.clone());

            _game_log.events.push(event_response);
            _game_log.result.attackers_used += 1;

            return Some(Ok(SocketResponse {
                frame_number: socket_request.frame_number,
                result_type: ResultType::PlacedAttacker,
                is_alive: Some(true),

                attacker_health: None,
                exploded_mines: None,
                triggered_defenders: None,
                // defender_damaged: return_state.unwrap().frame_no,
                damaged_buildings: None,
                artifacts_gained_total: None,
                is_sync: false,
                // state: Some(GameStateResponse {
                //     frame_no: defender_trigger_result.clone().unwrap().0,
                //     attacker_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_user_id,
                //     defender_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .defender_user_id,
                //     attacker: defender_trigger_result.clone().unwrap().2.attacker,
                //     attacker_death_count: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_death_count,
                //     bombs: defender_trigger_result.clone().unwrap().2.bombs,
                //     damage_percentage: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .damage_percentage,
                //     artifacts: defender_trigger_result.clone().unwrap().2.artifacts,
                //     defenders: defender_trigger_result.clone().unwrap().2.defenders,
                //     mines: defender_trigger_result.clone().unwrap().2.mines,
                //     buildings: defender_trigger_result.clone().unwrap().2.buildings,
                //     total_hp_buildings: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .total_hp_buildings,
                // }),
                is_game_over: false,
                message: Some(String::from("return test")),
            }));
        }
        ActionType::MoveAttacker => {
            // move_attacker
            // State::new()

            if let Some(attacker_id) = socket_request.attacker_id {
                let attacker: AttackerType = attacker_type.get(&attacker_id).unwrap().clone();
                let attacker_delta: Vec<Coords> = socket_request.attacker_path;

                let attacker_result = _game_state.attacker_movement(
                    socket_request.frame_number.clone(),
                    _roads,
                    Attacker {
                        id: attacker.id,
                        path_in_current_frame: attacker_delta.clone(),
                        attacker_pos: socket_request.start_position.unwrap(),
                        attacker_health: attacker.max_health,
                        attacker_speed: attacker.speed,
                        bombs: Vec::new(),
                        trigger_defender: false,
                    },
                );

                let attacker_result_clone = attacker_result.clone();

                defender_trigger_result =
                    _game_state.defender_movement(attacker_delta.clone(), _shortest_path);

                // .map(|(a, b, c)| (a.clone(), b.clone(), c.clone())).clone();
                // Some(Err(error::FrameError { frame_no: 0 }.into()))

                for coord in attacker_delta {
                    let mut direction = Direction::Up;

                    let prev_pos = _game_log.events.last().unwrap().coords.clone();
                    if prev_pos.x < coord.x {
                        direction = Direction::Down;
                    } else if prev_pos.x > coord.x {
                        direction = Direction::Up;
                    } else if prev_pos.y < coord.y {
                        direction = Direction::Left;
                    } else if prev_pos.y > coord.y {
                        direction = Direction::Right;
                    }

                    let event_response = EventResponse {
                        attacker_id: None,
                        bomb_id: None,
                        coords: coord,
                        direction,
                        is_bomb: false,
                    };

                    _game_log.events.push(event_response.clone());
                }

                let mut bool_temp = false;
                if attacker_result_clone.unwrap().trigger_defender {
                    bool_temp = true;
                }
                let result_type = if bool_temp {
                    ResultType::DefendersTriggered
                } else {
                    ResultType::Nothing
                };

                return Some(Ok(SocketResponse {
                    frame_number: socket_request.frame_number,
                    result_type,
                    is_alive: Some(true),
                    attacker_health: Some(defender_trigger_result.clone().attacker_health),
                    exploded_mines: None,
                    triggered_defenders: Some(defender_trigger_result.clone().defender_response),
                    // defender_damaged: return_state.unwrap().frame_no,
                    damaged_buildings: None,
                    artifacts_gained_total: Some(defender_trigger_result.clone().state.artifacts),
                    is_sync: false,
                    // state: Some(GameStateResponse {
                    //     frame_no: defender_trigger_result.clone().unwrap().0,
                    //     attacker_user_id: defender_trigger_result
                    //         .clone()
                    //         .unwrap()
                    //         .2
                    //         .attacker_user_id,
                    //     defender_user_id: defender_trigger_result
                    //         .clone()
                    //         .unwrap()
                    //         .2
                    //         .defender_user_id,
                    //     attacker: defender_trigger_result.clone().unwrap().2.attacker,
                    //     attacker_death_count: defender_trigger_result
                    //         .clone()
                    //         .unwrap()
                    //         .2
                    //         .attacker_death_count,
                    //     bombs: defender_trigger_result.clone().unwrap().2.bombs,
                    //     damage_percentage: defender_trigger_result
                    //         .clone()
                    //         .unwrap()
                    //         .2
                    //         .damage_percentage,
                    //     artifacts: defender_trigger_result.clone().unwrap().2.artifacts,
                    //     defenders: defender_trigger_result.clone().unwrap().2.defenders,
                    //     mines: defender_trigger_result.clone().unwrap().2.mines,
                    //     buildings: defender_trigger_result.clone().unwrap().2.buildings,
                    //     total_hp_buildings: defender_trigger_result
                    //         .clone()
                    //         .unwrap()
                    //         .2
                    //         .total_hp_buildings,
                    // }),
                    is_game_over: false,
                    message: Some(String::from("return test")),
                }));
            }
        }
        ActionType::IsMine => {
            // is_mine
            let start_pos: Option<Coords> = socket_request.start_position;
            exploded_mines_result = _game_state.mine_blast(start_pos);

            let mut bool_temp = false;
            if exploded_mines_result.clone().len() > 0 {
                bool_temp = true;
            }
            let result_type = if bool_temp {
                ResultType::MinesExploded
            } else {
                ResultType::Nothing
            };

            return Some(Ok(SocketResponse {
                frame_number: socket_request.frame_number,
                result_type,
                is_alive: Some(true),

                attacker_health: None,
                exploded_mines: Some(exploded_mines_result),
                triggered_defenders: None,
                // defender_damaged: return_state.unwrap().frame_no,
                damaged_buildings: None,
                artifacts_gained_total: None,
                is_sync: false,
                // state: Some(GameStateResponse {
                //     frame_no: defender_trigger_result.clone().unwrap().0,
                //     attacker_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_user_id,
                //     defender_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .defender_user_id,
                //     attacker: defender_trigger_result.clone().unwrap().2.attacker,
                //     attacker_death_count: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_death_count,
                //     bombs: defender_trigger_result.clone().unwrap().2.bombs,
                //     damage_percentage: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .damage_percentage,
                //     artifacts: defender_trigger_result.clone().unwrap().2.artifacts,
                //     defenders: defender_trigger_result.clone().unwrap().2.defenders,
                //     mines: defender_trigger_result.clone().unwrap().2.mines,
                //     buildings: defender_trigger_result.clone().unwrap().2.buildings,
                //     total_hp_buildings: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .total_hp_buildings,
                // }),
                is_game_over: false,
                message: Some(String::from("return test")),
            }));
        }
        ActionType::PlaceBombs => {
            // place_bombs
            let attacker_delta: Vec<Coords> = socket_request.attacker_path;
            let bomb_coords = socket_request.bomb_position;

            for coord in attacker_delta.clone() {
                let mut direction = Direction::Up;

                let prev_pos = _game_log.events.last().unwrap().coords.clone();
                if prev_pos.x < coord.x {
                    direction = Direction::Down;
                } else if prev_pos.x > coord.x {
                    direction = Direction::Up;
                } else if prev_pos.y < coord.y {
                    direction = Direction::Left;
                } else if prev_pos.y > coord.y {
                    direction = Direction::Right;
                }

                let event_response = EventResponse {
                    attacker_id: None,
                    bomb_id: None,
                    coords: coord,
                    direction,
                    is_bomb: coord == bomb_coords,
                };

                _game_log.events.push(event_response.clone());
            }

            _game_log.result.bombs_used += 1;
            _game_log.result.damage_done = _game_state.damage_percentage as i32;
            _game_log.result.artifacts_collected = _game_state.artifacts;

            buildings_damaged_result =
                _game_state.place_bombs(attacker_delta, bomb_coords);

            let mut bool_temp = false;
            if buildings_damaged_result.clone().len() > 0 {
                bool_temp = true;
            }
            let result_type = if bool_temp {
                ResultType::BuildingsDamaged
            } else {
                ResultType::Nothing
            };

            return Some(Ok(SocketResponse {
                frame_number: socket_request.frame_number,
                result_type,
                is_alive: Some(true),

                attacker_health: None,
                exploded_mines: None,
                triggered_defenders: None,
                // defender_damaged: return_state.unwrap().frame_no,
                damaged_buildings: Some(buildings_damaged_result),
                artifacts_gained_total: None,
                is_sync: false,
                // state: Some(GameStateResponse {
                //     frame_no: defender_trigger_result.clone().unwrap().0,
                //     attacker_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_user_id,
                //     defender_user_id: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .defender_user_id,
                //     attacker: defender_trigger_result.clone().unwrap().2.attacker,
                //     attacker_death_count: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_death_count,
                //     bombs: defender_trigger_result.clone().unwrap().2.bombs,
                //     damage_percentage: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .damage_percentage,
                //     artifacts: defender_trigger_result.clone().unwrap().2.artifacts,
                //     defenders: defender_trigger_result.clone().unwrap().2.defenders,
                //     mines: defender_trigger_result.clone().unwrap().2.mines,
                //     buildings: defender_trigger_result.clone().unwrap().2.buildings,
                //     total_hp_buildings: defender_trigger_result
                //         .clone()
                //         .unwrap()
                //         .2
                //         .total_hp_buildings,
                // }),
                is_game_over: false,
                message: Some(String::from("return test")),
            }));
        }
        ActionType::Idle => {
            // idle (waiting for user to choose next attacker)
            return Some(Ok(SocketResponse {
                frame_number: socket_request.frame_number,
                result_type: ResultType::Nothing,
                is_alive: Some(true),

                attacker_health: None,
                exploded_mines: None,
                triggered_defenders: None,
                // defender_damaged: return_state.unwrap().frame_no,
                damaged_buildings: None,
                artifacts_gained_total: None,
                is_sync: false,
                // state: Some(GameStateResponse {
                //     frame_no: socket_request.frame_number,
                //     attacker_user_id: 0,
                //     defender_user_id:0,
                //     attacker:None,
                //     attacker_death_count: 0,
                //     bombs: BombType {
                //         id: 0,
                //         radius: 0,
                //         damage: 0,
                //         total_count: 0,
                //     },
                //     damage_percentage: 0.0,
                //     artifacts: 0,
                //     defenders: [DefenderDetails{
                //         id: 0,
                //         radius: 0,
                //         speed: 0,
                //         damage: 0,
                //         defender_pos: Coords{x:0, y:0},
                //         is_alive: false,
                //         damage_dealt: false,
                //         target_id: None,
                //         path_in_current_frame: Vec::new(),
                //     }].to_vec(),
                //     mines: [MineDetails{
                //         id: 0,
                //         pos: Coords{x:0, y:0},
                //         radius: 0,
                //         damage: 0,
                //     }].to_vec(),

                //     buildings: [BuildingDetails{
                //         id: 0,
                //         current_hp: 0,
                //         total_hp: 0,
                //         artifacts_obtained: 0,
                //         tile: Coords{x:0, y:0},
                //         width: 0,
                //     }].to_vec(),
                //     total_hp_buildings:100,
                // }),
                is_game_over: false,
                message: Some(String::from("return test")),
            }));
        }
        ActionType::Terminate => {
            let socket_response = SocketResponse {
                frame_number: socket_request.frame_number,
                result_type: ResultType::GameOver,
                is_alive: None,
                attacker_health: None,
                exploded_mines: None,
                triggered_defenders: None,
                // defender_damaged: None,
                damaged_buildings: None,
                artifacts_gained_total: None,
                is_sync: false,
                // state: Some(GameStateResponse {
                //     frame_no: socket_request.frame_number,
                //     attacker_user_id: defender_trigger_result_clone
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_user_id,
                //     defender_user_id: defender_trigger_result_clone
                //         .clone()
                //         .unwrap()
                //         .2
                //         .defender_user_id,
                //     attacker: defender_trigger_result_clone.clone().unwrap().2.attacker,
                //     attacker_death_count: defender_trigger_result_clone
                //         .clone()
                //         .unwrap()
                //         .2
                //         .attacker_death_count,
                //     bombs: defender_trigger_result_clone.clone().unwrap().2.bombs,
                //     damage_percentage: defender_trigger_result_clone
                //         .clone()
                //         .unwrap()
                //         .2
                //         .damage_percentage,
                //     artifacts: defender_trigger_result_clone.clone().unwrap().2.artifacts,
                //     defenders: defender_trigger_result_clone.clone().unwrap().2.defenders,
                //     mines: defender_trigger_result_clone.clone().unwrap().2.mines,
                //     buildings: defender_trigger_result_clone.clone().unwrap().2.buildings,
                //     total_hp_buildings: defender_trigger_result_clone
                //         .clone()
                //         .unwrap()
                //         .2
                //         .total_hp_buildings,
                // }),
                is_game_over: true,
                message: Some(String::from("Game over")),
            };

            return Some(Ok(socket_response));
        }
    }
    None
}
