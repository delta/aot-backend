use crate::models::*;
use crate::simulation::error::EmptyAttackerPathError;
use anyhow::Result;

pub struct Attacker {
    pub id: i32,
    pub path: Vec<AttackerPath>,
    pub emps_used: usize,
    pub is_alive: bool,
    pub health: i32,
    pub speed: i32,
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

    pub fn get_damage(&mut self, damage: i32) {
        self.health -= damage;
        if self.health <= 0 {
            self.is_alive = false;
            self.health = 0;
        }
    }

    pub fn new(path: &[NewAttackerPath], attacker_type: &AttackerType, id: i32) -> Self {
        let emps_used = path.iter().filter(|p| p.is_emp).count();
        let mut attacker_path: Vec<AttackerPath> = path
            .iter()
            .enumerate()
            .map(|(id, path)| AttackerPath {
                id: id + 1,
                x_coord: path.x_coord,
                y_coord: path.y_coord,
                is_emp: path.is_emp,
                emp_type: path.emp_type,
                emp_time: path.emp_time,
            })
            .collect();
        attacker_path.reverse();
        Self {
            id,
            is_alive: true,
            path: attacker_path,
            emps_used,
            health: attacker_type.max_health,
            speed: attacker_type.speed,
        }
    }
}
