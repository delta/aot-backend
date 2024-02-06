use crate::api::auth::TokenClaims;
use crate::api::defense::util::{fetch_map_layout, get_map_details_for_attack, DefenseResponse};
use crate::api::error::AuthError;
use crate::api::game::util::UserDetail;
use crate::api::user::util::fetch_user;
use crate::api::util::{
    GameHistoryEntry, GameHistoryResponse, HistoryboardEntry, HistoryboardResponse,
};
use crate::api::{self, RedisConn};
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{
    Artifact, AttackerType, BlockCategory, BlockType, BuildingType, DefenderType, Game,
    LevelsFixture, MapLayout, MapSpaces, MineType, NewAttackerPath, NewGame, NewSimulationLog,
    ShortestPath, User,
};
use crate::schema::user;
use crate::simulation::blocks::{Coords, SourceDest};
use crate::simulation::{RenderAttacker, RenderMine};
use crate::simulation::{RenderDefender, Simulator};
use crate::util::function;
use crate::validator::util::{BuildingDetails, Coordinates, DefenderDetails, MineDetails};
use anyhow::{Context, Result};
use chrono::{Duration, Local, Utc};
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::seq::IteratorRandom;
use redis::Commands;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::env;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AttackToken {
    pub attacker_id: i32,
    pub defender_id: i32,
    pub iat: usize,
    pub exp: usize,
}

pub fn get_valid_emp_ids(conn: &mut PgConnection) -> Result<HashSet<i32>> {
    use crate::schema::attack_type;
    let valid_emp_ids = HashSet::from_iter(attack_type::table.select(attack_type::id).load(conn)?);
    Ok(valid_emp_ids)
}

pub fn get_map_id(defender_id: &i32, conn: &mut PgConnection) -> Result<Option<i32>> {
    use crate::schema::map_layout;
    let map_id = map_layout::table
        .filter(map_layout::player.eq(defender_id))
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
    defender_id: i32,
    map_layout_id: i32,
    conn: &mut PgConnection,
) -> Result<i32> {
    use crate::schema::game;

    // insert in game table

    let new_game = NewGame {
        attack_id: &attacker_id,
        defend_id: &defender_id,
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

#[derive(Serialize)]
pub struct AttackResponse {
    pub base: DefenseResponse,
    pub shortest_paths: HashMap<SourceDest, Coords>,
    pub attack_token: String,
}

pub fn get_random_opponent_id(attacker_id: i32, conn: &mut PgConnection) -> Result<Option<i32>> {
    let sorted_users: Vec<(i32, i32)> = user::table
        .order_by(user::trophies.asc())
        .select((user::id, user::trophies))
        .load::<(i32, i32)>(conn)?;

    let attacker_index = sorted_users
        .iter()
        .position(|(id, _)| *id == attacker_id)
        .unwrap_or_default();
    let less_or_equal_trophies = sorted_users
        .iter()
        .take(attacker_index)
        .rev()
        .take(5)
        .cloned()
        .collect::<Vec<_>>();
    let more_or_equal_trophies = sorted_users
        .iter()
        .skip(attacker_index + 1)
        .take(5)
        .cloned()
        .collect::<Vec<_>>();
    let random_opponent = less_or_equal_trophies
        .iter()
        .chain(&more_or_equal_trophies)
        .choose(&mut rand::thread_rng())
        .map(|&(id, _)| id);

    Ok(random_opponent)
}

pub fn get_shortest_paths(
    conn: &mut PgConnection,
    map_id: i32,
) -> Result<HashMap<SourceDest, Coords>> {
    use crate::schema::shortest_path::dsl::*;
    let results = shortest_path
        .filter(base_id.eq(map_id))
        .load::<ShortestPath>(conn)
        .map_err(|err| DieselError {
            table: "shortest_path",
            function: function!(),
            error: err,
        })?;
    let mut shortest_paths: HashMap<SourceDest, Coords> = HashMap::new();
    for path in results {
        shortest_paths.insert(
            SourceDest {
                source_x: path.source_x,
                source_y: path.source_y,
                dest_x: path.dest_x,
                dest_y: path.dest_y,
            },
            Coords {
                x: path.next_hop_x,
                y: path.next_hop_y,
            },
        );
    }
    Ok(shortest_paths)
}

pub fn get_opponent_base_details(
    defender_id: i32,
    conn: &mut PgConnection,
) -> Result<DefenseResponse> {
    let map = fetch_map_layout(conn, &defender_id)?;

    let response = get_map_details_for_attack(conn, map)?;

    Ok(response)
}

pub fn add_game_id_to_redis(
    attacker_id: i32,
    defender_id: i32,
    game_id: i32,
    mut redis_conn: RedisConn,
) -> Result<()> {
    redis_conn
        .set_ex(
            format!("Game:{}", attacker_id),
            game_id,
            (GAME_ID_EXPIRATION_TIME_IN_MINUTES * 60)
                .try_into()
                .unwrap(),
        )
        .map_err(|err| anyhow::anyhow!("Failed to set key: {}", err))?;

    redis_conn
        .set_ex(
            format!("Game:{}", defender_id),
            game_id,
            (GAME_ID_EXPIRATION_TIME_IN_MINUTES * 60)
                .try_into()
                .unwrap(),
        )
        .map_err(|err| anyhow::anyhow!("Failed to set key: {}", err))?;
    Ok(())
}

pub fn get_game_id_from_redis(user_id: i32, mut redis_conn: RedisConn) -> Result<Option<i32>> {
    let game_id: Option<i32> = redis_conn
        .get(format!("Game:{}", user_id))
        .map_err(|err| anyhow::anyhow!("Failed to get key: {}", err))?;
    Ok(game_id)
}

pub fn encode_attack_token(attacker_id: i32, defender_id: i32) -> Result<String> {
    let jwt_secret = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set!");
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let jwt_max_age: i64 = env::var("ATTACK_TOKEN_AGE_IN_MINUTES")
        .expect("JWT max age must be set!")
        .parse()
        .expect("JWT max age must be an integer!");
    let token_expiring_time = now + Duration::minutes(jwt_max_age);
    let exp = (token_expiring_time).timestamp() as usize;
    let token: AttackToken = AttackToken {
        attacker_id,
        defender_id,
        exp,
        iat,
    };

    let token_result = encode(
        &Header::default(),
        &token,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    );
    let token = match token_result {
        Ok(token) => token,
        Err(e) => return Err(e.into()),
    };

    Ok(token)
}

pub fn decode_user_token(token: &str) -> Result<i32> {
    let jwt_secret = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set!");
    let token_data = decode::<TokenClaims>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_str().as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|err| anyhow::anyhow!("Failed to decode token: {}", err))?;

    Ok(token_data.claims.id)
}

pub fn decode_attack_token(token: &str) -> Result<AttackToken> {
    let jwt_secret = env::var("COOKIE_KEY").expect("COOKIE_KEY must be set!");
    let token_data = decode::<AttackToken>(
        token,
        &DecodingKey::from_secret(jwt_secret.as_str().as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .map_err(|err| anyhow::anyhow!("Failed to decode token: {}", err))?;

    Ok(token_data.claims)
}

pub fn get_mines(conn: &mut PgConnection, map_id: i32) -> Result<Vec<MineDetails>> {
    use crate::schema::{block_type, map_spaces, mine_type};

    let joined_table = map_spaces::table
        .filter(map_spaces::map_id.eq(map_id))
        .inner_join(block_type::table.inner_join(mine_type::table));

    let mines: Vec<MineDetails> = joined_table
        .load::<(MapSpaces, (BlockType, MineType))>(conn)?
        .into_iter()
        .enumerate()
        .map(|(mine_id, (map_space, (_, mine_type)))| MineDetails {
            id: mine_id as i32,
            damage: mine_type.damage,
            radius: mine_type.radius,
            pos: Coordinates {
                x: map_space.x_coordinate,
                y: map_space.y_coordinate,
            },
        })
        .collect();

    Ok(mines)
}

pub fn get_defenders(conn: &mut PgConnection, map_id: i32) -> Result<Vec<DefenderDetails>> {
    use crate::schema::{block_type, building_type, defender_type, map_spaces};
    let result: Vec<(MapSpaces, (BlockType, BuildingType, DefenderType))> = map_spaces::table
        .inner_join(
            block_type::table
                .inner_join(building_type::table)
                .inner_join(defender_type::table),
        )
        .filter(map_spaces::map_id.eq(map_id))
        .load::<(MapSpaces, (BlockType, BuildingType, DefenderType))>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?;

    let mut defenders: Vec<DefenderDetails> = Vec::new();

    for (defender_id, (map_space, (_, _, defender_type))) in result.iter().enumerate() {
        let (hut_x, hut_y) = (map_space.x_coordinate, map_space.y_coordinate);
        // let path = vec![(hut_x, hut_y)];
        defenders.push(DefenderDetails {
            id: defender_id as i32 + 1,
            radius: defender_type.radius,
            speed: defender_type.speed,
            damage: defender_type.damage,
            defender_pos: Coordinates { x: hut_x, y: hut_y },
            is_alive: true,
            damage_dealt: false,
            target_id: None,
            path_in_current_frame: Vec::new(),
        })
    }
    // Sorted to handle multiple defenders attack same attacker at same frame
    defenders.sort_by(|defender_1, defender_2| (defender_2.damage).cmp(&defender_1.damage));
    Ok(defenders)
}

pub fn get_buildings(conn: &mut PgConnection, map_id: i32) -> Result<Vec<BuildingDetails>> {
    use crate::schema::{block_type, building_type, map_spaces};

    let joined_table = map_spaces::table
        .inner_join(block_type::table.inner_join(building_type::table))
        .filter(map_spaces::map_id.eq(map_id))
        .filter(building_type::id.ne(ROAD_ID));

    let buildings: Vec<BuildingDetails> = joined_table
        .load::<(MapSpaces, (BlockType, BuildingType))>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(map_space, (_, building_type))| BuildingDetails {
            id: map_space.id,
            // current_hp: building_type.hp,
            // total_hp: building_type.hp,
            current_hp: 0,
            total_hp: 0,
            artifacts_obtained: 0,
            tile: Coordinates {
                x: map_space.x_coordinate,
                y: map_space.y_coordinate,
            },
            width: building_type.width,
        })
        .collect();
    update_buidling_artifacts(conn, map_id, buildings)
}

pub fn update_buidling_artifacts(
    conn: &mut PgConnection,
    map_id: i32,
    mut buildings: Vec<BuildingDetails>,
) -> Result<Vec<BuildingDetails>> {
    use crate::schema::{artifact, map_spaces};

    let result: Vec<(MapSpaces, Artifact)> = map_spaces::table
        .inner_join(artifact::table)
        .filter(map_spaces::map_id.eq(map_id))
        .load::<(MapSpaces, Artifact)>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?;

    // From the above table, create a hashmap, key being map_space_id and value being the artifact count
    let mut artifact_count: HashMap<i32, i64> = HashMap::new();

    for (map_space, artifact) in result.iter() {
        artifact_count.insert(map_space.id, artifact.count.into());
    }

    // Update the buildings with the artifact count
    for building in buildings.iter_mut() {
        building.artifacts_obtained = *artifact_count.get(&building.id).unwrap_or(&0) as i32;
    }

    Ok(buildings)
}
// pub fn get_super_buildings(conn: &mut PgConnection, map_id: i32) -> Result<Vec<BuildingDetails>> {
//     use crate::schema::{block_type, building_type, map_spaces, artifact};

//     let buildings: Vec<BuildingDetails> = map_spaces::table
//     .inner_join(block_type::table.inner_join(building_type::table))
//     .filter(map_spaces::map_id.eq(map_id))
//     .filter(building_type::id.ne(ROAD_ID))
//     .left_join(artifact::table)
//     .select((
//         map_spaces::all_columns,
//         block_type::all_columns,
//         building_type::all_columns,
//         artifact::count,
//     ))
//     .load::<(MapSpaces, BlockType, BuildingType, i64)>(conn)
//     .map_err(|err| DieselError {
//         table: "map_spaces",
//         function: function!(),
//         error: err,
//     })?
//     .into_iter()
//     .map(|(map_space, _, building_type, artifact_count)| BuildingDetails {
//         id: map_space.id,
//         current_hp: 0,
//         total_hp: 0,
//         artifacts_obtained: artifact_count,
//         tile: Coordinates {
//             x: map_space.x_coordinate,
//             y: map_space.y_coordinate,
//         },
//         width: building_type.width,
//     })
//     .collect();

// }
