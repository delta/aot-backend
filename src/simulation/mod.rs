use std::collections::HashMap;

use self::attack::AttackManager;
use self::defense::defender::Defenders;
use self::defense::mine::Mines;
use crate::api::attack::util::NewAttacker;
use crate::constants::*;
use crate::error::DieselError;
use crate::simulation::defense::DefenseManager;
use crate::util::function;
use anyhow::Result;
use blocks::BuildingsManager;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::Serialize;

pub mod attack;
pub mod blocks;
pub mod defense;
pub mod error;

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenderAttacker {
    pub attacker_id: i32,
    pub health: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub emp_id: usize,
    pub attacker_type: i32,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenderDefender {
    pub defender_id: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub defender_type: i32,
}

#[derive(Debug, Serialize, Clone, Copy)]
pub struct RenderMine {
    pub mine_id: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub mine_type: i32,
    pub is_activated: bool,
}

#[derive(Debug, Serialize)]
pub struct RenderSimulation {
    pub attackers: HashMap<i32, Vec<RenderAttacker>>,
    pub defenders: HashMap<i32, Vec<RenderDefender>>,
    pub mines: HashMap<i32, RenderMine>,
    pub buildings: Vec<BuildingStats>,
}

#[derive(Debug, Serialize)]
pub struct BuildingStats {
    pub mapsace_id: i32,
    pub population: i32,
}

pub struct Simulator {
    buildings_manager: BuildingsManager,
    attack_manager: AttackManager,
    frames_passed: i32,
    defense_manager: DefenseManager,
    pub rating_factor: f32,
}

impl Simulator {
    pub fn new(map_id: i32, attackers: &Vec<NewAttacker>, conn: &mut PgConnection) -> Result<Self> {
        use crate::schema::{levels_fixture, map_layout};

        let rating_factor = map_layout::table
            .inner_join(levels_fixture::table)
            .select(levels_fixture::rating_factor)
            .filter(map_layout::id.eq(map_id))
            .first::<f32>(conn)
            .map_err(|err| DieselError {
                table: "map_layout levels_fixture",
                function: function!(),
                error: err,
            })?;

        let buildings_manager = BuildingsManager::new(conn, map_id)?;
        let attack_manager = AttackManager::new(conn, attackers)?;
        let defense_manager = DefenseManager::new(conn, map_id)?;

        Ok(Simulator {
            buildings_manager,
            attack_manager,
            frames_passed: 0,
            rating_factor,
            defense_manager,
        })
    }

    pub fn attacker_allowed(frames_passed: i32) -> bool {
        frames_passed > ATTACKER_RESTRICTED_FRAMES
    }

    pub fn get_minute(frames_passed: i32) -> i32 {
        frames_passed * GAME_MINUTES_PER_FRAME
    }

    pub fn get_damage_done(&self) -> i32 {
        /*

        **************************
        --------------------------
        function to compute damage
        --------------------------
        **************************

         */

        60
    }

    #[allow(dead_code)]
    pub fn get_attack_defence_metrics(&self) -> (i32, i32, i32) {
        let mut live_attackers = 0;
        let mut used_defenders = 0;
        let mut used_mines = 0;

        for a in self.attack_manager.attackers.values() {
            if a.is_alive {
                live_attackers += 1;
            }
        }
        let Defenders(defenders) = &self.defense_manager.defenders;
        for def in defenders {
            if def.damage_dealt {
                used_defenders += 1;
            }
        }
        let Mines(mines) = &self.defense_manager.mines;
        for min in mines {
            if !min.is_activated {
                used_mines += 1;
            }
        }

        (live_attackers, used_defenders, used_mines)
    }

    // return value (attack score, defence score)
    #[allow(dead_code)]
    pub fn get_scores(&self) -> (i32, i32) {
        let damage_done = self.get_damage_done();
        if damage_done < WIN_THRESHOLD {
            (damage_done - 100, 100 - damage_done)
        } else {
            (damage_done, -damage_done)
        }
    }

    pub fn get_defender_position(&self) -> Vec<RenderDefender> {
        self.defense_manager
            .defenders
            .get_defender_initial_position()
    }

    pub fn get_mines(&self) -> Vec<RenderMine> {
        self.defense_manager.mines.get_intial_mines()
    }

    pub fn simulate(&mut self) -> Result<RenderSimulation> {
        let Simulator {
            buildings_manager,
            attack_manager,
            frames_passed,
            defense_manager,
            ..
        } = self;
        *frames_passed += 1;

        let frames_passed = *frames_passed;

        //Simulate Emps and attackers
        attack_manager.simulate_attack(frames_passed, buildings_manager, defense_manager)?;

        defense_manager.simulate(attack_manager, buildings_manager, frames_passed)?;

        let render_attackers = attack_manager.get_attacker_positions()?;

        let render_defenders = defense_manager.defenders.post_simulate();

        let building_stats = buildings_manager.get_building_stats();

        let render_mines = defense_manager.mines.post_simulate();

        Ok(RenderSimulation {
            attackers: render_attackers,
            defenders: render_defenders,
            mines: render_mines,
            buildings: building_stats,
        })
    }
}
