mod util;

use crate::api::validator::util::{ActionType, Attacker, Base, SocketRequest, MyWebSocket};
use actix_web::{web, Error, HttpRequest, HttpResponse};
use actix_web_actors::ws;
use serde_json;
use actix::prelude::*;

impl Actor for MyWebSocket {
    type Context = ws::WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        println!("Websocket started");
        ctx.text("Websocket started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        println!("Websocket stopped");
        ctx.text("Websocket stopped");
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWebSocket {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => {
                // Parse the received JSON message
                println!("Received JSON message: {}", text);
                if let Ok(request) = serde_json::from_str::<SocketRequest>(&text) {
                    
                    if request.action_type == ActionType::PLACE_ATTACKER {
                        println!("Placing attacker");
                    } else if request.action_type == ActionType::IDLE {
                        println!("Idle");
                    } else {
                        println!("Invalid JSON input");
                        ctx.text("Invalid JSON input");
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

impl MyWebSocket {
    fn update(&mut self, request: SocketRequest, ctx: &mut ws::WebsocketContext<Self>) {
        // Update the attacker's position
        println!("Updating attacker position");
        //if cannot update attackerposition
        ctx.text("Error updating attacker position");
    }
}

pub async fn ws_validator_handler(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {

    // passing initial states to websocket
    let attacker = Attacker {
        x: 0,
        y: 0,
        health: 100,
        direction: "right".to_string(),
        speed: 5,
    };

    let base = Base { id: 0 };

    ws::start(MyWebSocket { attacker, base }, &req, stream)
}
