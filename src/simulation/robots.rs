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
    current_path: Vec<(i32, i32)>,
}

pub struct RobotsManager {
    pub robots: HashMap<i32, Robot>,
    pub robots_grid: Vec<Vec<HashSet<i32>>>,
    pub robots_destination: HashMap<i32, HashSet<i32>>,
    pub no_of_robots: i32,
    pub shortest_path_grid: Vec<Vec<HashSet<i32>>>,
}

impl Robot {
    pub fn take_damage(&mut self, damage: i32) {
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
        };
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

    fn exit_building(&self, buildings_manager: &mut BuildingsManager) -> Result<()> {
        let building = buildings_manager
            .buildings
            .get_mut(&self.destination)
            .ok_or(KeyError {
                key: self.destination,
                hashmap: "buildings".to_string(),
            })?;
        // remove population
        building.population -= 1;
        Ok(())
    }

    pub fn assign_destination(
        &mut self,
        buildings_manager: &BuildingsManager,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
        shortest_path_grid: &mut [Vec<HashSet<i32>>],
    ) -> Result<()> {
        let destination_id =
            buildings_manager.get_weighted_random_building(self.x_position, self.y_position)?;
        let destination = buildings_manager
            .buildings
            .get(&destination_id)
            .ok_or(KeyError {
                key: destination_id,
                hashmap: "buildings".to_string(),
            })?;
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

    fn move_robot(
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
            ..
        } = self;
        if *stay_in_time == 0 {
            match current_path.pop() {
                Some((x, y)) => {
                    robots_grid[*x_position as usize][*y_position as usize].remove(&self.id);
                    *x_position = x;
                    *y_position = y;
                    robots_grid[x as usize][y as usize].insert(self.id);
                }
                None => {
                    self.enter_building(buildings_manager)?;
                }
            }
        } else {
            *stay_in_time -= 1;
            if *stay_in_time == 0 {
                self.exit_building(buildings_manager)?;
                self.assign_destination(buildings_manager, robots_destination, shortest_path_grid)?;
            }
        }
        Ok(())
    }
}

impl RobotsManager {
    fn initiate_robots(
        buildings_manager: &BuildingsManager,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
        shortest_path_grid: &mut [Vec<HashSet<i32>>],
        no_of_robots: i32,
    ) -> Result<HashMap<i32, Robot>> {
        let mut robots = HashMap::new();
        for id in 1..=no_of_robots {
            robots.insert(
                id,
                Robot {
                    id,
                    health: HEALTH,
                    x_position: 0,
                    y_position: 0,
                    destination: 0,
                    stay_in_time: 0,
                    current_path: Vec::new(),
                },
            );
        }
        buildings_manager.assign_initial_buildings(&mut robots)?;
        for robot in robots.values_mut() {
            robot.assign_destination(buildings_manager, robots_destination, shortest_path_grid)?;
        }
        Ok(robots)
    }

    fn get_robots_grid(robots: &HashMap<i32, Robot>) -> Vec<Vec<HashSet<i32>>> {
        let mut grid = vec![vec![HashSet::new(); MAP_SIZE]; MAP_SIZE];
        for robot in robots.values() {
            let x = robot.x_position;
            let y = robot.y_position;
            grid[x as usize][y as usize].insert(robot.id);
        }
        grid
    }

    pub fn new(buildings_manager: &BuildingsManager, no_of_robots: i32) -> Result<Self> {
        let mut robots_destination = HashMap::new();
        let mut shortest_path_grid = vec![vec![HashSet::new(); MAP_SIZE]; MAP_SIZE];
        let robots = Self::initiate_robots(
            buildings_manager,
            &mut robots_destination,
            &mut shortest_path_grid,
            no_of_robots,
        )?;
        let robots_grid = Self::get_robots_grid(&robots);
        // TODO: is property needed?
        Ok(RobotsManager {
            robots,
            robots_grid,
            robots_destination,
            no_of_robots,
            shortest_path_grid,
        })
    }

    /// damage and reassign destinations for robots at location x, y with damage = emp.damage
    pub fn damage_and_reassign_robots(
        &mut self,
        damage: i32,
        x: i32,
        y: i32,
        buildings_manager: &BuildingsManager,
    ) -> Result<i32> {
        let robot_ids = &self.robots_grid[x as usize][y as usize];
        let mut destroyed_robots = 0;
        for robot_id in robot_ids {
            let robot = self.robots.get_mut(robot_id).ok_or(KeyError {
                key: *robot_id,
                hashmap: "robots".to_string(),
            })?;
            robot.take_damage(damage);
            if robot.health <= 0 {
                destroyed_robots += 1;
            }
            robot.assign_destination(
                buildings_manager,
                &mut self.robots_destination,
                &mut self.shortest_path_grid,
            )?;
        }
        Ok(destroyed_robots)
    }

    pub fn move_robots(&mut self, buildings_manager: &mut BuildingsManager) -> Result<()> {
        for robot in self.robots.values_mut() {
            robot.move_robot(
                buildings_manager,
                &mut self.robots_grid,
                &mut self.robots_destination,
                &mut self.shortest_path_grid,
            )?;
        }
        Ok(())
    }
}
