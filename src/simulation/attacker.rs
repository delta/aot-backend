use crate::error::DieselError;
use crate::models::*;
use crate::simulation::error::EmptyAttackerPathError;
use crate::util::function;
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};

pub struct Attacker {
    pub is_alive: bool,
    pub path: Vec<AttackerPath>,
}

impl Attacker {
    pub fn update_position(&mut self) {
        if self.is_alive && self.path.len() > 1 {
            self.path.pop();
        }
    }

    pub fn is_planted(&self, path_id: i32) -> Result<bool> {
        match self.path.last() {
            Some(attacker_path) => Ok(attacker_path.id >= path_id),
            None => Err(EmptyAttackerPathError.into()),
        }
    }

    pub fn get_current_position(&self) -> Result<(i32, i32)> {
        match self.path.last() {
            Some(attacker_path) => Ok((attacker_path.x_coord, attacker_path.y_coord)),
            None => Err(EmptyAttackerPathError.into()),
        }
    }

    pub fn kill(&mut self) {
        self.is_alive = false;
    }

    pub fn new(conn: &PgConnection, game_id: i32) -> Result<Self> {
        use crate::schema::attacker_path;
        let path = attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .order_by(attacker_path::id.desc())
            .load::<AttackerPath>(conn)
            .map_err(|err| DieselError {
                table: "attacker_path",
                function: function!(),
                error: err,
            })?;
        Ok(Self {
            is_alive: true,
            path,
        })
    }
}
