use serde::{Deserialize, Serialize};

use crate::validator::util::Coords;

#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    pub attacker: Coords,
    pub defenders: Vec<Coords>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub message_type: String,
    pub frame: u32,
    pub payload: Payload,
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
