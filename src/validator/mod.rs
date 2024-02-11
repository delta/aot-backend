use std::collections::{HashMap, HashSet};

use crate::{
    api::attack::socket::{ActionType, ResultType, SocketRequest, SocketResponse},
    simulation::blocks::{Coords, SourceDest},
};
use anyhow::{Ok, Result};
use crate::validator::state::{};

use self::{state::State, util::BombType};

pub mod error;
pub mod state;
pub mod util;

pub fn game_handler(
    socket_request: SocketRequest, 
    _game_state: &mut State, 
    _shortest_path: &HashMap<SourceDest, Coords>,
    _roads: &HashSet<(i32, i32)>,
    _bomb_types: &Vec<BombType>,
) -> Option<Result<SocketResponse>> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    match socket_request.action_type {
        ActionType::PlaceAttacker => return None,
        ActionType::MoveAttacker => {
            // move_attacker
        }
        ActionType::PlaceBombs => {
            // place_bombs
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
