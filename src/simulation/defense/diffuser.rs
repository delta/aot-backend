use crate::models::*;
use crate::simulation::attack::{
    attacker::Attacker,
    emp::{Emp, Emps},
    AttackManager,
};
use crate::simulation::blocks::*;
use anyhow::{Ok, Result};
use diesel::prelude::*;
use diesel::PgConnection;
use std::collections::{HashMap, HashSet};

pub struct Diffuser {
    pub id: i32,
    pub diffuser_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub hut_x_position: i32,
    pub hut_y_position: i32,
    pub target_emp_path_id: Option<usize>,
    pub target_emp_attacker_id: Option<i32>,
    pub diffuser_path: Vec<(i32, i32)>,
}

#[allow(dead_code)]
pub struct Diffusers(Vec<Diffuser>);

impl Diffusers {
    #[allow(dead_code)]
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{building_type, diffuser_type, map_spaces};
        let joined_table = map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .inner_join(building_type::table)
            .inner_join(
                diffuser_type::table
                    .on(building_type::building_category.eq(BuildingCategory::Diffuser)),
            );

        let diffusers: Vec<Diffuser> = joined_table
            .load::<(MapSpaces, BuildingType, DiffuserType)>(conn)?
            .into_iter()
            .map(|(map_space, _, diffuser_type)| Diffuser {
                id: map_space.id,
                diffuser_type: diffuser_type.id,
                radius: diffuser_type.radius,
                hut_x_position: map_space.x_coordinate,
                hut_y_position: map_space.y_coordinate,
                x_position: map_space.x_coordinate,
                y_position: map_space.y_coordinate,
                is_alive: true,
                target_emp_path_id: None,
                target_emp_attacker_id: None,
                speed: diffuser_type.speed,
                diffuser_path: Vec::new(),
            })
            .collect();
        Ok(Diffusers(diffusers))
    }

    fn simulate_diffuser(
        diffuser: &mut Diffuser,
        time_emps_map: &mut HashMap<i32, HashSet<Emp>>,
        shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
    ) -> Result<()> {
        let mut to_remove_emp = false;
        let mut remove_emp_time: Option<i32> = None;
        let mut remove_emp: Option<Emp> = None;

        for (emp_time, emps) in time_emps_map.iter() {
            for emp in emps.iter() {
                if (emp.path_id == diffuser.target_emp_path_id.unwrap())
                    && (emp.attacker_id == diffuser.target_emp_attacker_id.unwrap())
                {
                    //got the target emp
                    if (emp.x_coord == diffuser.x_position) && (emp.y_coord == diffuser.y_position)
                    {
                        //chechking whether diffuser reached emp
                        //kill the diffuser
                        diffuser.is_alive = false;
                        //remove the emp
                        to_remove_emp = true;
                        remove_emp_time = Some(*emp_time);
                        remove_emp = Some(emp.clone());
                        break;
                    } else {
                        //otherwise just move towards it
                        let (next_x, next_y) = diffuser.diffuser_path.pop().unwrap();
                        diffuser.x_position = next_x;
                        diffuser.y_position = next_y;

                        return Ok(());
                    }
                }
            }
            if to_remove_emp {
                break;
            }
        }

        //remove empdiffuser.diffuser_path = new_path;
        if to_remove_emp {
            let emps = time_emps_map.get_mut(&remove_emp_time.unwrap()).unwrap();
            emps.remove(&remove_emp.unwrap());
            return Ok(());
        }

        //target emp is already diffused so diffuser in it's way to it's hut
        diffuser.target_emp_attacker_id = None;
        diffuser.target_emp_path_id = None;
        let mut back_to_hut_path = shortest_paths
            .get(&SourceDest {
                source_x: diffuser.x_position,
                source_y: diffuser.y_position,
                dest_x: diffuser.hut_x_position,
                dest_y: diffuser.hut_y_position,
            })
            .unwrap()
            .clone();

        back_to_hut_path.reverse();
        back_to_hut_path.pop();

        diffuser.diffuser_path = back_to_hut_path;

        Ok(())
    }

    fn get_hut_entrance(
        diffuser: &Diffuser,
        conn: &mut PgConnection,
        map_id: i32,
    ) -> Result<(i32, i32)> {
        use crate::schema::{block_type, building_type, map_spaces};
        let map_spaces_result = map_spaces::table
            // .filter(map_spaces :: map_id.eq(map_id))
            // .filter(map_spaces :: x_coordinate.eq(diffuser.hut_x_position))
            // .filter(map_spaces :: y_coordinate.eq(diffuser.hut_y_position));
            .filter(
                map_spaces::map_id.eq(map_id).and(
                    map_spaces::x_coordinate
                        .eq(diffuser.hut_x_position)
                        .and(map_spaces::y_coordinate.eq(diffuser.hut_y_position)),
                ),
            );

        let join_table =
            map_spaces_result.inner_join(building_type::table.inner_join(block_type::table));

        let block_entrance: Vec<(i32, i32)> = join_table
            .load::<(MapSpaces, (BuildingType, BlockType))>(conn)?
            .into_iter()
            .map(|(_, (_, block_type))| (block_type.entrance_x, block_type.entrance_y))
            .collect();

        Ok(block_entrance[0])
    }

    fn assign_diffuser(
        diffuser: &mut Diffuser,
        time_emps_map: &mut HashMap<i32, HashSet<Emp>>,
        attackers: &HashMap<i32, Attacker>,
        shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
        map_id: i32,
        conn: &mut PgConnection,
        minute: i32,
    ) -> Result<()> {
        let (hut_entrance_x, hut_entrance_y) = Diffusers::get_hut_entrance(diffuser, conn, map_id)?;
        let mut optimal_emp: Option<Emp> = None;
        let mut optimal_emp_path: Option<Vec<(i32, i32)>> = None;

        for (emp_time, emps) in time_emps_map.iter() {
            for emp in emps.iter() {
                let attacker = attackers.get(&emp.attacker_id).unwrap();
                if attacker.is_planted(emp.path_id)? {
                    //this emp is visible
                    if (diffuser.x_position == diffuser.hut_x_position)
                        && (diffuser.y_position == diffuser.hut_y_position)
                    {
                        //diffuser is in his hut
                        let mut new_path = shortest_paths
                            .get(&SourceDest {
                                source_x: hut_entrance_x,
                                source_y: hut_entrance_y,
                                dest_x: emp.x_coord,
                                dest_y: emp.y_coord,
                            })
                            .unwrap()
                            .clone();

                        new_path.reverse();

                        let time_taken = new_path.len() as i32;
                        if (time_taken + minute) < *emp_time {
                            match &optimal_emp {
                                Some(opt_emp) => {
                                    if opt_emp.damage < emp.damage {
                                        optimal_emp = Some(emp.clone());
                                        optimal_emp_path = Some(new_path);
                                    }
                                }
                                None => {
                                    optimal_emp = Some(emp.clone());
                                    optimal_emp_path = Some(new_path);
                                }
                            }
                        }
                    } else {
                        //diffuser not in hut
                        let mut new_path = shortest_paths
                            .get(&SourceDest {
                                source_x: diffuser.x_position,
                                source_y: diffuser.y_position,
                                dest_x: emp.x_coord,
                                dest_y: emp.y_coord,
                            })
                            .unwrap()
                            .clone();

                        new_path.reverse();
                        new_path.pop();

                        let time_taken = new_path.len() as i32;
                        if (time_taken + minute) < *emp_time {
                            match &optimal_emp {
                                Some(opt_emp) => {
                                    if emp.damage > opt_emp.damage {
                                        optimal_emp = Some(emp.clone());
                                        optimal_emp_path = Some(new_path);
                                    }
                                }
                                None => {
                                    optimal_emp = Some(emp.clone());
                                    optimal_emp_path = Some(new_path);
                                }
                            }
                        }
                    }
                }
            }
        }

        match optimal_emp {
            Some(emp) => {
                diffuser.diffuser_path = optimal_emp_path.unwrap();
                diffuser.target_emp_path_id = Some(emp.path_id);
                diffuser.target_emp_attacker_id = Some(emp.attacker_id);
            }
            None => {
                //diffuser  is not assigned to any emp
                if !diffuser.diffuser_path.is_empty() {
                    //diffuser not in hut
                    let (next_x, next_y) = diffuser.diffuser_path.pop().unwrap();
                    diffuser.x_position = next_x;
                    diffuser.y_position = next_y;
                }
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn simulate(
        &mut self,
        minute: i32,
        attack_manager: &mut AttackManager,
        conn: &mut PgConnection,
        map_id: i32,
    ) -> Result<()> {
        //get list of active emps within radius
        let Diffusers(diffusers) = self;
        let Emps(time_emps_map) = &mut attack_manager.emps;
        let attackers = &attack_manager.attackers;
        let shortest_paths = BuildingsManager::get_shortest_paths(conn, map_id)?;

        for diffuser in diffusers.iter_mut() {
            if diffuser.is_alive {
                match diffuser.target_emp_path_id {
                    Some(_) => {
                        (Diffusers::simulate_diffuser(diffuser, time_emps_map, &shortest_paths))?;
                    }
                    None => {
                        (Diffusers::assign_diffuser(
                            diffuser,
                            time_emps_map,
                            attackers,
                            &shortest_paths,
                            map_id,
                            conn,
                            minute,
                        ))?;
                    }
                }
            }
        }
        Ok(())
    }
}
