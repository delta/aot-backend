use std::collections::{HashMap, HashSet};

#[derive(Debug)]
pub struct Robot {
    pub id: i32,
    pub health: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub destination: i32,
    pub stay_in_time: i32,
}

pub struct RobotsManager {
    pub robots: HashMap<i32, Robot>,
    pub robots_grid: Vec<Vec<HashSet<i32>>>,
}

impl Robot {
    pub fn take_damage(&mut self, damage: i32) {
        if self.health > damage {
            self.health -= damage;
        } else {
            self.health = 0;
        };
    }
}

impl RobotsManager {
    fn initiate_robots() -> HashMap<i32, Robot> {
        HashMap::new()
        // todo!();
    }

    fn get_robots_grid() -> Vec<Vec<HashSet<i32>>> {
        vec![vec![HashSet::new()]]
        // todo!();
    }

    pub fn new() -> Self {
        RobotsManager {
            robots: RobotsManager::initiate_robots(),
            robots_grid: RobotsManager::get_robots_grid(),
        }
    }
}
