use crate::models::*;
use crate::simulation::error::EmptyAttackerPathError;
use anyhow::Result;

pub struct Attacker {
    pub is_alive: bool,
    pub path: Vec<AttackerPath>,
    pub emps_used: usize,
}

impl Attacker {
    pub fn update_position(&mut self) {
        if self.is_alive && self.path.len() > 1 {
            self.path.pop();
        }
    }

    pub fn is_planted(&self, path_id: usize) -> Result<bool> {
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

    pub fn new(path: Vec<AttackerPath>) -> Self {
        let emps_used = path.iter().filter(|p| p.is_emp).count();
        Self {
            is_alive: true,
            path,
            emps_used,
        }
    }
}
