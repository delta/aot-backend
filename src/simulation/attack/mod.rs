use std::collections::HashMap;

use crate::api::attack::util::{get_attacker_types, NewAttacker};
use anyhow::Result;
use diesel::PgConnection;

use self::{attacker::Attacker, emp::Emps};

use super::{
    blocks::BuildingsManager, error::KeyError, robots::RobotsManager, RenderAttacker, Simulator,
};

pub mod attacker;
pub mod emp;

pub struct AttackManager {
    pub attackers: HashMap<i32, Attacker>,
    pub no_of_attackers: i32,
    pub emps: Emps,
}

impl AttackManager {
    pub fn new(conn: &mut PgConnection, new_attackers: &Vec<NewAttacker>) -> Result<Self> {
        let attacker_types = get_attacker_types(conn)?;
        let mut attackers: HashMap<i32, Attacker> = HashMap::new();
        for (id, new_attacker) in new_attackers.iter().enumerate() {
            let attacker_type =
                attacker_types
                    .get(&new_attacker.attacker_type)
                    .ok_or(KeyError {
                        key: new_attacker.attacker_type,
                        hashmap: "attacker_types".to_string(),
                    })?;
            attackers.insert(
                id as i32 + 1,
                Attacker::new(&new_attacker.attacker_path, attacker_type, id as i32 + 1),
            );
        }
        let emps = Emps::new(conn, &attackers)?;
        Ok(AttackManager {
            attackers,
            no_of_attackers: new_attackers.len() as i32,
            emps,
        })
    }

    pub fn update_attackers_position(&mut self, frames_passed: i32) {
        for (_, attacker) in self.attackers.iter_mut() {
            attacker.move_attacker(frames_passed);
        }
    }

    pub fn simulate_attack(
        &mut self,
        frames_passed: i32,
        robots_manager: &mut RobotsManager,
        buildings_manager: &mut BuildingsManager,
    ) -> Result<()> {
        self.update_attackers_position(frames_passed);
        let minute = Simulator::get_minute(frames_passed);
        self.emps.simulate(
            minute,
            robots_manager,
            buildings_manager,
            &mut self.attackers,
        )?;

        Ok(())
    }

    pub fn get_attacker_positions(&mut self) -> Result<HashMap<i32, Vec<RenderAttacker>>> {
        let mut attacker_positions: HashMap<i32, Vec<RenderAttacker>> = HashMap::new();
        for attacker in self.attackers.values_mut() {
            let render_attackers = attacker.post_simulate()?;
            attacker_positions.insert(attacker.id, render_attackers);
        }
        Ok(attacker_positions)
    }
}
