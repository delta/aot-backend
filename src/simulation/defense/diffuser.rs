use crate::models::*;
use crate::simulation::attack::{
    attacker::Attacker,
    emp::{Emp, Emps},
    AttackManager,
};
use crate::simulation::blocks::*;
use crate::simulation::error::KeyError;
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
    pub path_in_current_frame: Vec<(i32, i32)>,
    pub is_alive: bool,
    pub init_x_position: i32,
    pub init_y_position: i32,
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
            .inner_join(
                building_type::table.inner_join(
                    diffuser_type::table
                        .on(building_type::building_category.eq(BuildingCategory::Diffuser)),
                ),
            );

        let mut diffuser_id = 0;
        let diffusers: Vec<Diffuser> = joined_table
            .load::<(MapSpaces, (BuildingType, DiffuserType))>(conn)?
            .into_iter()
            .map(|(map_space, (_, diffuser_type))| {
                diffuser_id += 1;

                let new_path: Vec<(i32, i32)> =
                    vec![(map_space.x_coordinate, map_space.y_coordinate)];

                Diffuser {
                    id: diffuser_id,
                    diffuser_type: diffuser_type.id,
                    radius: diffuser_type.radius,
                    x_position: map_space.x_coordinate,
                    y_position: map_space.y_coordinate,
                    is_alive: true,
                    target_emp_path_id: None,
                    target_emp_attacker_id: None,
                    speed: diffuser_type.speed,
                    diffuser_path: new_path.clone(),
                    init_x_position: map_space.x_coordinate,
                    init_y_position: map_space.y_coordinate,
                    path_in_current_frame: new_path,
                }
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
                    let mut step = 0;
                    diffuser.path_in_current_frame = Vec::new();
                    loop {
                        let (curr_x, curr_y) = *diffuser.diffuser_path.last().unwrap();
                        diffuser.path_in_current_frame.push((curr_x, curr_y));
                        if (emp.x_coord == curr_x) && (emp.y_coord == curr_y) {
                            diffuser.is_alive = false;
                            //remove the emp
                            to_remove_emp = true;
                            remove_emp_time = Some(*emp_time);
                            remove_emp = Some(emp.clone());
                            break;
                        }

                        step += 1;

                        if step > diffuser.speed {
                            break;
                        }
                        diffuser.diffuser_path.pop();
                    }

                    diffuser.path_in_current_frame.reverse();

                    if !to_remove_emp {
                        return Ok(());
                    }

                    break;
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

        //target emp is already diffused so diffuser in it's way to it's intial position
        let (curr_x, curr_y) = diffuser.diffuser_path.last().unwrap();

        diffuser.target_emp_attacker_id = None;
        diffuser.target_emp_path_id = None;

        let mut back_to_initial_pos = shortest_paths
            .get(&SourceDest {
                source_x: *curr_x,
                source_y: *curr_y,
                dest_x: diffuser.init_x_position,
                dest_y: diffuser.init_y_position,
            })
            .unwrap()
            .clone();

        back_to_initial_pos.reverse();
        diffuser.path_in_current_frame = vec![(*curr_x, *curr_y)];
        diffuser.diffuser_path = back_to_initial_pos;

        Ok(())
    }

    fn assign_diffuser(
        diffuser: &mut Diffuser,
        time_emps_map: &mut HashMap<i32, HashSet<Emp>>,
        attackers: &HashMap<i32, Attacker>,
        shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
        minute: i32,
    ) -> Result<()> {
        let mut optimal_emp: Option<Emp> = None;
        let mut optimal_emp_path: Option<Vec<(i32, i32)>> = None;

        let (curr_x, curr_y) = *diffuser.diffuser_path.last().unwrap();

        for (emp_time, emps) in time_emps_map.iter() {
            for emp in emps.iter() {
                let attacker = attackers.get(&emp.attacker_id).ok_or(KeyError {
                    key: emp.attacker_id,
                    hashmap: "attackers".to_string(),
                })?;
                if attacker.is_planted(emp.path_id)? {
                    //this emp is visible
                    if (curr_x == diffuser.init_x_position) && (curr_y == diffuser.init_y_position)
                    {
                        //diffuser is in his his initial position
                        let mut new_path = shortest_paths
                            .get(&SourceDest {
                                source_x: diffuser.init_x_position,
                                source_y: diffuser.init_y_position,
                                dest_x: emp.x_coord,
                                dest_y: emp.y_coord,
                            })
                            .unwrap()
                            .clone();

                        new_path.reverse();

                        let time_taken = (new_path.len() - 1) as i32;
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

                        let time_taken = (new_path.len() - 1) as i32;
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
                diffuser.path_in_current_frame = Vec::new();
                let mut step = 0;
                loop {
                    let (curr_x, curr_y) = diffuser.diffuser_path.last().unwrap();
                    diffuser.path_in_current_frame.push((*curr_x, *curr_y));

                    if (*curr_x == diffuser.init_x_position)
                        && (*curr_y == diffuser.init_y_position)
                    {
                        //reached the initial position
                        diffuser.path_in_current_frame.reverse();
                        return Ok(());
                    }

                    step += 1;

                    if step > diffuser.speed {
                        break;
                    }
                    diffuser.diffuser_path.pop();
                }

                diffuser.path_in_current_frame.reverse();
                return Ok(());
            }
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub fn simulate(
        &mut self,
        minute: i32,
        attack_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
    ) -> Result<()> {
        //get list of active emps within radius
        let Diffusers(diffusers) = self;
        let Emps(time_emps_map) = &mut attack_manager.emps;
        let attackers = &attack_manager.attackers;
        let shortest_paths = &building_manager.shortest_paths;

        for diffuser in diffusers.iter_mut() {
            if diffuser.is_alive {
                match diffuser.target_emp_path_id {
                    Some(_) => {
                        (Diffusers::simulate_diffuser(diffuser, time_emps_map, shortest_paths))?;
                    }
                    None => {
                        (Diffusers::assign_diffuser(
                            diffuser,
                            time_emps_map,
                            attackers,
                            shortest_paths,
                            minute,
                        ))?;
                    }
                }
            }
        }
        Ok(())
    }
}
