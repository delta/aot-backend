pub mod util;

use crate::{
    api::socket::util::{ResultType, SocketRequest, SocketResponse},
    validator::game_handler,
};
use actix::prelude::*;
use actix_web_actors::ws;
use serde_json;

pub struct Socket {
    pub game_id: i32,
}

impl Actor for Socket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Websocket started");
        ctx.text("Websocket started");

        let response = SocketResponse {
            frame_number: 0,
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
        if let Ok(json_response) = serde_json::to_string(&response) {
            ctx.text(json_response);
        } else {
            println!("Error serializing JSON");
            ctx.text("Error serializing JSON");
        }
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Websocket stopped");
        ctx.text("Websocket stopped");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for Socket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                println!("Received JSON message: {}", text);
                if let Ok(request) = serde_json::from_str::<SocketRequest>(&text) {
                    println!("Parsed JSON message: {:?}", request);
                    let response = game_handler(0, &request);
                    if response.is_ok() {
                        let response = response.unwrap();
                        if let Ok(json_response) = serde_json::to_string(&response) {
                            ctx.text(json_response);
                        } else {
                            println!("Error serializing JSON");
                            ctx.text("Error serializing JSON");
                        }
                    } else {
                        println!("Error handling game");
                        ctx.text("Error handling game");
                    }
                } else {
                    println!("Error parsing JSON");
                    ctx.text("Error parsing JSON");
                }
            }
            _ => (),
        }
    }
}
