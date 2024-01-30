use serde::{Deserialize, Serialize};

use crate::validator::util::Coords;

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketRequest {
    pub frame_number: u32,
    pub action_type: ActionType,
    pub attacker_id: Option<u32>,
    pub bomb_id: Option<u32>,
    pub start_position: Option<Coords>,
    pub attacker_path: Vec<Coords>,
    pub bomb_positions: Vec<Coords>,
    pub is_game_over: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct SocketResponse {
    pub frame_number: u32,
    pub result_type: ResultType,
    pub is_alive: Option<bool>,
    pub attacker_health: Option<i32>,
    pub exploded_mines: Vec<MineResponse>,
    pub triggered_defenders: Vec<Coords>,
    pub defender_damaged: Option<DefenderResponse>,
    pub damaged_buildings: Vec<BuildingResponse>,
    pub artifacts_gained: Vec<ArtifactsResponse>,
    pub is_sync: bool,
    pub state: Option<GameStateResponse>,
    pub is_game_over: bool,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    PLACE_ATTACKER,
    MOVE_ATTACKER,
    PLACE_BOMBS,
    IDLE,
    TERMINATE,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    MINES_EXPLODED,
    DEFENDERS_DAMAGED,
    DEFENDERS_TRIGGERED,
    BUILDINGS_DAMAGED,
    RESYNC,
    GAME_OVER,
}

#[derive(Serialize, Deserialize)]
pub struct MineResponse {
    pub id: u32,
    pub position: Coords,
    pub damage: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DefenderResponse {
    pub id: u32,
    pub position: Coords,
    pub damage: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BuildingResponse {
    pub id: u32,
    pub position: Coords,
    pub hp: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ArtifactsResponse {
    pub buildingId: u32,
    pub amount: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateResponse {}

#[derive(Debug, Default)]
pub struct Attacker {
    pub x: i32,
    pub y: i32,
    pub health: i32,
    pub direction: String,
    pub speed: i32,
}

#[derive(Debug, Default)]
pub struct Base {
    pub id: i32,
}

pub struct MyWebSocket {
    pub attacker: Attacker,
    pub base: Base,
}
