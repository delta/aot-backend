use crate::models::*;
use crate::simulation::attack::{
    attacker::Attacker,
    emp::{Emp, Emps},
    AttackManager,
};
use crate::simulation::blocks::SourceDest;
use crate::simulation::blocks::*;
use crate::simulation::error::{KeyError, ShortestPathNotFoundError};
use crate::simulation::RenderDiffuser;
use anyhow::{Ok, Result};
use diesel::prelude::*;
use diesel::PgConnection;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct DiffuserPathStats {
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub emp_path_id: Option<usize>,
    pub emp_attacker_id: Option<i32>,
}

pub struct Diffuser {
    pub id: i32,
    pub diffuser_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub path_in_current_frame: Vec<DiffuserPathStats>,
    pub is_alive: bool,
    pub init_x_position: i32,
    pub init_y_position: i32,
    pub target_emp_path_id: Option<usize>,
    pub target_emp_attacker_id: Option<i32>,
    pub path: Vec<(i32, i32)>,
}

pub struct Diffusers(Vec<Diffuser>);

impl Diffusers {
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{building_type, diffuser_type, map_spaces};
        let joined_table = map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .inner_join(building_type::table.inner_join(diffuser_type::table));

        let diffusers: Vec<Diffuser> = joined_table
            .load::<(MapSpaces, (BuildingType, DiffuserType))>(conn)?
            .into_iter()
            .enumerate()
            .map(|(diffuser_id, (map_space, (_, diffuser_type)))| Diffuser {
                id: (diffuser_id as i32) + 1,
                diffuser_type: diffuser_type.id,
                radius: diffuser_type.radius,
                is_alive: true,
                target_emp_path_id: None,
                target_emp_attacker_id: None,
                speed: diffuser_type.speed,
                path: vec![(map_space.x_coordinate, map_space.y_coordinate)],
                init_x_position: map_space.x_coordinate,
                init_y_position: map_space.y_coordinate,
                path_in_current_frame: Vec::new(),
            })
            .collect();
        Ok(Diffusers(diffusers))
    }

    fn simulate_diffuser(
        diffuser: &mut Diffuser,
        time_emps_map: &mut HashMap<i32, HashSet<Emp>>,
        shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
        attackers: &mut HashMap<i32, Attacker>,
    ) -> Result<()> {
        let mut to_remove_emp = false;
        let mut remove_emp_time: Option<i32> = None;
        let mut remove_emp: Option<Emp> = None;
        diffuser.path_in_current_frame = Vec::new();
        match diffuser.target_emp_path_id {
            Some(target_emp_path_id) => {
                let target_emp_attacker_id = diffuser.target_emp_attacker_id.unwrap();
                for (emp_time, emps) in time_emps_map.iter() {
                    for emp in emps.iter() {
                        if (emp.path_id == target_emp_path_id)
                            && (emp.attacker_id == target_emp_attacker_id)
                        {
                            Self::move_diffuser(diffuser);

                            if diffuser.is_alive && diffuser.path.len() == 1 {
                                diffuser.is_alive = false;
                                to_remove_emp = true;
                                remove_emp_time = Some(*emp_time);
                                remove_emp = Some(emp.clone());
                            } else {
                                return Ok(());
                            }
                            break;
                        }
                    }

                    if to_remove_emp {
                        break;
                    }
                }

                if to_remove_emp {
                    let emps = time_emps_map.get_mut(&remove_emp_time.unwrap()).unwrap();
                    let remove_emp_ref = remove_emp.unwrap();
                    emps.remove(&remove_emp_ref);
                    let attacker = attackers.get_mut(&remove_emp_ref.attacker_id);

                    if let Some(att) = attacker {
                        let attacker_path = &mut att.path;
                        for path in attacker_path {
                            if path.id == remove_emp_ref.path_id {
                                path.is_emp = false;
                            }
                        }
                    }

                    return Ok(());
                }

                //target emp is already diffused so diffuser in it's way to it's intial position
                let (curr_x, curr_y) = diffuser.path.last().unwrap();

                diffuser.target_emp_attacker_id = None;
                diffuser.target_emp_path_id = None;

                let source_dest = SourceDest {
                    source_x: *curr_x,
                    source_y: *curr_y,
                    dest_x: diffuser.init_x_position,
                    dest_y: diffuser.init_y_position,
                };

                let mut back_to_initial_pos = shortest_paths
                    .get(&source_dest)
                    .ok_or(ShortestPathNotFoundError(source_dest))?
                    .clone();

                back_to_initial_pos.reverse();

                diffuser.path = back_to_initial_pos;
            }
            None => {
                Self::move_diffuser(diffuser);
            }
        }

        Ok(())
    }

    fn move_diffuser(diffuser: &mut Diffuser) {
        if diffuser.is_alive && diffuser.path.len() > 1 {
            let mut split_at_index: usize = 1;
            if diffuser.path.len() > diffuser.speed as usize {
                split_at_index = diffuser.path.len() - diffuser.speed as usize;
            }
            diffuser.path_in_current_frame = diffuser
                .path
                .split_off(split_at_index)
                .into_iter()
                .map(|(x_coord, y_coord)| DiffuserPathStats {
                    x_position: x_coord,
                    y_position: y_coord,
                    is_alive: diffuser.is_alive,
                    emp_path_id: diffuser.target_emp_path_id,
                    emp_attacker_id: diffuser.target_emp_attacker_id,
                })
                .collect();
        }
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

        let (curr_x, curr_y) = *diffuser.path.last().unwrap();

        for (emp_time, emps) in time_emps_map.iter() {
            for emp in emps.iter() {
                let attacker = attackers.get(&emp.attacker_id).ok_or(KeyError {
                    key: emp.attacker_id,
                    hashmap: "attackers".to_string(),
                })?;
                if attacker.is_planted(emp.path_id)? {
                    //this emp is visible
                    //diffuser is in his   his initial position
                    let source_dest = SourceDest {
                        source_x: emp.x_coord,
                        source_y: emp.y_coord,
                        dest_x: diffuser.init_x_position,
                        dest_y: diffuser.init_y_position,
                    };
                    let new_path = shortest_paths
                        .get(&source_dest)
                        .ok_or(ShortestPathNotFoundError(source_dest))?
                        .clone();

                    let dist_pow_2 = (emp.x_coord - curr_x).pow(2) + (emp.y_coord - curr_y).pow(2);
                    let time_taken =
                        (((new_path.len() - 1) as f32) / (diffuser.speed as f32)) as i32;
                    if ((time_taken + minute) < *emp_time) && (dist_pow_2 <= diffuser.radius.pow(2))
                    {
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
                }
            }
        }

        if let Some(emp) = optimal_emp {
            diffuser.path = optimal_emp_path.unwrap();
            diffuser.target_emp_path_id = Some(emp.path_id);
            diffuser.target_emp_attacker_id = Some(emp.attacker_id);

            return Ok(());
        };

        Ok(())
    }

    pub fn simulate(
        &mut self,
        minute: i32,
        attack_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
    ) -> Result<()> {
        //get list of active emps within radius
        let Diffusers(diffusers) = self;
        let Emps(time_emps_map) = &mut attack_manager.emps;
        let attackers = &mut attack_manager.attackers;
        let shortest_paths = &building_manager.shortest_paths;
        diffusers.sort_by(|diffuser_1, diffuser_2| {
            (diffuser_1.path.len() / (diffuser_1.speed as usize))
                .cmp(&(diffuser_2.path.len() / (diffuser_2.speed as usize)))
        });

        for diffuser in diffusers.iter_mut() {
            if diffuser.is_alive {
                match diffuser.target_emp_path_id {
                    Some(_) => {
                        Diffusers::simulate_diffuser(
                            diffuser,
                            time_emps_map,
                            shortest_paths,
                            attackers,
                        )?;
                    }
                    None => {
                        if diffuser.path.len() > 1 {
                            Diffusers::simulate_diffuser(
                                diffuser,
                                time_emps_map,
                                shortest_paths,
                                attackers,
                            )?;
                        } else {
                            Diffusers::assign_diffuser(
                                diffuser,
                                time_emps_map,
                                attackers,
                                shortest_paths,
                                minute,
                            )?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    pub fn get_diffuser_initial_position(&self) -> Vec<RenderDiffuser> {
        let mut render_positions = Vec::new();
        let Diffusers(defenders) = self;
        for diffuser in defenders {
            render_positions.push(RenderDiffuser {
                diffuser_id: diffuser.id,
                x_position: diffuser.init_x_position,
                y_position: diffuser.init_y_position,
                is_alive: diffuser.is_alive,
                diffuser_type: diffuser.diffuser_type,
                emp_path_id: -1,
                emp_attacker_id: -1,
            })
        }
        render_positions
    }

    pub fn post_simulate(&mut self) -> HashMap<i32, Vec<RenderDiffuser>> {
        let mut render_diffusers: HashMap<i32, Vec<RenderDiffuser>> = HashMap::new();
        let Diffusers(diffusers) = self;

        for diffuser in diffusers.iter_mut() {
            let mut diffuser_positions: Vec<RenderDiffuser> = Vec::new();
            let (destination_x, destination_y) = *diffuser.path.last().unwrap();
            diffuser.path_in_current_frame.insert(
                0,
                DiffuserPathStats {
                    x_position: destination_x,
                    y_position: destination_y,
                    is_alive: diffuser.is_alive,
                    emp_path_id: diffuser.target_emp_path_id,
                    emp_attacker_id: diffuser.target_emp_attacker_id,
                },
            );

            while diffuser.path_in_current_frame.len() > 1
                && diffuser.path_in_current_frame.last().unwrap().is_alive
            {
                diffuser.path_in_current_frame.pop();
                let diffuser_stat = diffuser.path_in_current_frame.last().unwrap();
                let mut emp_path_id: i32 = -1;
                let mut emp_attacker_id: i32 = -1;
                if let Some(path_id) = diffuser_stat.emp_path_id {
                    emp_path_id = path_id as i32;
                };

                if let Some(att_id) = diffuser_stat.emp_attacker_id {
                    emp_attacker_id = att_id;
                }
                diffuser_positions.push(RenderDiffuser {
                    diffuser_id: diffuser.id,
                    x_position: diffuser_stat.x_position,
                    y_position: diffuser_stat.y_position,
                    is_alive: diffuser_stat.is_alive,
                    diffuser_type: diffuser.diffuser_type,
                    emp_path_id,
                    emp_attacker_id,
                })
            }
            while diffuser_positions.len() < diffuser.speed as usize {
                let diffuser_stat = diffuser.path_in_current_frame.last().unwrap();
                let mut emp_path_id: i32 = -1;
                let mut emp_attacker_id: i32 = -1;
                if let Some(path_id) = diffuser_stat.emp_path_id {
                    emp_path_id = path_id as i32;
                };

                if let Some(att_id) = diffuser_stat.emp_attacker_id {
                    emp_attacker_id = att_id;
                }
                diffuser_positions.push(RenderDiffuser {
                    diffuser_id: diffuser.id,
                    x_position: diffuser_stat.x_position,
                    y_position: diffuser_stat.y_position,
                    is_alive: diffuser_stat.is_alive,
                    diffuser_type: diffuser.diffuser_type,
                    emp_path_id,
                    emp_attacker_id,
                });
            }
            render_diffusers.insert(diffuser.id, diffuser_positions);
        }

        render_diffusers
    }
}
