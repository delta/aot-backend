use crate::models::*;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};

pub struct Attacker {
    is_alive: bool,
    path: Vec<AttackerPath>,
}

impl Attacker {
    pub fn update_position(&mut self) {
        if self.is_alive && self.path.len() > 1 {
            self.path.pop();
        }
    }

    pub fn is_planted(&self, path_id: i32) -> bool {
        self.path.last().unwrap().id >= path_id
    }

    pub fn get_current_position(&self) -> (i32, i32) {
        match self.path.last() {
            Some(attacker_path) => (attacker_path.x_coord, attacker_path.y_coord),
            None => panic!("Empty Attacker Path"),
        }
    }

    pub fn kill(&mut self) {
        self.is_alive = false;
    }

    pub fn new(conn: &PgConnection, game_id: i32) -> Self {
        use crate::schema::attacker_path;
        let path = attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .order_by(attacker_path::id.desc())
            .load::<AttackerPath>(conn)
            .expect("Couldn't get attacker path");
        Self {
            is_alive: true,
            path,
        }
    }
}
