use crate::api;
use crate::api::error::AuthError;
use crate::api::game::util::UserDetail;
use crate::api::user::util::fetch_user;
use crate::api::util::{
    GameHistoryEntry, GameHistoryResponse, HistoryboardEntry, HistoryboardResponse,
};
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{
    AttackerType, BlockCategory, Game, LevelsFixture, MapLayout, NewAttackerPath, NewGame,
    NewSimulationLog, User,
};
use crate::schema::user;
use crate::simulation::{RenderAttacker, RenderMine};
use crate::simulation::{RenderDefender, Simulator};
use crate::util::function;
use crate::validator::state::Coords;
use anyhow::{Context, Result};
use chrono::Local;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::io::Write;

#[derive(Debug, Serialize)]
pub struct DefensePosition {
    pub y_coord: i32,
    pub x_coord: i32,
    pub block_category: BlockCategory,
}

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

#[derive(Deserialize)]
pub struct BombType {
    pub id: i32,
    pub bomb_x: i32,
    pub bomb_y: i32,
}

#[derive(Deserialize)]
pub struct FrameDetails {
    pub frame_no: i32,
    pub attacker_delta: Coords,
    pub attacker_type: i32,
    pub bombs_placed: Vec<BombType>,
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
    use crate::schema::{block_type, map_spaces};
    let valid_road_paths: HashSet<(i32, i32)> = map_spaces::table
        .inner_join(block_type::table)
        .filter(map_spaces::map_id.eq(map_id))
        .filter(block_type::building_type.eq(ROAD_ID))
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
        artifacts_collected: &0,
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
    user_id: i32,
    page: i64,
    limit: i64,
    conn: &mut PgConnection,
) -> Result<HistoryboardResponse> {
    use crate::schema::{game, levels_fixture, map_layout};
    let joined_table = game::table
        .filter(game::attack_id.eq(user_id))
        .inner_join(map_layout::table.inner_join(levels_fixture::table));

    let total_entries: i64 = joined_table
        .count()
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?;
    let off_set: i64 = (page - 1) * limit;
    let last_page: i64 = (total_entries as f64 / limit as f64).ceil() as i64;

    let games_result: Result<Vec<HistoryboardEntry>> = joined_table
        .offset(off_set)
        .limit(limit)
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            Ok(HistoryboardEntry {
                opponent_user_id: game.defend_id,
                is_attack: true,
                damage_percent: game.damage_done,
                artifacts_taken: game.artifacts_collected,
                trophies_taken: game.attack_score,
                match_id: game.id,
                replay_availability: is_replay_available,
            })
        })
        .collect();
    let games = games_result?;
    Ok(HistoryboardResponse { games, last_page })
}

pub fn fetch_top_attacks(user_id: i32, conn: &mut PgConnection) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table
        .inner_join(map_layout::table.inner_join(levels_fixture::table))
        .inner_join(user::table.on(game::defend_id.eq(user::id)));
    let games_result: Result<Vec<GameHistoryEntry>> = joined_table
        .order_by(game::attack_score.desc())
        .limit(10)
        .load::<(Game, (MapLayout, LevelsFixture), User)>(conn)?
        .into_iter()
        .map(|(game, (_, levels_fixture), defender)| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            let attacker = fetch_user(conn, game.attack_id)?.ok_or(AuthError::UserNotFound)?;
            Ok(GameHistoryEntry {
                game,
                attacker: UserDetail {
                    user_id: attacker.id,
                    username: attacker.username,
                    trophies: attacker.trophies,
                    avatar_id: attacker.avatar_id,
                },
                defender: UserDetail {
                    user_id: defender.id,
                    username: defender.username,
                    trophies: defender.trophies,
                    avatar_id: defender.avatar_id,
                },
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
    map_id: i32,
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
        writeln!(
            content,
            "{},{},{},{},{}",
            attacker_id + 1,
            attacker_path[0].y_coord,
            attacker_path[0].x_coord,
            attacker_path[0].is_emp,
            attacker_type,
        )?;
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
        Simulator::new(map_id, &attackers, conn).with_context(|| "Failed to create simulator")?;

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
        writeln!(content, "{defender_id},{x_position},{y_position}")?;
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
        writeln!(content, "mine {mine_id}")?;
        writeln!(content, "id,x,is_activated,y,mine_type")?;
        writeln!(
            content,
            "{mine_id},{x_position},{is_activated},{y_position},{mine_type}"
        )?;
    }

    for frame in 1..=NO_OF_FRAMES {
        writeln!(content, "frame {frame}")?;
        let simulated_frame = simulator
            .simulate()
            .with_context(|| format!("Failed to simulate frame {frame}"))?;
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
                    "{attacker_id},{x_position},{y_position},{is_alive},{emp_id},{health},{attacker_type}"
                )?;
            }
        }
        writeln!(content, "building_stats")?;
        writeln!(content, "map_space_id,population")?;

        for building_stat in simulated_frame.buildings {
            writeln!(
                content,
                "{},{}",
                building_stat.mapsace_id, building_stat.population
            )?;
        }

        for (defender_id, defender) in simulated_frame.defenders {
            writeln!(content, "defender {defender_id}")?;
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
                    "{defender_id},{is_alive},{x_position},{y_position},{defender_type}"
                )?;
            }
        }

        for (mine_id, mine) in simulated_frame.mines {
            writeln!(content, "mine {mine_id}")?;
            writeln!(content, "id,is_activated,mine_type")?;
            writeln!(
                content,
                "{},{},{}",
                mine.mine_id, mine.is_activated, mine.mine_type,
            )?;
        }

        /*
        position of robots
         */
    }
    //TODO: Change is_alive to no_of_attackers_alive and emps_used too
    let (attack_score, defend_score) = simulator.get_scores();
    let attack_defence_metrics = simulator.get_attack_defence_metrics();
    let (attacker_rating, defender_rating, attacker_rating_change, defender_rating_change) =
        diesel::update(game::table.find(game_id))
            .set((
                game::damage_done.eq(simulator.get_damage_done()),
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
            .update_rating(attack_defence_metrics, conn)
            .map_err(|err| DieselError {
                table: "user",
                function: function!(),
                error: err,
            })?;
    let damage = simulator.get_damage_done();
    writeln!(content, "Result")?;
    writeln!(content, "Damage: {damage}")?;
    writeln!(content, "New attacker rating: {attacker_rating}")?;
    writeln!(content, "New defender rating: {defender_rating}")?;
    writeln!(content, "Attacker rating change: {attacker_rating_change}")?;
    writeln!(content, "Defender rating change: {defender_rating_change}")?;

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

pub fn run_test_base_simulation(
    map_id: i32,
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
        writeln!(
            content,
            "{},{},{},{},{}",
            attacker_id + 1,
            attacker_path[0].y_coord,
            attacker_path[0].x_coord,
            attacker_path[0].is_emp,
            attacker_type,
        )?;
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

    let mut simulator =
        Simulator::new(map_id, &attackers, conn).with_context(|| "Failed to create simulator")?;

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
        writeln!(content, "{defender_id},{x_position},{y_position}")?;
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
        writeln!(content, "mine {mine_id}")?;
        writeln!(content, "id,x,is_activated,y,mine_type")?;
        writeln!(
            content,
            "{mine_id},{x_position},{is_activated},{y_position},{mine_type}"
        )?;
    }

    for frame in 1..=NO_OF_FRAMES {
        writeln!(content, "frame {frame}")?;
        let simulated_frame = simulator
            .simulate()
            .with_context(|| format!("Failed to simulate frame {frame}"))?;
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
                    "{attacker_id},{x_position},{y_position},{is_alive},{emp_id},{health},{attacker_type}"
                )?;
            }
        }
        writeln!(content, "building_stats")?;
        writeln!(content, "map_space_id,population")?;

        for building_stat in simulated_frame.buildings {
            writeln!(
                content,
                "{},{}",
                building_stat.mapsace_id, building_stat.population
            )?;
        }

        for (defender_id, defender) in simulated_frame.defenders {
            writeln!(content, "defender {defender_id}")?;
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
                    "{defender_id},{is_alive},{x_position},{y_position},{defender_type}"
                )?;
            }
        }

        for (mine_id, mine) in simulated_frame.mines {
            writeln!(content, "mine {mine_id}")?;
            writeln!(content, "id,is_activated,mine_type")?;
            writeln!(
                content,
                "{},{},{}",
                mine.mine_id, mine.is_activated, mine.mine_type,
            )?;
        }

        /*
        position of robots
         */
    }
    //TODO: Change is_alive to no_of_attackers_alive and emps_used too
    let damage = simulator.get_damage_done();
    writeln!(content, "Result")?;
    writeln!(content, "Damage: {damage}")?;

    Ok(content)
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
                    name: attacker.name.clone(),
                    max_health: attacker.max_health,
                    speed: attacker.speed,
                    amt_of_emps: attacker.amt_of_emps,
                    level: attacker.level,
                    cost: attacker.cost,
                },
            )
        })
        .collect::<HashMap<i32, AttackerType>>())
}
