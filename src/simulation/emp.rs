use crate::models::{AttackType, AttackerPath};
use crate::simulation::attacker::Attacker;
use crate::simulation::blocks::BuildingsManager;
use crate::simulation::robots::RobotsManager;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Eq, Hash, PartialEq)]
struct Emp {
    path_id: i32,
    x_coord: i32,
    y_coord: i32,
    radius: i32,
    damage: i32,
}

pub struct Emps(HashMap<i32, HashSet<Emp>>);

impl Emps {
    // Returns a hashmap of Emp with time as key
    pub fn new(conn: &PgConnection, game_id: i32) -> Self {
        use crate::schema::{attack_type, attacker_path};

        let mut emps = HashMap::new();
        let emp_types: HashMap<i32, AttackType> = attack_type::table
            .load::<AttackType>(conn)
            .expect("Error fetching attack type from db")
            .into_iter()
            .map(|emp| (emp.id, emp))
            .collect();

        attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .filter(attacker_path::is_emp.eq(true))
            .load::<AttackerPath>(conn)
            .expect("Error fetching attacker path from db")
            .into_iter()
            .for_each(|path| {
                let emp_type = emp_types.get(&path.emp_type.unwrap()).unwrap();
                let emp_time = path.emp_time.unwrap();
                let emp = Emp {
                    path_id: path.id,
                    x_coord: path.x_coord,
                    y_coord: path.y_coord,
                    radius: emp_type.attack_radius,
                    damage: emp_type.attack_damage,
                };

                emps.entry(emp_time).or_insert_with(HashSet::new);
                emps.get_mut(&emp_time).unwrap().insert(emp);
            });
        Emps(emps)
    }

    pub fn simulate(
        &self,
        time: i32,
        robots_manager: &mut RobotsManager,
        buildings_manager: &mut BuildingsManager,
        attacker: &mut Attacker,
    ) {
        let Emps(emps) = self;
        if !emps.contains_key(&time) {
            return;
        }
        for emp in emps.get(&time).unwrap() {
            if !attacker.is_planted(emp.path_id) {
                return;
            }
            let radius = emp.radius;

            let (attacker_x, attacker_y) = attacker.get_current_position();
            let attacker_distance =
                (attacker_x - emp.x_coord).pow(2) + (attacker_y - emp.y_coord).pow(2);
            if attacker_distance <= radius.pow(2) {
                attacker.kill();
            }

            for x in emp.x_coord - radius..=emp.x_coord + radius {
                for y in emp.y_coord - radius..=emp.y_coord + radius {
                    if !(0..40).contains(&x) || !(0..40).contains(&y) {
                        return;
                    }
                    let distance = (x - emp.x_coord).pow(2) + (y - emp.y_coord).pow(2);
                    if distance > radius.pow(2) {
                        return;
                    }

                    let RobotsManager {
                        robots,
                        robots_grid,
                    } = robots_manager;
                    let robot_ids = &robots_grid[x as usize][y as usize];
                    let building_id = buildings_manager.buildings_grid[x as usize][y as usize];

                    for robot_id in robot_ids {
                        let robot = robots.get_mut(robot_id).unwrap();
                        robot.take_damage(emp.damage);
                        robot.assign_destination(buildings_manager);
                    }

                    if building_id != 0 {
                        buildings_manager.damage_building(time, building_id);
                    }
                }
            }
        }
    }
}
