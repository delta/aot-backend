use serde::{Deserialize, Serialize};

use crate::validator::util::Coords;

#[derive(Serialize, Deserialize, Debug)]
pub struct SocketRequest {
    pub frame_number: i32,
    pub action_type: ActionType,
    pub attacker_id: Option<i32>,
    pub bomb_id: Option<i32>,
    pub start_position: Option<Coords>,
    pub attacker_path: Vec<Coords>,
    pub bomb_positions: Vec<Coords>,
    pub is_game_over: Option<bool>,
}

#[derive(Serialize, Deserialize)]
pub struct SocketResponse {
    pub frame_number: i32,
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
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ActionType {
    PlaceAttacker,
    MoveAttacker,
    PlaceBombs,
    Idle,
    Terminate,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ResultType {
    MinesExploded,
    DefendersDamaged,
    DefendersTriggered,
    BuildingsDamaged,
    Resync,
    GameOver,
}

#[derive(Serialize, Deserialize)]
pub struct MineResponse {
    pub id: i32,
    pub position: Coords,
    pub damage: i32,
    pub radius: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DefenderResponse {
    pub id: i32,
    pub position: Coords,
    pub damage: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BuildingResponse {
    pub id: i32,
    pub position: Coords,
    pub hp: i32,
}

#[derive(Serialize, Deserialize)]
pub struct ArtifactsResponse {
    pub building_id: i32,
    pub amount: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GameStateResponse {}
