use crate::api::socket::util::{ActionType, ResultType, SocketRequest, SocketResponse};
use anyhow::{Ok, Result};

pub mod error;
pub mod state;
pub mod util;

pub fn game_handler(game_id: i32, socket_request: &SocketRequest) -> Result<SocketResponse> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    if socket_request.action_type == ActionType::PLACE_ATTACKER {
        // place_attacker
    } else if socket_request.action_type == ActionType::MOVE_ATTACKER {
        // move_attacker
    } else if socket_request.action_type == ActionType::PLACE_BOMBS {
        // place_bombs
    } else if socket_request.action_type == ActionType::IDLE {
        // idle (waiting for user to choose next attacker)
    } else if socket_request.action_type == ActionType::TERMINATE {
        // terminate
    }

    let socket_response = SocketResponse {
        frame_number: socket_request.frame_number,
        result_type: ResultType::GAME_OVER,
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
