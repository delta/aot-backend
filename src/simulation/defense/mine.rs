use crate::models::*;
use crate::simulation::attack::AttackManager;
use anyhow::{Ok, Result};
use diesel::prelude::*;
use diesel::PgConnection;

pub struct Mine {
    pub id: i32,
    pub mine_type: i32,
    pub damage: i32,
    pub radius: i32,
    pub is_activated: bool,
    pub x_position: i32,
    pub y_position: i32,
}

pub struct Mines(Vec<Mine>);

impl Mines {
    #[allow(dead_code)]
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{building_type, map_spaces, mine_type};

        let joined_table = map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .inner_join(building_type::table.inner_join(
                mine_type::table.on(building_type::building_category.eq(BuildingCategory::Mine)),
            ));

        let mut mine_id = 0;
        let mines: Vec<Mine> = joined_table
            .load::<(MapSpaces, (BuildingType, MineType))>(conn)?
            .into_iter()
            .map(|(map_space, (_, mine_type))| {
                mine_id += 1;
                Mine {
                    id: mine_id,
                    mine_type: mine_type.id,
                    damage: mine_type.damage,
                    radius: mine_type.radius,
                    is_activated: true,
                    x_position: map_space.x_coordinate,
                    y_position: map_space.y_coordinate,
                }
            })
            .collect();

        Ok(Mines(mines))
    }

    #[allow(dead_code)]
    pub fn simulate(&mut self, attack_manager: &mut AttackManager) -> Result<()> {
        //get pos of attckers
        let Mines(mines) = self;
        let attackers = &mut attack_manager.attackers;

        for mine in mines.iter_mut() {
            let mine_x = mine.x_position;
            let mine_y = mine.y_position;

            if mine.is_activated {
                for attacker in attackers.values_mut() {
                    let (attacker_x, attacker_y) = attacker.get_current_position()?;
                    let dist = (((mine_x - attacker_x).pow(2) + (mine_y - attacker_y).pow(2))
                        as f32)
                        .sqrt();
                    //check if there is any attacker within the range
                    if dist as i32 <= mine.radius {
                        //damage attckers
                        attacker.get_damage(mine.damage, attacker.path_in_current_frame.len() - 1);
                        mine.is_activated = false;
                    }
                }
            }
        }

        Ok(())
    }
}
