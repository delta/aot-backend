use std::cmp::Ordering;

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

pub struct Defender {
    pub id: i32,
    pub defender_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub is_alive: bool,
    pub target_id: Option<i32>,
    pub path: Vec<(i32, i32)>,
    pub path_in_current_frame: Vec<(i32, i32)>,
}

pub struct Defenders(Vec<Defender>);

pub enum MovementType {
    Attacker,
    Defender,
    AttackerAndDefender,
}

impl Defenders {
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        use crate::schema::{block_type, building_type, defender_type, map_spaces};
        let result: Vec<(MapSpaces, (BuildingType, BlockType, DefenderType))> = map_spaces::table
            .inner_join(
                building_type::table
                    .inner_join(block_type::table)
                    .inner_join(defender_type::table),
            )
            .filter(map_spaces::map_id.eq(map_id))
            .load::<(MapSpaces, (BuildingType, BlockType, DefenderType))>(conn)
            .map_err(|err| DieselError {
                table: "map_spaces",
                function: function!(),
                error: err,
            })?;

        let mut defenders: Vec<Defender> = Vec::new();

        for (defender_id, (map_space, (_, block_type, defender_type))) in result.iter().enumerate()
        {
            let (hut_x, hut_y) = Self::get_absolute_entrance(map_space, block_type);
            let path = vec![(hut_x, hut_y)];
            defenders.push(Defender {
                id: defender_id as i32 + 1,
                defender_type: defender_type.id,
                radius: defender_type.radius,
                speed: defender_type.speed,
                damage: defender_type.radius,
                is_alive: true,
                target_id: None,
                path,
                path_in_current_frame: Vec::new(),
            })
        }
        Ok(Defenders(defenders))
    }

    pub fn simulate(
        &mut self,
        attacker_manager: &mut AttackManager,
        building_manager: &mut BuildingsManager,
    ) -> Result<()> {
        let Defenders(defenders) = self;
        let attackers = &mut attacker_manager.attackers;
        let shortest_paths = &building_manager.shortest_paths;

        defenders.sort_by(|defender_1, defender_2| {
            (defender_1.path.len() / (defender_1.speed as usize))
                .cmp(&(defender_2.path.len() / (defender_2.speed as usize)))
        });
        for defender in defenders.iter_mut() {
            defender.path_in_current_frame = Vec::new();
            if defender.is_alive {
                if let Some(attacker_id) = defender.target_id {
                    let attacker = attackers.get_mut(&attacker_id).ok_or(KeyError {
                        key: attacker_id,
                        hashmap: "attackers".to_string(),
                    })?;
                    let movement_sequence =
                        Self::generate_movement_sequence(attacker.speed, defender.speed);
                    let mut current_attacker_pos = attacker.path_in_current_frame.len() - 1;
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
                } else {
                    let (defender_pos_x, defender_pos_y) = defender.path[defender.path.len() - 1];
                    defender
                        .path_in_current_frame
                        .push((defender_pos_x, defender_pos_y));
                    let mut target_id: Option<i32> = None;
                    let mut optimal_path: Vec<(i32, i32)> = Vec::new();
                    let mut optimal_distance = i32::MAX;
                    for attacker in attackers.values() {
                        let (attacker_pos_x, attacker_pos_y) = attacker.get_current_position()?;
                        let distance = (attacker_pos_x - defender_pos_x).pow(2)
                            + (attacker_pos_y - defender_pos_y).pow(2);
                        if distance > defender.radius.pow(2) {
                            continue;
                        }
                        let source_dest = SourceDest {
                            source_x: defender_pos_x,
                            source_y: defender_pos_y,
                            dest_x: attacker_pos_x,
                            dest_y: attacker_pos_x,
                        };
                        if distance < optimal_distance && attacker.is_alive {
                            optimal_distance = distance;
                            target_id = Some(attacker.id);
                            optimal_path = shortest_paths
                                .get(&source_dest)
                                .ok_or(ShortestPathNotFoundError(source_dest))?
                                .clone();
                            optimal_path.reverse();
                        }
                    }
                    defender.target_id = target_id;
                    if target_id.is_some() {
                        defender.path = optimal_path;
                    }
                }
            }
        }
        Ok(())
    }

    fn damage_attacker(
        attacker: &mut Attacker,
        defender: &mut Defender,
        current_attacker_pos: &usize,
    ) {
        attacker.get_damage(defender.damage, *current_attacker_pos);
        defender.is_alive = false;
    }

    fn move_defender(
        attacker: &mut Attacker,
        defender: &mut Defender,
        current_attacker_pos: &usize,
    ) -> Result<()> {
        if defender.is_alive && defender.path.len() > 1 {
            if !attacker.path_in_current_frame[*current_attacker_pos].is_alive {
                defender.is_alive = false;
                return Ok(());
            }
            defender.path.pop();
            defender
                .path_in_current_frame
                .insert(0, *defender.path.last().unwrap());
            let attacker_pos = Self::get_attacker_position(attacker, current_attacker_pos);
            let defender_pos = Self::get_defender_position(defender)?;
            if attacker_pos == defender_pos {
                Self::damage_attacker(attacker, defender, current_attacker_pos);
            }
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
            if defender.path.len() > 1 {
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

    fn get_absolute_entrance(map_space: &MapSpaces, block_type: &BlockType) -> (i32, i32) {
        match map_space.rotation {
            0 => (
                map_space.x_coordinate + block_type.entrance_x,
                map_space.y_coordinate + block_type.entrance_y,
            ),
            90 => (
                map_space.x_coordinate - block_type.entrance_y,
                map_space.y_coordinate + block_type.entrance_x,
            ),
            180 => (
                map_space.x_coordinate - block_type.entrance_x,
                map_space.y_coordinate - block_type.entrance_y,
            ),
            270 => (
                map_space.x_coordinate + block_type.entrance_y,
                map_space.y_coordinate - block_type.entrance_x,
            ),
            _ => panic!("Invalid Map Space Rotation"),
        }
    }

    pub fn post_simulate(&self) -> Vec<RenderDefender> {
        let mut defender_positions = Vec::new();
        let Defenders(defenders) = self;
        for defender in defenders {
            for path in defender.path_in_current_frame.iter() {
                defender_positions.push(RenderDefender {
                    defender_id: defender.id,
                    x_position: path.0,
                    y_position: path.1,
                    is_alive: defender.is_alive,
                    defender_type: defender.defender_type,
                })
            }
        }
        defender_positions
    }
}
