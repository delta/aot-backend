use serde::{Deserialize, Serialize};

use crate::validator::util::Coords;

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SocketRequest {
    pub frame_number: u32,
    pub action_type: ActionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attacker_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bomb_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attacker_start_coords: Option<Coords>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attacker_path: Option<Vec<Coords>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bomb_coords: Option<Vec<Coords>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_game_over: Option<bool>,
}


#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub enum ActionType {
    PLACE_ATTACKER,
    MOVE_ATTACKER,
    PLACE_BOMBS,
    IDLE,
    #[default]
    TERMINATE,
}


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
