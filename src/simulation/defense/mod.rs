use diesel::PgConnection;

use self::{defender::Defenders, mine::Mines};

use super::{attack::AttackManager, blocks::BuildingsManager, Simulator};
use anyhow::{Ok, Result};

pub mod defender;
pub mod mine;

pub struct DefenseManager {
    pub defenders: Defenders,
    pub mines: Mines,
}

impl DefenseManager {
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        let defenders = Defenders::new(conn, map_id)?;
        let mines = Mines::new(conn, map_id)?;

        Ok(DefenseManager { defenders, mines })
    }

    pub fn simulate(
        &mut self,
        attacker_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
        frames_passed: i32,
    ) -> Result<()> {
        if !Simulator::attacker_allowed(frames_passed) {
            return Ok(());
        }
        self.mines.simulate(attacker_manager)?;
        self.defenders
            .simulate(attacker_manager, building_manager)?;
        Ok(())
    }
}
