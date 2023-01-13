use self::{defender::Defenders, diffuser::Diffusers, mine::Mines};
use anyhow::{Ok, Result};
use diesel::PgConnection;

use crate::simulation::attack::AttackManager;
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
        let defenders = Defenders::new(conn)?;
        let diffusers = Diffusers::new(conn,map_id)?;
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
        attack_manager: &mut AttackManager,
        conn: &mut PgConnection,
        map_id: i32,
        minute: i32,
    ) -> Result<()> {
        self.mines.simulate(attack_manager)?;
        self.diffusers.simulate(minute, attack_manager)?;
        self.defenders.simulate(attack_manager, conn, map_id)?;
        Ok(())
    }
}
