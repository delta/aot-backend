use crate::models::*;
use crate::simulation::attack::AttackManager;
use anyhow::{Ok, Result};
use diesel::dsl::not;
use diesel::prelude::*;
use diesel::PgConnection;
pub struct Diffuser {
    pub id: i32,
    pub diffuser_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub target_emp_id: Option<i32>,
}

#[allow(dead_code)]
pub struct Diffusers(Vec<Diffuser>);

impl Diffusers {
    #[allow(dead_code)]
    pub fn new(conn: &mut PgConnection) -> Result<Self> {
        use crate::schema::{building_type, diffuser_type, map_spaces};

        let joined_table = map_spaces::table
            .inner_join(building_type::table)
            .inner_join(diffuser_type::table.on(not(building_type::diffuser_type.is_null())));

        let mines: Vec<Diffuser> = joined_table
            .load::<(MapSpaces, BuildingType, DiffuserType)>(conn)?
            .into_iter()
            .map(|(map_space, _, diffuser_type)| Diffuser {
                id: map_space.id,
                diffuser_type: diffuser_type.id,
                radius: diffuser_type.radius,
                x_position: map_space.x_coordinate,
                y_position: map_space.y_coordinate,
                is_alive: true,
                target_emp_id: None,
                speed: diffuser_type.speed,
            })
            .collect();

        Ok(Diffusers(mines))
    }

    #[allow(dead_code)]
    pub fn simulate(&mut self, minute: i32, attack_manager: &mut AttackManager) -> Result<()> {
        //get list of active emps within radius
        let Diffusers(diffusers) = self;
        // let active_emps = emps_manager.get_active_emps(minute as usize);
        for (_, diffuser) in diffusers.iter_mut().enumerate() {
            if diffuser.is_alive
                && attack_manager.emps.effect_from_diffuser(
                    diffuser.x_position,
                    diffuser.y_position,
                    diffuser.radius,
                    diffuser.speed,
                    minute as usize,
                )
            {
                diffuser.is_alive = false;
            }
        }
        Ok(())
    }
}
