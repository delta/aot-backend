use crate::api::attack::util::FrameDetails;
use anyhow::{Result, Ok};

use self::util::{Coords, ValidatorResponse};

pub mod state;
pub mod util;
pub mod error;

pub fn game_handler(game_id: i32, frame: &FrameDetails) -> Result<ValidatorResponse> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    let validator_response = ValidatorResponse {
        frame_no: frame.frame_no,
        attacker_pos: Coords { x: 0, y: 0 },
        mines_triggered: Vec::new(),
        buildings_damaged: Vec::new(),
        artifacts_gained: 0,
        state: None,
        is_sync: false, // might be redundant
    };
    Ok(validator_response)
}
