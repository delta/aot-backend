use std::collections::HashMap;

use crate::api::attack::util::{get_attacker_types, NewAttacker};
use anyhow::Result;
use diesel::PgConnection;

use self::{attacker::Attacker, emp::Emps};

use super::{blocks::BuildingsManager, error::KeyError, robots::RobotsManager, Simulator};

pub mod attacker;
mod emp;

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

    pub fn update_attackers_position(&mut self) {
        for (_, attacker) in self.attackers.iter_mut() {
            attacker.update_position()
        }
    }

    pub fn simulate_attack(
        &mut self,
        frames_passed: i32,
        robots_manager: &mut RobotsManager,
        buildings_manager: &mut BuildingsManager,
    ) -> Result<()> {
        if Simulator::attacker_allowed(frames_passed) {
            self.update_attackers_position();
        }
        let minute = Simulator::get_minute(frames_passed);
        self.emps.simulate(
            minute,
            robots_manager,
            buildings_manager,
            &mut self.attackers,
        )?;

        Ok(())
    }
}
