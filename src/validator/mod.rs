use crate::api::attack::util::FrameDetails;
use anyhow::{Result, Ok};
use serde::Serialize;

use self::state::{Coords, State};

pub mod state;

#[derive(Serialize)]
pub struct MineDetails {
    pub id: i32,
    pub pos: Coords,
    pub radius: i32,
    pub damage: i32,
}

#[derive(Serialize)]
pub struct BuildingDetails {
    pub id: i32,
    pub current_hp: i32,
    pub artifacts_obtained: i32,
}

#[derive(Serialize)]
pub struct ValidatorResponse {
    pub frame_no: i32,
    pub attacker_pos: Coords,
    pub mines_triggered: Vec<MineDetails>,
    pub buildings_damaged: Vec<BuildingDetails>,
    pub artifacts_gained: i32,
    pub state: Option<State>,
    pub is_sync: bool,
}

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
