use crate::api::socket::util::{ActionType, ResultType, SocketRequest, SocketResponse};
use anyhow::{Ok, Result};

pub mod error;
pub mod state;
pub mod util;

pub fn game_handler(_game_id: i32, socket_request: &SocketRequest) -> Result<SocketResponse> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    match socket_request.action_type {
        ActionType::PlaceAttacker => {
            // place_attacker
        }
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
            // terminate
        }
    }

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

    Ok(socket_response)
}
