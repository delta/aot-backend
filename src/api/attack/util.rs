use crate::api::util::{can_show_replay, GameHistoryEntry, GameHistoryResponse};
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{Game, LevelsFixture, MapLayout, NewAttackerPath, NewGame, NewSimulationLog};
use crate::simulation::RenderRobot;
use crate::simulation::{RenderAttacker, Simulator};
use crate::util::function;
use anyhow::{Context, Result};
use chrono::{Local, NaiveTime};
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewAttack {
    pub defender_id: i32,
    pub attacker_path: Vec<NewAttackerPath>,
}

/// checks if the attack is allowed at current time
pub fn is_attack_allowed_now() -> bool {
    let start_time = NaiveTime::from_hms(START_HOUR as u32, 0, 0);
    let end_time = NaiveTime::from_hms(END_HOUR, 0, 0);
    let current_time = Local::now().naive_local().time();
    current_time >= start_time && current_time <= end_time
}

pub fn get_valid_emp_ids(conn: &PgConnection) -> Result<HashSet<i32>> {
    use crate::schema::attack_type;
    let valid_emp_ids = HashSet::from_iter(attack_type::table.select(attack_type::id).load(conn)?);
    Ok(valid_emp_ids)
}

pub fn get_current_levels_fixture(conn: &PgConnection) -> Result<LevelsFixture> {
    use crate::schema::levels_fixture;
    let current_date = Local::now().naive_local().date();
    let level: LevelsFixture = levels_fixture::table
        .filter(levels_fixture::start_date.le(current_date))
        .filter(levels_fixture::end_date.gt(current_date))
        .first(conn)
        .map_err(|err| DieselError {
            table: "levels_fixture",
            function: function!(),
            error: err,
        })?;
    Ok(level)
}

pub fn get_map_id(defender_id: &i32, level_id: &i32, conn: &PgConnection) -> Result<i32> {
    use crate::schema::map_layout;
    let map_id: i32 = map_layout::table
        .filter(map_layout::player.eq(defender_id))
        .filter(map_layout::level_id.eq(level_id))
        .select(map_layout::id)
        .first(conn)
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;
    Ok(map_id)
}

pub fn get_valid_road_paths(map_id: i32, conn: &PgConnection) -> Result<HashSet<(i32, i32)>> {
    use crate::schema::map_spaces;
    let valid_road_paths: HashSet<(i32, i32)> = map_spaces::table
        .filter(map_spaces::map_id.eq(map_id))
        .filter(map_spaces::blk_type.eq(ROAD_ID))
        .select((map_spaces::x_coordinate, map_spaces::y_coordinate))
        .load::<(i32, i32)>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?
        .iter()
        .cloned()
        .collect();
    Ok(valid_road_paths)
}

/// checks if the number of attacks per day is less than allowed for the given attacker
pub fn is_attack_allowed(attacker_id: i32, defender_id: i32, conn: &PgConnection) -> Result<bool> {
    let current_date = Local::now().naive_local().date();
    use crate::schema::{game, levels_fixture, map_layout};
    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let total_attacks_this_level: i64 = joined_table
        .filter(game::attack_id.eq(attacker_id))
        .filter(levels_fixture::start_date.le(current_date))
        .filter(levels_fixture::end_date.gt(current_date))
        .count()
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "joined_table",
            function: function!(),
            error: err,
        })?;
    let total_attacks_on_a_base: i64 = joined_table
        .filter(game::attack_id.eq(defender_id))
        .filter(levels_fixture::start_date.le(current_date))
        .filter(levels_fixture::end_date.gt(current_date))
        .count()
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "joined_table",
            function: function!(),
            error: err,
        })?;
    let is_duplicate_attack: bool = select(exists(
        joined_table
            .filter(game::attack_id.eq(attacker_id))
            .filter(game::defend_id.eq(defender_id))
            .filter(levels_fixture::start_date.le(current_date))
            .filter(levels_fixture::end_date.gt(current_date)),
    ))
    .get_result(conn)
    .map_err(|err| DieselError {
        table: "joined_table",
        function: function!(),
        error: err,
    })?;
    Ok(total_attacks_this_level < TOTAL_ATTACKS_PER_LEVEL
        && total_attacks_on_a_base < TOTAL_ATTACKS_ON_A_BASE
        && !is_duplicate_attack)
}

pub fn add_game(
    attacker_id: i32,
    new_attack: &NewAttack,
    map_layout_id: i32,
    conn: &PgConnection,
) -> Result<i32> {
    use crate::schema::game;

    // insert in game table

    let new_game = NewGame {
        attack_id: &attacker_id,
        defend_id: &new_attack.defender_id,
        map_layout_id: &map_layout_id,
        attack_score: &0,
        defend_score: &0,
        robots_destroyed: &0,
        damage_done: &0,
        emps_used: &0,
        is_attacker_alive: &false,
    };

    let inserted_game: Game = diesel::insert_into(game::table)
        .values(&new_game)
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?;

    Ok(inserted_game.id)
}

pub fn fetch_attack_history(
    attacker_id: i32,
    user_id: i32,
    conn: &PgConnection,
) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games = joined_table
        .filter(game::attack_id.eq(attacker_id))
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = can_show_replay(user_id, &game, &levels_fixture);
            GameHistoryEntry {
                game,
                is_replay_available,
            }
        })
        .collect();
    Ok(GameHistoryResponse { games })
}

pub fn fetch_top_attacks(user_id: i32, conn: &PgConnection) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games = joined_table
        .order_by(game::attack_score.desc())
        .limit(10)
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = can_show_replay(user_id, &game, &levels_fixture);
            GameHistoryEntry {
                game,
                is_replay_available,
            }
        })
        .collect();
    Ok(GameHistoryResponse { games })
}

pub fn run_simulation(
    game_id: i32,
    attacker_path: Vec<NewAttackerPath>,
    conn: &PgConnection,
) -> Result<Vec<u8>> {
    let mut content = Vec::new();
    writeln!(content, "attacker_path")?;
    writeln!(content, "id,y,x,is_emp")?;
    attacker_path
        .iter()
        .enumerate()
        .try_for_each(|(id, path)| {
            writeln!(
                content,
                "{},{},{},{}",
                id, path.y_coord, path.x_coord, path.is_emp
            )
        })?;

    use crate::schema::game;
    let mut simulator = Simulator::new(game_id, &attacker_path, conn)
        .with_context(|| "Failed to create simulator")?;

    writeln!(content, "emps")?;
    writeln!(content, "id,time,type")?;
    attacker_path
        .iter()
        .enumerate()
        .try_for_each(|(id, path)| {
            if path.is_emp {
                writeln!(
                    content,
                    "{},{},{}",
                    id,
                    path.emp_time.unwrap(),
                    path.emp_type.unwrap()
                )
            } else {
                Ok(())
            }
        })?;

    for frame in 1..=NO_OF_FRAMES {
        writeln!(content, "frame {}", frame)?;
        let simulated_frame = simulator
            .simulate()
            .with_context(|| format!("Failed to simulate frame {}", frame))?;

        writeln!(content, "attacker")?;
        writeln!(content, "x,y,is_alive,emp_id")?;
        let RenderAttacker {
            x_position,
            y_position,
            is_alive,
            emp_id,
        } = simulated_frame.attacker;
        writeln!(
            content,
            "{},{},{},{}",
            x_position, y_position, is_alive, emp_id
        )?;

        writeln!(content, "robots")?;
        writeln!(content, "id,health,x,y,in_building")?;
        for robot in simulated_frame.robots {
            let RenderRobot {
                id,
                health,
                x_position,
                y_position,
                in_building,
            } = robot;
            writeln!(
                content,
                "{},{},{},{},{}",
                id, health, x_position, y_position, in_building
            )?;
        }
    }
    let (attack_score, defend_score) = simulator.get_scores();
    diesel::update(game::table.find(game_id))
        .set((
            game::damage_done.eq(simulator.get_damage_done()),
            game::robots_destroyed.eq(simulator.get_no_of_robots_destroyed()),
            game::is_attacker_alive.eq(simulator.get_is_attacker_alive()),
            game::emps_used.eq(simulator.get_emps_used()),
            game::attack_score.eq(attack_score),
            game::defend_score.eq(defend_score),
        ))
        .get_result::<Game>(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?
        .update_rating(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;

    insert_simulation_log(game_id, &content, conn)?;

    Ok(content)
}

pub fn insert_simulation_log(game_id: i32, content: &[u8], conn: &PgConnection) -> Result<()> {
    use crate::schema::simulation_log;
    let log_text = String::from_utf8(content.to_vec())?;
    let new_simulation_log = NewSimulationLog {
        game_id: &game_id,
        log_text: &log_text,
    };
    diesel::insert_into(simulation_log::table)
        .values(new_simulation_log)
        .execute(conn)
        .map_err(|err| DieselError {
            table: "simulation_log",
            function: function!(),
            error: err,
        })?;
    Ok(())
}
