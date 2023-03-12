use crate::constants::*;
use crate::simulation::blocks::BuildingsManager;
use crate::simulation::error::*;
use anyhow::Result;
use std::collections::{HashMap, HashSet};

pub use self::robot::Robot;

pub mod robot;

pub struct RobotsManager {
    pub robots: HashMap<i32, Robot>,
    pub robots_grid: Vec<Vec<HashSet<i32>>>,
    pub robots_destination: HashMap<i32, HashSet<i32>>,
    pub no_of_robots: i32,
    pub shortest_path_grid: Vec<Vec<HashSet<i32>>>,
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
        buildings_manager: &mut BuildingsManager,
    ) -> Result<i32> {
        let robot_ids = self.robots_grid[x as usize][y as usize].clone();
        let mut destroyed_robots = 0;
        for robot_id in robot_ids.iter() {
            let robot = self.robots.get_mut(robot_id).ok_or(KeyError {
                key: *robot_id,
                hashmap: "robots".to_string(),
            })?;
            if robot.take_damage(damage) {
                destroyed_robots += 1;
                robot.assign_destination(
                    buildings_manager,
                    &mut self.robots_destination,
                    &mut self.shortest_path_grid,
                )?;
            }
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
