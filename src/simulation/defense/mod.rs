use diesel::PgConnection;

use self::{defender::Defenders, diffuser::Diffusers, mine::Mines};

use super::{attack::AttackManager, blocks::BuildingsManager};
use anyhow::{Ok, Result};

pub mod defender;
pub mod diffuser;
pub mod mine;

pub struct DefenseManager {
    pub defenders: Defenders,
    pub diffusers: Diffusers,
    pub mines: Mines,
}

impl DefenseManager {
    #[allow(dead_code)]
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        let defenders = Defenders::new(conn, map_id)?;
        let diffusers = Diffusers::new(conn, map_id)?;
        let mines = Mines::new(conn, map_id)?;

        Ok(DefenseManager {
            defenders,
            diffusers,
            mines,
        })
    }

    #[allow(dead_code)]
    pub fn simulate(
        &mut self,
        attacker_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
        minute: i32,
    ) -> Result<()> {
        self.mines.simulate(attacker_manager)?;
        self.diffusers
            .simulate(minute, attacker_manager, building_manager)?;
        self.defenders
            .simulate(attacker_manager, building_manager)?;
        Ok(())
    }
}
