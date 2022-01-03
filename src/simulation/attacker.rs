use crate::models::*;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};

pub struct Attacker {
    health: u32,
    path: Vec<AttackerPath>,
}

impl Attacker {
    pub fn update_position(&mut self) {
        if self.health > 0 && self.path.len() > 1 {
            self.path.pop();
        }
    }

    pub fn get_current_position(&self) -> (i32, i32) {
        match self.path.last() {
            Some(attacker_path) => (attacker_path.x_coord, attacker_path.y_coord),
            None => panic!("Empty Attacker Path"),
        }
    }

    pub fn take_damage(&mut self, damage: u32) {
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
        }
    }

    pub fn new(conn: &PgConnection, game_id: i32) -> Self {
        use crate::schema::attacker_path;
        let results = attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .order_by(attacker_path::id.desc())
            .load::<AttackerPath>(conn)
            .expect("Couldn't get attacker path");
        Self {
            health: 100,
            path: results,
        }
    }
}
