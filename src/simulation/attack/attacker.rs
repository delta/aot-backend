use crate::models::*;
use crate::simulation::error::EmptyAttackerPathError;
use crate::simulation::{RenderAttacker, Simulator};
use anyhow::Result;

#[derive(Debug)]
pub struct AttackPathStats {
    pub attacker_path: AttackerPath,
    pub health: i32,
    pub is_alive: bool,
}

pub struct Attacker {
    pub id: i32,
    pub path: Vec<AttackerPath>,
    pub emps_used: usize,
    pub is_alive: bool,
    pub health: i32,
    pub speed: i32,
    pub attacker_type: i32,
    pub path_in_current_frame: Vec<AttackPathStats>,
}

impl Attacker {
    pub fn update_position(&mut self, frames_passed: i32) {
        self.path_in_current_frame.clear();
        if !Simulator::attacker_allowed(frames_passed) {
            self.path_in_current_frame.push(AttackPathStats {
                attacker_path: self.path[self.path.len() - 1],
                health: self.health,
                is_alive: self.is_alive,
            });
            return;
        }
        if self.is_alive && self.path.len() > 1 {
            if self.path.len() > self.speed as usize {
                self.path_in_current_frame = self
                    .path
                    .split_off(self.path.len() - self.speed as usize)
                    .into_iter()
                    .map(|attacker_path| AttackPathStats {
                        attacker_path,
                        health: self.health,
                        is_alive: self.is_alive,
                    })
                    .collect();
            } else {
                self.path_in_current_frame = self
                    .path
                    .split_off(1)
                    .into_iter()
                    .map(|attacker_path| AttackPathStats {
                        attacker_path,
                        health: self.health,
                        is_alive: self.is_alive,
                    })
                    .collect();
            }
        }
        self.path_in_current_frame.insert(
            0,
            AttackPathStats {
                attacker_path: self.path[self.path.len() - 1],
                health: self.health,
                is_alive: self.is_alive,
            },
        )
    }

    pub fn is_planted(&self, path_id: usize) -> Result<bool> {
        match self.path.last() {
            Some(attacker_path) => Ok(attacker_path.id >= path_id),
            None => Err(EmptyAttackerPathError.into()),
        }
    }

    pub fn get_current_position(&self) -> Result<(i32, i32)> {
        match self.path_in_current_frame.last() {
            Some(attacker_path_stats) => Ok((
                attacker_path_stats.attacker_path.x_coord,
                attacker_path_stats.attacker_path.y_coord,
            )),
            None => Err(EmptyAttackerPathError.into()),
        }
    }

    pub fn get_damage(&mut self, damage: i32, current_attacker_pos: usize) {
        for position in 0..=current_attacker_pos {
            self.path_in_current_frame[position].health -= damage;
            if self.path_in_current_frame[position].health <= 0 {
                self.path_in_current_frame[position].is_alive = false;
                self.path_in_current_frame[position].health = 0
            }
        }
    }

    pub fn post_simulate(&mut self) -> Result<Vec<RenderAttacker>> {
        let mut render_attacker: Vec<RenderAttacker> = Vec::new();
        if self.path_in_current_frame.is_empty() {
            self.path_in_current_frame.push(AttackPathStats {
                attacker_path: self.path[self.path.len() - 1],
                health: self.health,
                is_alive: self.is_alive,
            })
        }
        while self.path_in_current_frame.len() > 1
            && self.path_in_current_frame.last().unwrap().is_alive
        {
            self.path_in_current_frame.pop();
            let attacker_path_stats = self.path_in_current_frame.last().unwrap();
            render_attacker.push(RenderAttacker {
                attacker_id: self.id,
                health: attacker_path_stats.health,
                x_position: attacker_path_stats.attacker_path.x_coord,
                y_position: attacker_path_stats.attacker_path.y_coord,
                is_alive: attacker_path_stats.is_alive,
                attacker_type: self.attacker_type,
                emp_id: if attacker_path_stats.attacker_path.is_emp {
                    attacker_path_stats.attacker_path.id
                } else {
                    0
                },
            });
        }

        while render_attacker.len() < self.speed as usize {
            let attacker_path_stats = self.path_in_current_frame.last().unwrap();
            render_attacker.push(RenderAttacker {
                attacker_id: self.id,
                health: attacker_path_stats.health,
                x_position: attacker_path_stats.attacker_path.x_coord,
                y_position: attacker_path_stats.attacker_path.y_coord,
                is_alive: attacker_path_stats.is_alive,
                attacker_type: self.attacker_type,
                emp_id: if attacker_path_stats.attacker_path.is_emp {
                    attacker_path_stats.attacker_path.id
                } else {
                    0
                },
            });
        }

        let destination = self.path_in_current_frame.last().unwrap();
        self.health = destination.health;
        self.is_alive = destination.is_alive;

        self.path_in_current_frame.remove(0);

        while !self.path_in_current_frame.is_empty() {
            let attacker_path_stats = self.path_in_current_frame.remove(0);
            self.path.push(attacker_path_stats.attacker_path)
        }
        Ok(render_attacker)
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
        let path_in_current_frame = Vec::new();
        Self {
            id,
            is_alive: true,
            path: attacker_path,
            emps_used,
            health: attacker_type.max_health,
            speed: attacker_type.speed,
            attacker_type: attacker_type.id,
            path_in_current_frame,
        }
    }
}
