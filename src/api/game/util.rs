use crate::api::util::can_show_replay;
use crate::constants::TOTAL_ATTACKS_ON_A_BASE;
use crate::error::DieselError;
use crate::models::{Game, LevelsFixture, MapLayout, SimulationLog};
use crate::util::function;
use anyhow::Result;
use chrono::Local;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    pub overall_rating: f32,
    pub can_be_attacked: bool,
}

pub fn get_leaderboard(page: i64, limit: i64, conn: &PgConnection) -> Result<LeaderboardResponse> {
    use crate::schema::{game, levels_fixture, map_layout, user};

    let current_date = Local::now().naive_local().date();
    let level_id: i32 = levels_fixture::table
        .filter(levels_fixture::start_date.le(current_date))
        .filter(levels_fixture::end_date.gt(current_date))
        .select(levels_fixture::id)
        .first(conn)
        .map_err(|err| DieselError {
            table: "levels_fixture",
            function: function!(),
            error: err,
        })?;
    let no_of_times_attacked: HashMap<i32, i64> = game::table
        .inner_join(map_layout::table)
        .select(game::defend_id)
        .filter(map_layout::level_id.eq(level_id))
        .load::<i32>(conn)
        .map_err(|err| DieselError {
            table: "game_join_map_layout",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .fold(HashMap::new(), |mut hashmap, user_id| {
            *hashmap.entry(user_id).or_insert(0) += 1;
            hashmap
        });
    let can_be_attacked = |user_id: i32, map_valid: Option<bool>| {
        *no_of_times_attacked.get(&user_id).unwrap_or(&0) < TOTAL_ATTACKS_ON_A_BASE
            && map_valid.unwrap_or(false)
    };

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
        .left_join(
            map_layout::table.on(map_layout::player
                .eq(user::id)
                .and(map_layout::level_id.eq(level_id))
                .and(map_layout::is_valid.eq(true))),
        )
        .select((
            user::id,
            user::username,
            user::overall_rating,
            map_layout::is_valid.nullable(),
        ))
        .order_by(user::overall_rating.desc())
        .offset(offset)
        .limit(limit)
        .load::<(i32, String, f32, Option<bool>)>(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(
            |(user_id, username, overall_rating, map_valid)| LeaderboardEntry {
                username,
                overall_rating,
                can_be_attacked: can_be_attacked(user_id, map_valid),
            },
        )
        .collect();

    Ok(LeaderboardResponse {
        leaderboard_entries,
        last_page,
    })
}

pub fn fetch_is_replay_allowed(game_id: i32, user_id: i32, conn: &PgConnection) -> Result<bool> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let result = joined_table
        .filter(game::id.eq(game_id))
        .first::<(Game, (MapLayout, LevelsFixture))>(conn)
        .optional()?;

    if let Some((game, (_, fixture))) = result {
        return Ok(can_show_replay(user_id, &game, &fixture));
    }

    Ok(false)
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
