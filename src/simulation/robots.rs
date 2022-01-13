use crate::simulation::blocks::{BuildingsManager, SourceDest};
use rand::Rng;
use std::collections::{HashMap, HashSet};

const HEALTH: i32 = 20;
const MAX_STAY_IN_TIME: i32 = 10;

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
}

impl Robot {
    pub fn take_damage(&mut self, damage: i32) {
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
        };
    }

    fn enter_building(&mut self, buildings_manager: &mut BuildingsManager) {
        self.stay_in_time = rand::thread_rng().gen_range(1..=MAX_STAY_IN_TIME);
        let building = buildings_manager
            .buildings
            .get_mut(&self.destination)
            .unwrap();
        building.weight -= 1;
    }

    fn exit_building(&self, buildings_manager: &mut BuildingsManager) {
        let building = buildings_manager
            .buildings
            .get_mut(&self.destination)
            .unwrap();
        building.weight += 1;
    }

    pub fn assign_destination(
        &mut self,
        buildings_manager: &BuildingsManager,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
    ) {
        let destination_id =
            buildings_manager.get_weighted_random_building(self.x_position, self.y_position);
        let destination = buildings_manager.buildings.get(&destination_id).unwrap();
        self.destination = destination_id;
        robots_destination
            .entry(destination_id)
            .or_insert_with(HashSet::new);
        robots_destination
            .get_mut(&destination_id)
            .unwrap()
            .insert(self.id);
        self.current_path = buildings_manager
            .shortest_paths
            .get(&SourceDest {
                source_x: self.x_position,
                source_y: self.y_position,
                dest_x: destination.absolute_entrance_x,
                dest_y: destination.absolute_entrance_y,
            })
            .unwrap()
            .clone();
    }

    fn move_robot(
        &mut self,
        buildings_manager: &mut BuildingsManager,
        robots_grid: &mut Vec<Vec<HashSet<i32>>>,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
    ) {
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
                    self.enter_building(buildings_manager);
                }
            }
        } else {
            *stay_in_time -= 1;
            if *stay_in_time == 0 {
                self.exit_building(buildings_manager);
                self.assign_destination(buildings_manager, robots_destination);
            }
        }
    }
}

impl RobotsManager {
    fn initiate_robots(
        buildings_manager: &BuildingsManager,
        robots_destination: &mut HashMap<i32, HashSet<i32>>,
    ) -> HashMap<i32, Robot> {
        let mut robots = HashMap::new();
        for id in 1..=1000 {
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
        buildings_manager.assign_initial_buildings(&mut robots);
        for robot in robots.values_mut() {
            robot.assign_destination(buildings_manager, robots_destination);
        }
        robots
    }

    fn get_robots_grid(robots: &HashMap<i32, Robot>) -> Vec<Vec<HashSet<i32>>> {
        let mut grid = vec![vec![HashSet::new()]];
        for robot in robots.values() {
            let x = robot.x_position;
            let y = robot.y_position;
            grid[x as usize][y as usize].insert(robot.id);
        }
        grid
    }

    pub fn new(buildings_manager: &BuildingsManager) -> Self {
        let mut robots_destination = HashMap::new();
        let robots = Self::initiate_robots(buildings_manager, &mut robots_destination);
        let robots_grid = Self::get_robots_grid(&robots);
        RobotsManager {
            robots,
            robots_grid,
            robots_destination,
        }
    }

    /// damage and reassign destinations for robots at location x, y with damage = emp.damage
    pub fn damage_and_reassign_robots(
        &mut self,
        damage: i32,
        x: i32,
        y: i32,
        buildings_manager: &BuildingsManager,
    ) {
        let robot_ids = &self.robots_grid[x as usize][y as usize];
        for robot_id in robot_ids {
            let robot = self.robots.get_mut(robot_id).unwrap();
            robot.take_damage(damage);
            robot.assign_destination(buildings_manager, &mut self.robots_destination);
        }
    }

    pub fn move_robots(&mut self, buildings_manager: &mut BuildingsManager) {
        for robot in self.robots.values_mut() {
            robot.move_robot(
                buildings_manager,
                &mut self.robots_grid,
                &mut self.robots_destination,
            );
        }
    }
}
