use std::collections::{HashMap, HashSet};

use crate::{
    api::attack::socket::{ActionType, ResultType, SocketRequest, SocketResponse}, models::AttackerType, simulation::{attack::attacker, blocks::{Coords, SourceDest}}
};
use anyhow::{Ok, Result};
use tungstenite::protocol::frame;

use self::{state::State, util::{Attacker, BombType}};

pub mod error;
pub mod state;
pub mod util;

// use crate::validator::state::State::place_attacker;

// use crate::validator::state::State::place_attacker;
// get_attacker_types
use crate::api::attack::util::get_attacker_types;

pub fn game_handler(
    attacker_type: &HashMap<i32,AttackerType>,
    socket_request: SocketRequest, 
    _game_state: &mut State, 
    _shortest_path: &HashMap<SourceDest,Coords>,
    _roads: &HashSet<(i32, i32)>,
    _bomb_types: &Vec<BombType>,
) -> Option<Result<SocketResponse>> {
    // redis for storing mapping
    // fetch validator instance (has redis)
    // iterate through input data and call appropriate instance functions
    // form response and send

    match socket_request.action_type {
        ActionType::PlaceAttacker => {
            dotenv::dotenv().ok();
            
         

            if socket_request.frame_number == 1 {
                let bomb_max_count = std::env::var("BOMBS_MAX_COUNT")
                .unwrap_or("0".to_string())
                .parse::<i32>()
                .unwrap_or(0);
            for bomb_type in _bomb_types {

                if let Some(bomb_id) = socket_request.bomb_id {
                    if bomb_type.id == bomb_id {
                        _game_state.set_bombs(bomb_type.clone(), bomb_max_count);
                    }
                }
           
            }
            }
            

            if let Some(attacker_id) = socket_request.attacker_id {
                let attacker: AttackerType = attacker_type.get(&attacker_id).unwrap().clone();
                _game_state.place_attacker(Attacker{
                    id: attacker.id,
                    path_in_current_frame: Vec::new(),
                    attacker_pos: socket_request.start_position.unwrap(),
                    attacker_health: attacker.max_health,
                    attacker_speed: attacker.speed,
                    bombs: Vec::new(),
                });
            }
           
            _game_state.update_frame_number(socket_request.frame_number.clone());
               

            
        },
        ActionType::MoveAttacker => {
            // move_attacker
            // State::new()
            if let Some(attacker_id) = socket_request.attacker_id {
                let attacker: AttackerType = attacker_type.get(&attacker_id).unwrap().clone();
                let attacker_delta: Vec<Coords> = socket_request.attacker_path;
                
                

                _game_state.attacker_movement(socket_request.frame_number.clone(),attacker_delta.clone(), Attacker{
                    id: attacker.id,
                    path_in_current_frame: Vec::new(),
                    attacker_pos: socket_request.start_position.unwrap(),
                    attacker_health: attacker.max_health,
                    attacker_speed: attacker.speed,
                    bombs: Vec::new(),
                });

                _game_state.defender_movement(socket_request.frame_number.clone(), attacker_delta.clone(), _shortest_path);
                _game_state.update_frame_number(socket_request.frame_number.clone());

            }
         
  
        }
        ActionType::PlaceBombs => {
            // place_bombs
            let attacker_delta: Vec<Coords> = socket_request.attacker_path;


            _game_state.place_bombs(attacker_delta,socket_request.bomb_position);
           
            
        }
        ActionType::Idle => {
            // idle (waiting for user to choose next attacker)
        }
        ActionType::Terminate => {
            let socket_response = SocketResponse {
                frame_number: socket_request.frame_number,
                result_type: ResultType::GameOver,
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

            return Some(Ok(socket_response));
        }
    }

    Some(Err(error::FrameError { frame_no: 0 }.into()))
}
