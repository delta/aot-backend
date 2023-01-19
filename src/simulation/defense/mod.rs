use diesel::PgConnection;

use self::defender::Defenders;

use super::{attack::AttackManager, blocks::BuildingsManager};
use anyhow::{Ok, Result};

pub mod defender;
pub mod diffuser;
pub mod mine;

pub struct DefenseManager {
    pub defenders: Defenders,
}

impl DefenseManager {
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        let defenders = Defenders::new(conn, map_id)?;
        Ok(Self { defenders })
    }

    pub fn simulate(
        &mut self,
        attacker_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
    ) -> Result<()> {
        self.defenders
            .simulate(attacker_manager, building_manager)?;
        Ok(())
    }
}
