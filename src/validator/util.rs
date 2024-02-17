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

// pub fn is_road(pos: &Coords) -> bool {
//     // create user_map_space with id and block_type_id stored with the map_id for base (also redis?)
//     let block_type_id = user_map_space[pos.x][pos.y].block_type_id;
//     // have a global block_types (same as BlockType table) (redis)
//     block_types[block_type_id][BUILDING_TYPE_INDEX] == ROAD_ID
// }
