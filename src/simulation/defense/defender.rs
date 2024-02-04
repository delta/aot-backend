use std::cmp::Ordering;
use std::collections::HashMap;
use std::collections::HashSet;

use crate::error::DieselError;
use crate::models::BlockType;
use crate::models::BuildingType;
use crate::models::DefenderType;
use crate::models::MapSpaces;
use crate::simulation::attack::attacker::Attacker;
use crate::simulation::attack::AttackManager;
use crate::simulation::blocks::*;
use crate::simulation::error::EmptyDefenderPathError;
use crate::simulation::error::KeyError;
use crate::simulation::error::ShortestPathNotFoundError;
use crate::simulation::RenderDefender;
use crate::util::function;
use anyhow::{Ok, Result};
use diesel::prelude::*;

#[derive(Debug)]
pub struct DefenderPathStats {
    pub x_coord: i32,
    pub y_coord: i32,
    pub is_alive: bool,
}

pub struct Defender {
    pub id: i32,
    pub defender_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub hut_x: i32,
    pub hut_y: i32,
    pub is_alive: bool,
    pub damage_dealt: bool,
    pub target_id: Option<i32>,
    pub path: Vec<(i32, i32)>,
    pub path_in_current_frame: Vec<DefenderPathStats>,
}

pub struct Defenders(pub Vec<Defender>);

#[derive(Debug)]
pub enum MovementType {
    Attacker,
    Defender,
    AttackerAndDefender,
}

impl Defender {
    pub fn move_defender_to_hut(&mut self) {
        let mut split_at_index: usize = 1;
        if self.path.len() > self.speed as usize {
            split_at_index = self.path.len() - self.speed as usize;
        }
        self.path_in_current_frame = self
            .path
            .split_off(split_at_index)
            .into_iter()
            .map(|(x_coord, y_coord)| DefenderPathStats {
                x_coord,
                y_coord,
                is_alive: self.is_alive,
            })
            .collect();
        let (dest_x, dest_y) = self.path.last().unwrap();
        self.path_in_current_frame.insert(
            0,
            DefenderPathStats {
                x_coord: *dest_x,
                y_coord: *dest_y,
                is_alive: self.is_alive,
            },
        );
        self.path_in_current_frame.pop();
    }
}

impl Defenders {
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{block_type, building_type, defender_type, map_spaces};
        let result: Vec<(MapSpaces, (BlockType, BuildingType, DefenderType))> = map_spaces::table
            .inner_join(
                block_type::table
                    .inner_join(building_type::table)
                    .inner_join(defender_type::table),
            )
            .filter(map_spaces::map_id.eq(map_id))
            .load::<(MapSpaces, (BlockType, BuildingType, DefenderType))>(conn)
            .map_err(|err| DieselError {
                table: "map_spaces",
                function: function!(),
                error: err,
            })?;

        let mut defenders: Vec<Defender> = Vec::new();

        for (defender_id, (map_space, (_, _, defender_type))) in result.iter().enumerate() {
            let (hut_x, hut_y) = (map_space.x_coordinate, map_space.y_coordinate);
            let path = vec![(hut_x, hut_y)];
            defenders.push(Defender {
                id: defender_id as i32 + 1,
                defender_type: defender_type.id,
                radius: defender_type.radius,
                speed: defender_type.speed,
                damage: defender_type.damage,
                is_alive: true,
                damage_dealt: false,
                target_id: None,
                path,
                path_in_current_frame: Vec::new(),
                hut_x,
                hut_y,
            })
        }
        // Sorted to handle multiple defenders attack same attacker at same frame
        defenders.sort_by(|defender_1, defender_2| (defender_2.damage).cmp(&defender_1.damage));
        Ok(Defenders(defenders))
    }

    pub fn simulate(
        &mut self,
        attacker_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
    ) -> Result<()> {
        let Defenders(defenders) = self;
        let attackers = &mut attacker_manager.attackers;
        // let shortest_paths = &building_manager.shortest_paths;

        let mut defender_without_target: HashSet<i32> = HashSet::new();
        for defender in defenders.iter_mut() {
            defender.path_in_current_frame.clear();
            if defender.is_alive {
                if let Some(attacker_id) = defender.target_id {
                    let attacker = attackers.get_mut(&attacker_id).ok_or(KeyError {
                        key: attacker_id,
                        hashmap: "attackers".to_string(),
                    })?;
                    let movement_sequence =
                        Self::generate_movement_sequence(attacker.speed, defender.speed);
                    let mut current_attacker_pos = attacker.path_in_current_frame.len() - 1;
                    if !attacker.path_in_current_frame[0].is_alive {
                        // Self::reassign_defender(defender, shortest_paths)?;
                        defender.move_defender_to_hut();
                        defender.target_id = None;
                        let mut split_at_index: usize = 1;
                        if defender.path.len() > defender.speed as usize {
                            split_at_index = defender.path.len() - defender.speed as usize;
                        }
                        defender.path_in_current_frame = defender
                            .path
                            .split_off(split_at_index)
                            .into_iter()
                            .map(|(x_coord, y_coord)| DefenderPathStats {
                                x_coord,
                                y_coord,
                                is_alive: defender.is_alive,
                            })
                            .collect();
                        let (dest_x, dest_y) = defender.path.last().unwrap();
                        defender.path_in_current_frame.insert(
                            0,
                            DefenderPathStats {
                                x_coord: *dest_x,
                                y_coord: *dest_y,
                                is_alive: defender.is_alive,
                            },
                        );
                        defender.path_in_current_frame.pop();
                        continue;
                    }
                    for movement in movement_sequence.iter() {
                        if !defender.is_alive {
                            break;
                        }
                        match movement {
                            MovementType::Attacker => {
                                Self::move_attacker(attacker, &mut current_attacker_pos, defender)?;
                            }
                            MovementType::Defender => {
                                Self::move_defender(attacker, defender, &current_attacker_pos)?;
                            }
                            MovementType::AttackerAndDefender => {
                                Self::move_attacker(attacker, &mut current_attacker_pos, defender)?;
                                Self::move_defender(attacker, defender, &current_attacker_pos)?;
                            }
                        }
                    }
                } else if defender.path.len() > 1 {
                    defender.move_defender_to_hut();
                } else {
                    defender_without_target.insert(defender.id);
                }
            }
        }
        // for defender in defenders.iter_mut() {
        //     if defender.is_alive && defender_without_target.contains(&defender.id) {
        //         Self::assign_defender(defender, attackers, shortest_paths)?;
        //     }
        // }
        Ok(())
    }

    // pub fn reassign_defender(
    //     defender: &mut Defender,
    //     shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
    // ) -> Result<()> {
    //     let (defender_pos_x, defender_pos_y) = defender.path[defender.path.len() - 1];
    //     let source_dest = SourceDest {
    //         source_x: defender_pos_x,
    //         source_y: defender_pos_y,
    //         dest_x: defender.hut_x,
    //         dest_y: defender.hut_y,
    //     };
    //     defender.path = shortest_paths
    //         .get(&source_dest)
    //         .ok_or(ShortestPathNotFoundError(source_dest))?
    //         .clone();
    //     defender.path.reverse();
    //     Ok(())
    // }

    pub fn assign_defender(
        defender: &mut Defender,
        attackers: &HashMap<i32, Attacker>,
        shortest_paths: &HashMap<SourceDest, Vec<(i32, i32)>>,
    ) -> Result<()> {
        let (defender_pos_x, defender_pos_y) = defender.path[defender.path.len() - 1];
        defender.path_in_current_frame.push(DefenderPathStats {
            x_coord: defender_pos_x,
            y_coord: defender_pos_y,
            is_alive: defender.is_alive,
        });
        let mut target_id: Option<i32> = None;
        let mut optimal_path: Vec<(i32, i32)> = Vec::new();
        let mut optimal_distance = i32::MAX;
        for attacker in attackers.values() {
            if !attacker.path_in_current_frame[0].is_alive {
                continue;
            }
            for attacker_path_stat in attacker.path_in_current_frame.iter().rev() {
                let (attacker_pos_x, attacker_pos_y) = (
                    attacker_path_stat.attacker_path.x_coord,
                    attacker_path_stat.attacker_path.y_coord,
                );
                let distance = (attacker_pos_x - defender_pos_x).pow(2)
                    + (attacker_pos_y - defender_pos_y).pow(2);
                if distance > defender.radius.pow(2) {
                    continue;
                }
                let source_dest = SourceDest {
                    source_x: attacker.path_in_current_frame[0].attacker_path.x_coord,
                    source_y: attacker.path_in_current_frame[0].attacker_path.y_coord,
                    dest_x: defender_pos_x,
                    dest_y: defender_pos_y,
                };
                if distance < optimal_distance {
                    optimal_distance = distance;
                    target_id = Some(attacker.id);
                    optimal_path = shortest_paths
                        .get(&source_dest)
                        .ok_or(ShortestPathNotFoundError(source_dest))?
                        .clone();
                }
                break;
            }
        }
        defender.target_id = target_id;
        if target_id.is_some() {
            defender.path = optimal_path;
        }
        Ok(())
    }

    fn damage_attacker(
        attacker: &mut Attacker,
        defender: &mut Defender,
        current_attacker_pos: &usize,
    ) {
        defender.damage_dealt = true;
        attacker.get_damage(defender.damage, *current_attacker_pos);
        defender.is_alive = false;
    }

    fn move_defender(
        attacker: &mut Attacker,
        defender: &mut Defender,
        current_attacker_pos: &usize,
    ) -> Result<()> {
        if defender.is_alive && defender.path.len() > 1 {
            defender.path.pop();
            let attacker_pos = Self::get_attacker_position(attacker, current_attacker_pos);
            let defender_pos = Self::get_defender_position(defender)?;
            if attacker_pos == defender_pos {
                Self::damage_attacker(attacker, defender, current_attacker_pos);
            }
            let current_pos = *defender.path.last().unwrap();
            defender.path_in_current_frame.insert(
                0,
                DefenderPathStats {
                    x_coord: current_pos.0,
                    y_coord: current_pos.1,
                    is_alive: defender.is_alive,
                },
            );
        }
        Ok(())
    }

    fn move_attacker(
        attacker: &mut Attacker,
        current_attacker_pos: &mut usize,
        defender: &mut Defender,
    ) -> Result<()> {
        if *current_attacker_pos > 0
            && attacker.path_in_current_frame[*current_attacker_pos].is_alive
        {
            *current_attacker_pos -= 1;
            let (attacker_x, attacker_y) =
                Self::get_attacker_position(attacker, current_attacker_pos);
            if defender.path.len() > 1 && defender.is_alive {
                if defender.path[1].0 == attacker_x && defender.path[1].1 == attacker_y {
                    defender.path.remove(0);
                } else {
                    defender.path.insert(0, (attacker_x, attacker_y));
                }
                let attacker_pos = Self::get_attacker_position(attacker, current_attacker_pos);
                let defender_pos = Self::get_defender_position(defender)?;
                if attacker_pos == defender_pos {
                    Self::damage_attacker(attacker, defender, current_attacker_pos);
                }
            }
        }
        Ok(())
    }

    fn get_attacker_position(attacker: &Attacker, current_attacker_pos: &usize) -> (i32, i32) {
        (
            attacker.path_in_current_frame[*current_attacker_pos]
                .attacker_path
                .x_coord,
            attacker.path_in_current_frame[*current_attacker_pos]
                .attacker_path
                .y_coord,
        )
    }

    fn get_defender_position(defender: &Defender) -> Result<(i32, i32)> {
        Ok(*defender
            .path
            .last()
            .ok_or::<EmptyDefenderPathError>(EmptyDefenderPathError)?)
    }

    fn generate_movement_sequence(attacker_speed: i32, defender_speed: i32) -> Vec<MovementType> {
        let mut movement_sequence = Vec::new();
        let mut attacker_time_frame: Vec<i32> = Vec::new();
        let mut defender_time_frame: Vec<i32> = Vec::new();

        for iterator in 1..=defender_speed {
            defender_time_frame.push(iterator * attacker_speed);
        }

        for iterator in 1..=attacker_speed {
            attacker_time_frame.push(iterator * defender_speed);
        }

        while !attacker_time_frame.is_empty() || !defender_time_frame.is_empty() {
            match attacker_time_frame[0].cmp(&defender_time_frame[0]) {
                Ordering::Equal => {
                    attacker_time_frame.remove(0);
                    defender_time_frame.remove(0);
                    movement_sequence.push(MovementType::AttackerAndDefender);
                }
                Ordering::Greater => {
                    defender_time_frame.remove(0);
                    movement_sequence.push(MovementType::Defender);
                }
                Ordering::Less => {
                    attacker_time_frame.remove(0);
                    movement_sequence.push(MovementType::Attacker);
                }
            }
        }
        movement_sequence
    }

    pub fn post_simulate(&mut self) -> HashMap<i32, Vec<RenderDefender>> {
        let mut render_defenders = HashMap::new();
        let Defenders(defenders) = self;
        for defender in defenders {
            let mut defender_positions = Vec::new();
            if defender.path_in_current_frame.is_empty() {
                let destination = defender.path.last().unwrap();
                defender.path_in_current_frame.push(DefenderPathStats {
                    x_coord: destination.0,
                    y_coord: destination.1,
                    is_alive: defender.is_alive,
                })
            }
            for path in defender.path_in_current_frame.iter().rev() {
                defender_positions.push(RenderDefender {
                    defender_id: defender.id,
                    x_position: path.x_coord,
                    y_position: path.y_coord,
                    is_alive: path.is_alive,
                    defender_type: defender.defender_type,
                })
            }
            while defender_positions.len() < defender.speed as usize {
                let path = &defender.path_in_current_frame[0];
                defender_positions.push(RenderDefender {
                    defender_id: defender.id,
                    x_position: path.x_coord,
                    y_position: path.y_coord,
                    is_alive: path.is_alive,
                    defender_type: defender.defender_type,
                })
            }
            render_defenders.insert(defender.id, defender_positions);
        }
        render_defenders
    }

    pub fn get_defender_initial_position(&self) -> Vec<RenderDefender> {
        let mut render_positions = Vec::new();
        let Defenders(defenders) = self;
        for defender in defenders {
            let starting_position = defender.path.last().unwrap();
            render_positions.push(RenderDefender {
                defender_id: defender.id,
                x_position: starting_position.0,
                y_position: starting_position.1,
                is_alive: true,
                defender_type: defender.defender_type,
            })
        }
        render_positions
    }

    pub fn get_damage(&mut self, x_position: i32, y_position: i32) {
        let Defenders(defenders) = self;

        for defender in defenders {
            let (defender_x, defender_y) = defender.path.last().unwrap();
            if *defender_x == x_position && *defender_y == y_position {
                defender.is_alive = false;
            }
        }
    }
}
