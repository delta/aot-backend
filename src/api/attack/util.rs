use crate::api;
use crate::api::util::{GameHistoryEntry, GameHistoryResponse};
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{
    AttackerType, Game, LevelsFixture, MapLayout, NewAttackerPath, NewGame, NewSimulationLog,
};
use crate::simulation::{RenderAttacker, RenderDiffuser, RenderMine, RenderRobot};
use crate::simulation::{RenderDefender, Simulator};
use crate::util::function;
use anyhow::{Context, Result};
use chrono::{Local, NaiveTime};
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;

#[derive(Debug, Deserialize, Serialize)]
pub struct NewAttack {
    pub defender_id: i32,
    pub no_of_attackers: i32,
    pub attackers: Vec<NewAttacker>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NewAttacker {
    pub attacker_type: i32,
    pub attacker_path: Vec<NewAttackerPath>,
}

/// checks if the attack is allowed at current time
pub fn is_attack_allowed_now() -> bool {
    let start_time = NaiveTime::parse_from_str(ATTACK_START_TIME, "%H:%M:%S").unwrap();
    let end_time = NaiveTime::parse_from_str(ATTACK_END_TIME, "%H:%M:%S").unwrap();
    let current_time = Local::now().naive_local().time();
    current_time >= start_time && current_time <= end_time
}

pub fn get_valid_emp_ids(conn: &mut PgConnection) -> Result<HashSet<i32>> {
    use crate::schema::attack_type;
    let valid_emp_ids = HashSet::from_iter(attack_type::table.select(attack_type::id).load(conn)?);
    Ok(valid_emp_ids)
}

pub fn get_map_id(
    defender_id: &i32,
    level_id: &i32,
    conn: &mut PgConnection,
) -> Result<Option<i32>> {
    use crate::schema::map_layout;
    let map_id = map_layout::table
        .filter(map_layout::player.eq(defender_id))
        .filter(map_layout::level_id.eq(level_id))
        .filter(map_layout::is_valid.eq(true))
        .select(map_layout::id)
        .first::<i32>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;
    Ok(map_id)
}

pub fn get_valid_road_paths(map_id: i32, conn: &mut PgConnection) -> Result<HashSet<(i32, i32)>> {
    use crate::schema::{building_type, map_spaces};
    let valid_road_paths: HashSet<(i32, i32)> = map_spaces::table
        .inner_join(building_type::table)
        .filter(map_spaces::map_id.eq(map_id))
        .filter(building_type::blk_type.eq(ROAD_ID))
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
pub fn is_attack_allowed(
    attacker_id: i32,
    defender_id: i32,
    conn: &mut PgConnection,
) -> Result<bool> {
    let current_date = Local::now().naive_local();
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
    let map_layout_join_levels_fixture = map_layout::table.inner_join(levels_fixture::table);
    let attacker: Option<i32> = map_layout_join_levels_fixture
        .filter(map_layout::player.eq(attacker_id))
        .filter(levels_fixture::start_date.le(current_date))
        .filter(levels_fixture::end_date.gt(current_date))
        .filter(map_layout::is_valid.eq(true))
        .select(map_layout::player)
        .first(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;
    let is_self_attack = attacker_id == defender_id;
    Ok(total_attacks_this_level < TOTAL_ATTACKS_PER_LEVEL
        && total_attacks_on_a_base < TOTAL_ATTACKS_ON_A_BASE
        && !is_duplicate_attack
        && !is_self_attack
        && attacker.is_some())
}

pub fn add_game(
    attacker_id: i32,
    new_attack: &NewAttack,
    map_layout_id: i32,
    conn: &mut PgConnection,
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
    conn: &mut PgConnection,
) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games_result: Result<Vec<GameHistoryEntry>> = joined_table
        .filter(game::attack_id.eq(attacker_id))
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            let player_name = api::util::get_username(game.defend_id, conn)?;
            Ok(GameHistoryEntry {
                game,
                player_name,
                is_replay_available,
            })
        })
        .collect();
    let games = games_result?;
    Ok(GameHistoryResponse { games })
}

pub fn fetch_top_attacks(user_id: i32, conn: &mut PgConnection) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games_result: Result<Vec<GameHistoryEntry>> = joined_table
        .order_by(game::attack_score.desc())
        .limit(10)
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            let player_name = api::util::get_username(game.attack_id, conn)?;
            Ok(GameHistoryEntry {
                game,
                player_name,
                is_replay_available,
            })
        })
        .collect();
    let games = games_result?;
    Ok(GameHistoryResponse { games })
}

pub fn remove_game(game_id: i32, conn: &mut PgConnection) -> Result<()> {
    use crate::schema::game;

    diesel::delete(game::table.filter(game::id.eq(game_id)))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?;
    Ok(())
}

pub fn run_simulation(
    game_id: i32,
    attackers: Vec<NewAttacker>,
    conn: &mut PgConnection,
) -> Result<Vec<u8>> {
    let mut content = Vec::new();

    for (attacker_id, attacker) in attackers.iter().enumerate() {
        writeln!(content, "attacker {}", attacker_id + 1)?;
        let attacker_path = &attacker.attacker_path;
        let attacker_type = &attacker.attacker_type;
        writeln!(content, "attacker_path")?;
        writeln!(content, "id,y,x,is_emp,type")?;
        attacker_path
            .iter()
            .enumerate()
            .try_for_each(|(id, path)| {
                writeln!(
                    content,
                    "{},{},{},{},{}",
                    id + 1,
                    path.y_coord,
                    path.x_coord,
                    path.is_emp,
                    attacker_type,
                )
            })?;
        writeln!(content, "emps")?;
        writeln!(content, "id,time,type,attacker_id")?;
        attacker_path
            .iter()
            .enumerate()
            .try_for_each(|(id, path)| {
                if path.is_emp {
                    writeln!(
                        content,
                        "{},{},{},{}",
                        id + 1,
                        path.emp_time.unwrap(),
                        path.emp_type.unwrap(),
                        attacker_id + 1,
                    )
                } else {
                    Ok(())
                }
            })?;
    }

    use crate::schema::game;
    let mut simulator =
        Simulator::new(game_id, &attackers, conn).with_context(|| "Failed to create simulator")?;

    let defenders_positions = simulator.get_defender_position();

    for position in defenders_positions {
        writeln!(content, "defender {}", position.defender_id)?;
        writeln!(content, "id,x,y")?;
        let RenderDefender {
            defender_id,
            x_position,
            y_position,
            ..
        } = position;
        writeln!(content, "{},{},{}", defender_id, x_position, y_position)?;
    }

    let diffuser_positions = simulator.get_diffuser_position();

    for position in diffuser_positions {
        let RenderDiffuser {
            diffuser_id,
            x_position,
            y_position,
            is_alive,
            ..
        } = position;
        writeln!(content, "diffuser {}", diffuser_id)?;
        writeln!(content, "id,is_alive,x,y")?;
        writeln!(
            content,
            "{},{},{},{}",
            diffuser_id, is_alive, x_position, y_position,
        )?;
    }

    let mines = simulator.get_mines();

    for mine in mines {
        let RenderMine {
            mine_id,
            x_position,
            y_position,
            is_activated,
            mine_type,
        } = mine;
        writeln!(content, "mine {}", mine_id)?;
        writeln!(content, "id,is_activated,x,y,mine_type")?;
        writeln!(
            content,
            "{},{},{},{},{}",
            mine_id, is_activated, x_position, y_position, mine_type
        )?;
    }

    for frame in 1..=NO_OF_FRAMES {
        writeln!(content, "frame {}", frame)?;
        let simulated_frame = simulator
            .simulate()
            .with_context(|| format!("Failed to simulate frame {}", frame))?;
        for attacker in simulated_frame.attackers {
            writeln!(content, "attacker {}", attacker.0)?;
            writeln!(content, "id,x,y,is_alive,emp_id,health,type")?;
            for position in attacker.1 {
                let RenderAttacker {
                    x_position,
                    y_position,
                    is_alive,
                    emp_id,
                    health,
                    attacker_type,
                    attacker_id,
                } = position;
                writeln!(
                    content,
                    "{},{},{},{},{},{},{}",
                    attacker_id, x_position, y_position, is_alive, emp_id, health, attacker_type
                )?;
            }
        }

        for (defender_id, defender) in simulated_frame.defenders {
            writeln!(content, "defender {}", defender_id)?;
            writeln!(content, "id,is_alive,x,y,type")?;
            for position in defender {
                let RenderDefender {
                    defender_id,
                    x_position,
                    y_position,
                    defender_type,
                    is_alive,
                } = position;
                writeln!(
                    content,
                    "{},{},{},{},{}",
                    defender_id, is_alive, x_position, y_position, defender_type
                )?;
            }
        }

        for (diffuser_id, diffuser) in simulated_frame.diffusers {
            writeln!(content, "diffuser {}", diffuser_id)?;
            writeln!(content, "id,is_alive,x,y,type,emp_id,attacker_id")?;
            for defender_position in diffuser {
                let RenderDiffuser {
                    diffuser_id,
                    x_position,
                    y_position,
                    is_alive,
                    diffuser_type,
                    emp_attacker_id,
                    emp_path_id,
                } = defender_position;
                writeln!(
                    content,
                    "{},{},{},{},{},{},{}",
                    diffuser_id,
                    is_alive,
                    x_position,
                    y_position,
                    diffuser_type,
                    emp_path_id,
                    emp_attacker_id
                )?;
            }
        }

        for (mine_id, mine) in simulated_frame.mines {
            writeln!(content, "mine {}", mine_id)?;
            writeln!(content, "id,is_activated,x,y,mine_type")?;
            writeln!(
                content,
                "{},{},{},{},{}",
                mine.mine_id, mine.is_activated, mine.x_position, mine.y_position, mine.mine_type,
            )?;
        }

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
    //TODO: Change is_alive to no_of_attackers_alive and emps_used too
    let (attack_score, defend_score) = simulator.get_scores();
    let (attacker_rating_change, defender_rating_change) =
        diesel::update(game::table.find(game_id))
            .set((
                game::damage_done.eq(simulator.get_damage_done()),
                game::robots_destroyed.eq(simulator.get_no_of_robots_destroyed()),
                game::is_attacker_alive.eq(true),
                game::emps_used.eq(1),
                game::attack_score.eq(attack_score),
                game::defend_score.eq(defend_score),
            ))
            .get_result::<Game>(conn)
            .map_err(|err| DieselError {
                table: "game",
                function: function!(),
                error: err,
            })?
            .update_rating(simulator.rating_factor, simulator.no_of_robots, conn)
            .map_err(|err| DieselError {
                table: "user",
                function: function!(),
                error: err,
            })?;
    writeln!(content, "Result")?;
    writeln!(content, "Attack score: {}", attack_score)?;
    writeln!(content, "Defend score: {}", defend_score)?;
    writeln!(
        content,
        "Attacker rating change: {}",
        attacker_rating_change
    )?;
    writeln!(
        content,
        "Defender rating change: {}",
        defender_rating_change
    )?;

    insert_simulation_log(game_id, &content, conn)?;

    Ok(content)
}

pub fn insert_simulation_log(game_id: i32, content: &[u8], conn: &mut PgConnection) -> Result<()> {
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

pub fn get_attacker_types(conn: &mut PgConnection) -> Result<HashMap<i32, AttackerType>> {
    use crate::schema::attacker_type::dsl::*;
    Ok(attacker_type
        .load::<AttackerType>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?
        .iter()
        .map(|attacker| {
            (
                attacker.id,
                AttackerType {
                    id: attacker.id,
                    max_health: attacker.max_health,
                    speed: attacker.speed,
                    amt_of_emps: attacker.amt_of_emps,
                },
            )
        })
        .collect::<HashMap<i32, AttackerType>>())
}
