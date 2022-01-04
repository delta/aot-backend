use crate::models::{AttackType, AttackerPath};
use crate::simulation::blocks::BuildingsManager;
use crate::simulation::robots::RobotsManager;

use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};

use std::collections::HashMap;

#[derive(Debug)]
pub struct Emp {
    x_coord: i32,
    y_coord: i32,
    radius: i32,
    damage: i32,
}

// Returns a hashmap of Emp with time as key
pub fn get_emps(conn: &PgConnection, game_id: i32) -> HashMap<i32, Emp> {
    use crate::schema::attack_type::dsl::attack_type;
    use crate::schema::attacker_path::dsl::{attacker_path, game_id as game_id_field, is_emp};

    let emp_types: HashMap<i32, AttackType> = attack_type
        .load::<AttackType>(conn)
        .expect("Error fetching attack type from db")
        .into_iter()
        .map(|emp| (emp.id, emp))
        .collect();

    attacker_path
        .filter(game_id_field.eq(game_id))
        .filter(is_emp.eq(true))
        .load::<AttackerPath>(conn)
        .expect("Error fetching attacker path from db")
        .into_iter()
        .map(|path| {
            let emp_type = emp_types.get(&path.emp_type.unwrap()).unwrap();
            (
                path.emp_time.unwrap(),
                Emp {
                    x_coord: path.x_coord,
                    y_coord: path.y_coord,
                    radius: emp_type.attack_radius,
                    damage: emp_type.attack_damage,
                },
            )
        })
        .collect()
}

pub fn blast_emp(
    time: i32,
    emps: &HashMap<i32, Emp>,
    robots_manager: &mut RobotsManager,
    buildings_manager: &mut BuildingsManager,
) {
    if emps.contains_key(&time) {
        let emp = emps.get(&time).unwrap();
        let radius = emp.radius;

        for x in emp.x_coord - radius..=emp.x_coord + radius {
            for y in emp.y_coord - radius..=emp.y_coord + radius {
                if (0..40).contains(&x) && (0..40).contains(&y) {
                    let distance = (x - emp.x_coord).abs().pow(2) + (y - emp.y_coord).abs().pow(2);
                    if distance <= radius.pow(2) {
                        let RobotsManager {
                            ref mut robots,
                            ref robots_grid,
                        } = robots_manager;
                        let robot_ids = &robots_grid[x as usize][y as usize];
                        let building_id = buildings_manager.buildings_grid[x as usize][y as usize];

                        for robot_id in robot_ids {
                            let robot = robots.get_mut(robot_id).unwrap();
                            robot.take_damage(emp.damage);
                            todo!("Call assign_destination(robot)");
                        }

                        if building_id != 0 {
                            buildings_manager.damage_building(time, building_id);
                        }
                    }
                }
            }
        }
    }
}
