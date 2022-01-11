use crate::models::AttackerPath;
use attacker::Attacker;
use blocks::BuildingsManager;
use diesel::prelude::*;
use emp::Emps;
use robots::RobotsManager;

pub mod attacker;
pub mod blocks;
pub mod emp;
pub mod robots;

const GAME_TIME_MINUTES: i32 = 480;
const REAL_MINUTES_PER_FRAME: i32 = 2;
const REAL_TIME_SECONDS: i32 = 120;
const NO_OF_FRAMES: i32 = GAME_TIME_MINUTES / REAL_MINUTES_PER_FRAME;
const FRAMES_PER_SECOND: i32 = NO_OF_FRAMES / REAL_TIME_SECONDS;
const ATTACKER_RESTRICTED_FRAMES: i32 = 30;

#[derive(Debug)]
struct RenderAttacker {
    x_position: i32,
    y_position: i32,
    is_alive: bool,
    emp_id: i32,
}

#[derive(Clone, Debug)]
pub struct RenderEmp {
    id: i32,
    time: i32,
    emp_type: i32,
}

#[derive(Debug)]
struct RenderRobot {
    id: i32,
    health: i32,
    x_position: i32,
    y_position: i32,
    in_building: bool,
}

#[derive(Debug)]
pub struct RenderSimulation {
    attacker: RenderAttacker,
    robots: Vec<RenderRobot>,
}

#[allow(dead_code)]
pub struct Simulator {
    buildings_manager: BuildingsManager,
    robots_manager: RobotsManager,
    attacker: Attacker,
    emps: Emps,
    frames_passed: i32,
    render_emps: Vec<RenderEmp>,
}

#[allow(dead_code)]
impl Simulator {
    pub fn new(game_id: i32) -> Self {
        use crate::schema::{attacker_path, game};

        let pool = crate::get_connection_pool();
        let conn = &*pool.get().unwrap();
        let map_id = game::table
            .filter(game::id.eq(game_id))
            .select(game::map_layout_id)
            .first::<i32>(conn)
            .unwrap_or_else(|_| panic!("Could not get map_id for game {}", game_id));

        let buildings_manager = BuildingsManager::new(conn, map_id);
        let robots_manager = RobotsManager::new(&buildings_manager);
        let attacker = Attacker::new(conn, game_id);
        let emps = Emps::new(conn, game_id);
        let render_emps = attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .filter(attacker_path::is_emp.eq(true))
            .load::<AttackerPath>(conn)
            .unwrap_or_else(|_| panic!("Could not get attacker_path for game {}", game_id))
            .iter()
            .map(|path| RenderEmp {
                id: path.id,
                time: path.emp_time.unwrap(),
                emp_type: path.emp_type.unwrap(),
            })
            .collect();

        Simulator {
            buildings_manager,
            robots_manager,
            attacker,
            emps,
            frames_passed: 0,
            render_emps,
        }
    }

    pub fn attacker_allowed(frames_passed: i32) -> bool {
        frames_passed > ATTACKER_RESTRICTED_FRAMES
    }

    pub fn is_second(frames_passed: i32) -> bool {
        frames_passed % FRAMES_PER_SECOND == 0
    }

    pub fn get_second(frames_passed: i32) -> i32 {
        frames_passed / FRAMES_PER_SECOND
    }

    pub fn is_hour(frames_passed: i32) -> bool {
        Self::get_second(frames_passed) % 60 == 0
    }

    pub fn get_hour(frames_passed: i32) -> i32 {
        Self::get_second(frames_passed) / 60
    }

    pub fn simulate(&mut self) -> RenderSimulation {
        let Simulator {
            buildings_manager,
            robots_manager,
            attacker,
            emps,
            frames_passed,
            ..
        } = self;
        *frames_passed += 1;
        robots_manager.move_robots(buildings_manager);
        if Self::attacker_allowed(*frames_passed) {
            attacker.update_position();
        }
        if Self::is_second(*frames_passed) {
            let seconds = Self::get_second(*frames_passed);
            emps.simulate(seconds, robots_manager, buildings_manager, attacker);
            buildings_manager.revive_buildings(seconds);
        }
        if Self::is_hour(*frames_passed) {
            buildings_manager.update_building_weights(Self::get_hour(*frames_passed));
        }

        let render_robots: Vec<RenderRobot> = robots_manager
            .robots
            .values()
            .map(|robot| RenderRobot {
                id: robot.id,
                health: robot.health,
                x_position: robot.x_position,
                y_position: robot.y_position,
                in_building: robot.stay_in_time > 0,
            })
            .collect();
        let render_attacker = RenderAttacker {
            x_position: attacker.get_current_position().0,
            y_position: attacker.get_current_position().1,
            is_alive: attacker.is_alive,
            emp_id: match attacker.path.last() {
                Some(path) => {
                    if path.is_emp {
                        path.id
                    } else {
                        0
                    }
                }
                None => 0,
            },
        };
        RenderSimulation {
            attacker: render_attacker,
            robots: render_robots,
        }
    }
}
