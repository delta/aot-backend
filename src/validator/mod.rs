use std::collections::{HashMap, HashSet};

use crate::{
    api::attack::socket::{ActionType, ResultType, SocketRequest, SocketResponse},
    simulation::{attack::attacker, blocks::{Coords, SourceDest}},
};
use anyhow::{Ok, Result};

use self::{state::State, util::{Attacker, BombType, Coordinates}};

pub mod error;
pub mod state;
pub mod util;

// use crate::validator::state::State::place_attacker;

// use crate::validator::state::State::place_attacker;


pub fn game_handler(
    socket_request: SocketRequest, 
    game_state: &mut State, 
    shortest_path: &HashMap<(Coordinates,Coordinates), Coordinates>,
    _roads: &HashSet<(i32, i32)>,
    _bomb_types: &Vec<BombType>,
) -> Option<Result<SocketResponse>> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    match socket_request.action_type {
        ActionType::PlaceAttacker => {
            let attacker = Attacker {
                id: 1,
                path_in_current_frame: Vec::new(),
                attacker_pos: todo!(),
                attacker_health: todo!(),
                attacker_speed: todo!(),
                bombs: todo!(),
            };
            game_state.place_attacker(attacker);
        },
        ActionType::MoveAttacker => {
            // move_attacker
            // State::new()
            let attacker : Attacker = Attacker {
                id: 1,
                path_in_current_frame: Vec::new(),
                attacker_pos: todo!(),
                attacker_health: todo!(),
                attacker_speed: todo!(),
                bombs: todo!(),
            };
            let attacker_delta: Vec<Coordinates> = vec![Coordinates { x: 1, y: 1 }];
            game_state.attacker_movement(1,attacker_delta, attacker);
            game_state.defender_movement(1, attacker_delta, shortest_path);
        }
        ActionType::PlaceBombs => {
            // place_bombs
            let attacker : Attacker = Attacker {
                id: 1,
                path_in_current_frame: Vec::new(),
                attacker_pos: todo!(),
                attacker_health: todo!(),
                attacker_speed: todo!(),
                bombs: todo!(),
            };
            game_state.place_bombs(attacker);
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
