use crate::error::DieselError;
use crate::util::function;
use crate::{models::AttackerPath, simulation::error::EmpDetailsError};
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

const GAME_TIME_MINUTES: i32 = 420;
pub const GAME_MINUTES_PER_FRAME: i32 = 2;
const ATTACKER_RESTRICTED_FRAMES: i32 = 30;
const START_HOUR: i32 = 9;
pub const NO_OF_FRAMES: i32 = GAME_TIME_MINUTES / GAME_MINUTES_PER_FRAME;

#[derive(Debug, Serialize)]
pub struct RenderAttacker {
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub emp_id: i32,
}

#[derive(Clone, Debug)]
pub struct RenderEmp {
    pub id: i32,
    pub time: i32,
    pub emp_type: i32,
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
    render_emps: Vec<RenderEmp>,
}

impl Simulator {
    pub fn new(game_id: i32, conn: &PgConnection) -> Result<Self> {
        use crate::schema::{attacker_path, game};

        let map_id = game::table
            .filter(game::id.eq(game_id))
            .select(game::map_layout_id)
            .first::<i32>(conn)
            .map_err(|err| DieselError {
                table: "game",
                function: function!(),
                error: err,
            })?;

        let buildings_manager = BuildingsManager::new(conn, map_id)?;
        let robots_manager = RobotsManager::new(&buildings_manager)?;
        let attacker = Attacker::new(conn, game_id)?;
        let emps = Emps::new(conn, game_id)?;
        let render_emps: Result<Vec<RenderEmp>> = attacker_path::table
            .filter(attacker_path::game_id.eq(game_id))
            .filter(attacker_path::is_emp.eq(true))
            .load::<AttackerPath>(conn)
            .map_err(|err| DieselError {
                table: "attacker_path",
                function: function!(),
                error: err,
            })?
            .iter()
            .map(|path| {
                if let (Some(emp_type), Some(time)) = (path.emp_type, path.emp_time) {
                    Ok(RenderEmp {
                        id: path.id,
                        time,
                        emp_type,
                    })
                } else {
                    Err(EmpDetailsError { path_id: path.id }.into())
                }
            })
            .collect();
        let render_emps = render_emps?;

        Ok(Simulator {
            buildings_manager,
            robots_manager,
            attacker,
            emps,
            frames_passed: 0,
            render_emps,
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
        START_HOUR + Self::get_minute(frames_passed) / 60
    }

    pub fn render_emps(&self) -> Vec<RenderEmp> {
        self.render_emps.clone()
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
        buildings_manager.revive_buildings(minute);

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
