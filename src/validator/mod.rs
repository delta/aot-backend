use std::collections::{HashMap, HashSet};

use crate::{
    api::attack::socket::{ActionType, ResultType, SocketRequest, SocketResponse}, models::AttackerType, simulation::{attack::attacker, blocks::{Coords, SourceDest}}
};
use anyhow::{Ok, Result};

use self::{state::State, util::{Attacker, BombType}};

pub mod error;
pub mod state;
pub mod util;

// use crate::validator::state::State::place_attacker;

// use crate::validator::state::State::place_attacker;
// get_attacker_types
use crate::api::attack::util::get_attacker_types;

pub fn game_handler(
    attacker_type: &HashMap<i32,AttackerType>,
    socket_request: SocketRequest, 
    _game_state: &mut State, 
    _shortest_path: &HashMap<SourceDest,Coords>,
    _roads: &HashSet<(i32, i32)>,
    _bomb_types: &Vec<BombType>,
) -> Option<Result<SocketResponse>> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    match socket_request.action_type {
        ActionType::PlaceAttacker => {

            if let Some(attacker_id) = socket_request.attacker_id {
                let attacker: AttackerType = attacker_type.get(&attacker_id).unwrap().clone();
                _game_state.place_attacker(Attacker{
                    id: attacker.id,
                    path_in_current_frame: Vec::new(),
                    attacker_pos: socket_request.start_position.unwrap(),
                    attacker_health: attacker.max_health,
                    attacker_speed: attacker.speed,
                    bombs: Vec::new(),
                });
            }
        },
        ActionType::MoveAttacker => {
            // move_attacker
            // State::new()
            let _attacker : Attacker = Attacker {
                id: 1,
                path_in_current_frame: Vec::new(),
                attacker_pos: todo!(),
                attacker_health: todo!(),
                attacker_speed: todo!(),
                bombs: todo!(),
            };
            let attacker_delta: Vec<Coords> = vec![Coords { x: 1, y: 1 }];
            _game_state.attacker_movement(1,attacker_delta, _attacker);
            _game_state.defender_movement(1, attacker_delta, _shortest_path);
        }
        ActionType::PlaceBombs => {
            // place_bombs
            let _attacker : Attacker = Attacker {
                id: 1,
                path_in_current_frame: Vec::new(),
                attacker_pos: todo!(),
                attacker_health: todo!(),
                attacker_speed: todo!(),
                bombs: todo!(),
            };
            _game_state.place_bombs(_attacker);
        }
        ActionType::Idle => {
            // idle (waiting for user to choose next attacker)
        }
        ActionType::Terminate => {
            let socket_response = SocketResponse {
                frame_number: socket_request.frame_number,
                result_type: ResultType::GameOver,
                is_alive: None,
                attacker_health: None,
                exploded_mines: Vec::new(),
                triggered_defenders: Vec::new(),
                defender_damaged: None,
                damaged_buildings: Vec::new(),
                artifacts_gained: Vec::new(),
                is_sync: false,
                state: None,
                is_game_over: true,
                message: None,
            };

            return Some(Ok(socket_response));
        }
    }

    Some(Err(error::FrameError { frame_no: 0 }.into()))
}
