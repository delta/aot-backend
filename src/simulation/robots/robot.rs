use crate::constants::*;
use crate::simulation::blocks::{BuildingsManager, SourceDest};
use crate::simulation::error::*;
use anyhow::Result;
use rand::Rng;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Robot {
    pub id: i32,
    pub health: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub destination: i32,
    pub stay_in_time: i32,
    pub(in crate::simulation::robots) current_path: Vec<(i32, i32)>,
}

impl Robot {
    pub fn take_damage(&mut self, damage: i32) -> bool {
        if self.health <= 0 {
            return false;
        }
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
        };
        true
    }

    fn enter_building(&mut self, buildings_manager: &mut BuildingsManager) -> Result<()> {
        self.stay_in_time = rand::thread_rng().gen_range(1..=MAX_STAY_IN_TIME);
        let building = buildings_manager
            .buildings
            .get_mut(&self.destination)
            .ok_or(KeyError {
                key: self.destination,
                hashmap: "buildings".to_string(),
            })?;
        // add population
        building.population += 1;
        Ok(())
    }

    fn exit_building(
        &mut self,
        buildings_manager: &mut BuildingsManager,
        robots_grid: &mut [Vec<HashSet<i32>>],
    ) -> Result<()> {
        let building = buildings_manager
            .buildings
            .get_mut(&self.destination)
            .ok_or(KeyError {
                key: self.destination,
                hashmap: "buildings".to_string(),
            })?;
        // remove population
        robots_grid[building.absolute_entrance_x as usize][building.absolute_entrance_y as usize]
            .remove(&self.id);
        self.stay_in_time = 0;
        building.population -= 1;
        Ok(())
    }

    pub fn assign_destination(
        &mut self,
        buildings_manager: &BuildingsManager,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
        shortest_path_grid: &mut [Vec<HashSet<i32>>],
    ) -> Result<()> {
        if self.health <= 0 {
            return Ok(());
        }
        let destination_id =
            buildings_manager.get_weighted_random_building(self.x_position, self.y_position)?;
        let destination = buildings_manager
            .buildings
            .get(&destination_id)
            .ok_or(KeyError {
                key: destination_id,
                hashmap: "buildings".to_string(),
            })?;
        self.stay_in_time = 0;
        self.destination = destination_id;
        robots_destination
            .entry(destination_id)
            .or_insert_with(HashSet::new);
        robots_destination
            .get_mut(&destination_id)
            .unwrap()
            .insert(self.id);
        let source_dest = SourceDest {
            source_x: self.x_position,
            source_y: self.y_position,
            dest_x: destination.absolute_entrance_x,
            dest_y: destination.absolute_entrance_y,
        };
        self.current_path.iter().for_each(|(x, y)| {
            shortest_path_grid[*x as usize][*y as usize].remove(&self.id);
        });
        self.current_path = buildings_manager
            .shortest_paths
            .get(&source_dest)
            .ok_or(ShortestPathNotFoundError(source_dest))?
            .clone();
        self.current_path.iter().for_each(|(x, y)| {
            shortest_path_grid[*x as usize][*y as usize].insert(self.id);
        });
        self.current_path.reverse();
        Ok(())
    }

    pub(in crate::simulation::robots) fn move_robot(
        &mut self,
        buildings_manager: &mut BuildingsManager,
        robots_grid: &mut [Vec<HashSet<i32>>],
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
        shortest_path_grid: &mut [Vec<HashSet<i32>>],
    ) -> Result<()> {
        let Robot {
            x_position,
            y_position,
            stay_in_time,
            current_path,
            health,
            ..
        } = self;
        if *health <= 0 {
            return Ok(());
        }

        if *stay_in_time == 0 {
            if let Some((x, y)) = current_path.pop() {
                robots_grid[*x_position as usize][*y_position as usize].remove(&self.id);
                *x_position = x;
                *y_position = y;
                robots_grid[x as usize][y as usize].insert(self.id);
                let building = buildings_manager
                    .buildings
                    .get_mut(&self.destination)
                    .ok_or(KeyError {
                        key: self.destination,
                        hashmap: "buildings".to_string(),
                    })?;
                if x == building.absolute_entrance_x
                    && y == building.absolute_entrance_y
                    && current_path.is_empty()
                {
                    self.enter_building(buildings_manager)?;
                }
            }
        } else {
            *stay_in_time -= 1;
            if *stay_in_time == 0 {
                self.exit_building(buildings_manager, robots_grid)?;
                self.assign_destination(buildings_manager, robots_destination, shortest_path_grid)?;
                self.current_path.pop();
            }
        }
        Ok(())
    }
}
