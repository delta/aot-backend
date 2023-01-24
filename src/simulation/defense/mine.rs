use std::collections::HashMap;

use crate::models::*;
use crate::simulation::attack::AttackManager;
use crate::simulation::RenderMine;
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
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{building_type, map_spaces, mine_type};

        let joined_table = map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .inner_join(building_type::table.inner_join(mine_type::table));

        let mines: Vec<Mine> = joined_table
            .load::<(MapSpaces, (BuildingType, MineType))>(conn)?
            .into_iter()
            .enumerate()
            .map(|(mine_id, (map_space, (_, mine_type)))| Mine {
                id: (mine_id as i32) + 1,
                mine_type: mine_type.id,
                damage: mine_type.damage,
                radius: mine_type.radius,
                is_activated: true,
                x_position: map_space.x_coordinate,
                y_position: map_space.y_coordinate,
            })
            .collect();

        Ok(Mines(mines))
    }

    pub fn simulate(&mut self, attack_manager: &mut AttackManager) -> Result<()> {
        //get pos of attckers
        let Mines(mines) = self;
        let attackers = &mut attack_manager.attackers;

        for mine in mines.iter_mut() {
            let mine_x = mine.x_position;
            let mine_y = mine.y_position;

            if mine.is_activated {
                for attacker in attackers.values_mut() {
                    let attacker_path = attacker.path_in_current_frame.clone();
                    for (position, attacker_path_stats) in attacker_path.iter().rev().enumerate() {
                        let (attacker_x, attacker_y) = (
                            attacker_path_stats.attacker_path.x_coord,
                            attacker_path_stats.attacker_path.y_coord,
                        );
                        let dist = (mine_x - attacker_x).pow(2) + (mine_y - attacker_y).pow(2);
                        //check if there is any attacker within the range
                        if dist <= mine.radius.pow(2) {
                            //damage attckers
                            attacker.get_damage(mine.damage, attacker_path.len() - position - 1);
                            mine.is_activated = false;
                            break;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn post_simulate(&mut self) -> HashMap<i32, RenderMine> {
        let mut render_mines = HashMap::new();

        let Mines(mines) = self;

        for mine in mines.iter() {
            render_mines.insert(
                mine.id,
                RenderMine {
                    mine_id: mine.id,
                    x_position: mine.x_position,
                    y_position: mine.y_position,
                    mine_type: mine.mine_type,
                    is_activated: mine.is_activated,
                },
            );
        }

        render_mines
    }

    pub fn get_intial_mines(&self) -> Vec<RenderMine> {
        let Mines(mines) = self;

        let mut initial_mines = Vec::new();
        for mine in mines.iter() {
            initial_mines.push(RenderMine {
                mine_id: mine.id,
                x_position: mine.x_position,
                y_position: mine.y_position,
                mine_type: mine.mine_type,
                is_activated: mine.is_activated,
            });
        }

        initial_mines
    }
}
