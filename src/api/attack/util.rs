use crate::api::util::{can_show_replay, GameHistoryEntry, GameHistoryResponse};
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{
    Game, LevelsFixture, MapLayout, NewAttackerPath, NewGame, NewSimulationLog, SimulationLog,
};
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
    pub attacker_path: Vec<NewPath>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct NewPath {
    pub y_coord: i32,
    pub x_coord: i32,
    pub is_emp: bool,
    pub emp_type: Option<i32>,
    pub emp_time: Option<i32>,
}

#[derive(Deserialize)]
pub struct LeaderboardQuery {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}

#[derive(Deserialize, Serialize)]
pub struct LeaderboardResponse {
    pub leaderboard_entries: Vec<LeaderboardEntry>,
    pub last_page: i64,
}

#[derive(Queryable, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub username: String,
    pub overall_rating: i32,
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
    Ok(total_attacks_this_level < TOTAL_ATTACKS_PER_LEVEL && !is_duplicate_attack)
}

pub fn insert_attack(
    attacker_id: i32,
    new_attack: &NewAttack,
    map_layout_id: i32,
    conn: &PgConnection,
) -> Result<i32> {
    use crate::schema::{attacker_path, game};

    // insert in game table

    let new_game = NewGame {
        attack_id: &attacker_id,
        defend_id: &new_attack.defender_id,
        map_layout_id: &map_layout_id,
        attack_score: &0,
        defend_score: &0,
        is_attacker_alive: &false,
        robots_destroyed: &0,
        damage_done: &0,
        emps_used: &0,
    };

    let inserted_game: Game = diesel::insert_into(game::table)
        .values(&new_game)
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?;

    // insert in attacker path table

    let new_attacker_paths: Vec<NewAttackerPath> = new_attack
        .attacker_path
        .iter()
        .enumerate()
        .map(|(id, path)| (id as i32, path))
        .map(|(id, path)| NewAttackerPath {
            id,
            y_coord: &path.y_coord,
            x_coord: &path.x_coord,
            is_emp: &path.is_emp,
            game_id: &inserted_game.id,
            emp_type: path.emp_type.as_ref(),
            emp_time: path.emp_time.as_ref(),
        })
        .collect();

    diesel::insert_into(attacker_path::table)
        .values(new_attacker_paths)
        .execute(conn)
        .map_err(|err| DieselError {
            table: "attacker_path",
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

    let current_date = Local::now().naive_local().date();

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games = joined_table
        .filter(game::attack_id.eq(attacker_id))
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available =
                can_show_replay(user_id, &game, &levels_fixture, current_date);
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

    let current_date = Local::now().naive_local().date();

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games = joined_table
        .order_by(game::attack_score.desc())
        .limit(10)
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available =
                can_show_replay(user_id, &game, &levels_fixture, current_date);
            GameHistoryEntry {
                game,
                is_replay_available,
            }
        })
        .collect();
    Ok(GameHistoryResponse { games })
}

pub fn get_leaderboard(page: i64, limit: i64, conn: &PgConnection) -> Result<LeaderboardResponse> {
    use crate::schema::user;
    let total_entries: i64 = user::table
        .count()
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    let offset: i64 = (page - 1) * limit;
    let last_page: i64 = (total_entries as f64 / limit as f64).ceil() as i64;

    let leaderboard_entries = user::table
        .select((user::username, user::overall_rating))
        .order_by(user::overall_rating.desc())
        .offset(offset)
        .limit(limit)
        .load::<LeaderboardEntry>(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;

    Ok(LeaderboardResponse {
        leaderboard_entries,
        last_page,
    })
}

pub fn run_simulation(game_id: i32, conn: &PgConnection) -> Result<Vec<u8>> {
    use crate::schema::game;
    let mut simulator =
        Simulator::new(game_id, conn).with_context(|| "Failed to create simulator")?;
    let mut content = Vec::new();

    writeln!(content, "emps")?;
    writeln!(content, "id,time,type")?;
    let emps = simulator.render_emps();
    for emp in emps {
        writeln!(content, "{},{},{}", emp.id, emp.time, emp.emp_type)?;
    }

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
    diesel::update(game::table.find(game_id))
        .set((
            game::damage_done.eq(simulator.get_damage_done()),
            game::robots_destroyed.eq(simulator.get_no_of_robots_destroyed()),
            game::is_attacker_alive.eq(simulator.get_is_attacker_alive()),
            game::emps_used.eq(simulator.get_emps_used()),
        ))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "game",
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

pub fn fetch_is_replay_allowed(game_id: i32, user_id: i32, conn: &PgConnection) -> bool {
    use crate::schema::{game, levels_fixture, map_layout};

    let current_date = Local::now().naive_local().date();

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let result = joined_table
        .filter(game::id.eq(game_id))
        .first::<(Game, (MapLayout, LevelsFixture))>(conn);

    if let Ok((game, (_, fixture))) = result {
        return can_show_replay(user_id, &game, &fixture, current_date);
    }

    false
}

pub fn fetch_replay(game_id: i32, conn: &PgConnection) -> Result<SimulationLog> {
    use crate::schema::simulation_log;
    Ok(simulation_log::table
        .filter(simulation_log::game_id.eq(game_id))
        .first(conn)
        .map_err(|err| DieselError {
            table: "simulation_log",
            function: function!(),
            error: err,
        })?)
}
