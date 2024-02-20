use crate::api::attack::socket::{ResultType, SocketResponse};
// use crate::{constants::ROAD_ID, validator::state::State};
use crate::validator::state::State;
use crate::{api::attack::socket::DefenderResponse, simulation::blocks::Coords};
use serde::{Deserialize, Serialize};

// Structs present in the state
// #[derive(Debug, Serialize, Deserialize, Clone, Hash, Eq, PartialEq, Copy)]
// pub struct Coords {
//     pub x: i32,
//     pub y: i32,
// }

#[derive(Serialize, Clone, Copy, Deserialize)]
pub struct Bomb {
    pub id: i32,
    pub blast_radius: i32,
    pub damage: i32,
    pub pos: Coords,
    pub is_dropped: bool,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct Attacker {
    pub id: i32,
    pub attacker_pos: Coords,
    pub attacker_health: i32,
    pub attacker_speed: i32,
    pub path_in_current_frame: Vec<Coords>,
    pub bombs: Vec<Bomb>,
    pub trigger_defender: bool,
    pub bomb_count: i32,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct IsTriggered {
    pub is_triggered: bool,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DefenderDetails {
    pub id: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub defender_pos: Coords,
    pub is_alive: bool,
    pub damage_dealt: bool,
    pub target_id: Option<f32>,
    pub path_in_current_frame: Vec<Coords>,
}

// Structs for sending response
#[derive(Serialize, Deserialize, Clone)]
pub struct MineDetails {
    pub id: i32,
    pub position: Coords,
    pub radius: i32,
    pub damage: i32,
}

#[derive(Serialize, Clone, Deserialize)]
pub struct BombType {
    pub id: i32,
    pub radius: i32,
    pub damage: i32,
    pub total_count: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct BuildingDetails {
    pub id: i32,
    pub current_hp: i32,
    pub total_hp: i32,
    pub artifacts_obtained: i32,
    pub tile: Coords,
    pub width: i32,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct InValidation {
    pub message: String,
    pub is_invalidated: bool,
}

#[derive(Serialize, Clone, Copy)]
pub struct SourceDest {
    pub source: Coords,
    pub dest: Coords,
}
#[derive(Serialize, Clone)]

pub struct DefenderReturnType {
    pub attacker_health: i32,
    pub defender_response: Vec<DefenderResponse>,
    pub state: State,
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

pub fn send_terminate_game_message(frame_number: i32, message: String) -> SocketResponse {
    let socket_response = SocketResponse {
        frame_number,
        result_type: ResultType::GameOver,
        is_alive: None,
        attacker_health: None,
        exploded_mines: None,
        defender_damaged: None,
        damaged_buildings: None,
        artifacts_gained_total: None,
        is_sync: false,
        is_game_over: true,
        message: Some(String::from(message)),
    };

    socket_response
}
