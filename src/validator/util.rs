use serde::{Deserialize, Serialize};
use crate::validator::state::State;

// Structs present in the state
#[derive(Serialize, Deserialize)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize)]
pub struct Bomb {
    pub id: i32,
    pub blast_radius: i32,
    pub damage: i32,
}

#[derive(Serialize)]
pub struct Attacker {
    pub id: i32,
    pub attacker_pos: Coords,
    pub attacker_health: i32,
    pub attacker_speed: i32,
    pub path_in_current_frame: Vec<Coords>,
    pub bomb: Bomb,
}

#[derive(Serialize)]
pub struct Defender {
    pub id: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub defender_pos: Coords,
    pub is_alive: bool,
    pub damage_dealt: bool,
    pub target_id: Option<i32>,
    pub path_in_current_frame: Vec<Coords>,
}

// Structs for sending response
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
