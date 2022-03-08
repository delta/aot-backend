use crate::constants::*;
use crate::error::DieselError;
use crate::models::{AttackerPath, NewAttackerPath};
use crate::util::function;
use anyhow::Result;
use attacker::Attacker;
use blocks::BuildingsManager;
use diesel::prelude::*;
use emp::Emps;
use robots::RobotsManager;
use serde::Serialize;

pub mod attacker;
pub mod blocks;
pub mod emp;
pub mod error;
pub mod robots;

#[derive(Debug, Serialize)]
pub struct RenderAttacker {
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub emp_id: usize,
}

#[derive(Debug, Serialize)]
pub struct RenderRobot {
    pub id: i32,
    pub health: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub in_building: bool,
}

#[derive(Debug, Serialize)]
pub struct RenderSimulation {
    pub attacker: RenderAttacker,
    pub robots: Vec<RenderRobot>,
}

pub struct Simulator {
    buildings_manager: BuildingsManager,
    robots_manager: RobotsManager,
    attacker: Attacker,
    emps: Emps,
    frames_passed: i32,
    pub no_of_robots: i32,
}

impl Simulator {
    pub fn new(
        game_id: i32,
        attacker_path: &[NewAttackerPath],
        conn: &PgConnection,
    ) -> Result<Self> {
        use crate::schema::{game, levels_fixture, map_layout};

        let map_id = game::table
            .filter(game::id.eq(game_id))
            .select(game::map_layout_id)
            .first::<i32>(conn)
            .map_err(|err| DieselError {
                table: "game",
                function: function!(),
                error: err,
            })?;
        let no_of_robots = map_layout::table
            .inner_join(levels_fixture::table)
            .select(levels_fixture::no_of_robots)
            .filter(map_layout::id.eq(map_id))
            .first::<i32>(conn)
            .map_err(|err| DieselError {
                table: "map_layout levels_fixture",
                function: function!(),
                error: err,
            })?;

        let buildings_manager = BuildingsManager::new(conn, map_id)?;
        let robots_manager = RobotsManager::new(&buildings_manager, no_of_robots)?;
        let mut attacker_path: Vec<AttackerPath> = attacker_path
            .iter()
            .enumerate()
            .map(|(id, path)| AttackerPath {
                id: id + 1,
                x_coord: path.x_coord,
                y_coord: path.y_coord,
                is_emp: path.is_emp,
                emp_type: path.emp_type,
                emp_time: path.emp_time,
            })
            .collect();
        attacker_path.reverse();
        let emps = Emps::new(conn, &attacker_path)?;
        let attacker = Attacker::new(attacker_path);

        Ok(Simulator {
            buildings_manager,
            robots_manager,
            attacker,
            emps,
            frames_passed: 0,
            no_of_robots,
        })
    }

    pub fn attacker_allowed(frames_passed: i32) -> bool {
        frames_passed > ATTACKER_RESTRICTED_FRAMES
    }

    pub fn get_minute(frames_passed: i32) -> i32 {
        frames_passed * GAME_MINUTES_PER_FRAME
    }

    pub fn is_hour(frames_passed: i32) -> bool {
        Self::get_minute(frames_passed) % 60 == 0
    }

    pub fn get_hour(frames_passed: i32) -> i32 {
        GAME_START_HOUR + Self::get_minute(frames_passed) / 60
    }

    pub fn get_emps_used(&self) -> i32 {
        self.attacker.emps_used as i32
    }

    pub fn get_is_attacker_alive(&self) -> bool {
        self.attacker.is_alive
    }

    pub fn get_no_of_robots_destroyed(&self) -> i32 {
        let mut destroyed = 0;
        for r in self.robots_manager.robots.iter() {
            if r.1.health == 0 {
                destroyed += 1;
            }
        }
        destroyed
    }

    pub fn get_damage_done(&self) -> i32 {
        let mut sum_health = 0;
        for r in self.robots_manager.robots.iter() {
            sum_health += r.1.health;
        }
        HEALTH * self.no_of_robots - sum_health
    }

    pub fn get_scores(&self) -> (i32, i32) {
        let damage_done = self.get_damage_done();
        let no_of_robots_destroyed = self.get_no_of_robots_destroyed();
        let max_score = 2 * HEALTH * self.no_of_robots;
        let attack_score = damage_done + HEALTH * no_of_robots_destroyed;
        let defend_score = max_score - attack_score;
        (attack_score, defend_score)
    }

    pub fn simulate(&mut self) -> Result<RenderSimulation> {
        let Simulator {
            buildings_manager,
            robots_manager,
            attacker,
            emps,
            frames_passed,
            ..
        } = self;
        *frames_passed += 1;

        let frames_passed = *frames_passed;

        robots_manager.move_robots(buildings_manager)?;

        let minute = Self::get_minute(frames_passed);
        emps.simulate(minute, robots_manager, buildings_manager, attacker)?;

        if Self::attacker_allowed(frames_passed) {
            attacker.update_position();
        }
        if Self::is_hour(frames_passed) {
            buildings_manager.update_building_weights(Self::get_hour(frames_passed))?;
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
        let (x_position, y_position) = attacker.get_current_position()?;
        let render_attacker = RenderAttacker {
            x_position,
            y_position,
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
        Ok(RenderSimulation {
            attacker: render_attacker,
            robots: render_robots,
        })
    }
}
