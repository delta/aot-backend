use crate::api::error::AuthError;
use crate::api::user::util::fetch_user;
use crate::api::util::{can_show_replay, get_current_levels_fixture};
use crate::constants::TOTAL_ATTACKS_ON_A_BASE;
use crate::error::DieselError;
use crate::models::{Game, LevelsFixture, MapLayout, SimulationLog};
use crate::util::function;
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

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
pub struct UserDetail {
    pub user_id: i32,
    pub username: String,
    pub overall_rating: i32,
    pub avatar: i32,
}

#[derive(Queryable, Deserialize, Serialize)]
pub struct LeaderboardEntry {
    pub attacker: UserDetail,
    pub defender: UserDetail,
    pub can_be_attacked: bool,
}

pub fn get_leaderboard(
    page: i64,
    limit: i64,
    user_id: i32,
    conn: &mut PgConnection,
) -> Result<LeaderboardResponse> {
    use crate::schema::{game, map_layout, user};

    let level_id: i32 = get_current_levels_fixture(conn)?.id;
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
        .fold(HashMap::new(), |mut hashmap, defender_id| {
            *hashmap.entry(defender_id).or_insert(0) += 1;
            hashmap
        });
    let user = fetch_user(conn, user_id)?.ok_or(AuthError::UserNotFound)?;
    let already_attacked: HashSet<i32> = game::table
        .inner_join(map_layout::table)
        .select(game::defend_id)
        .filter(map_layout::level_id.eq(level_id))
        .filter(game::attack_id.eq(user_id))
        .load::<i32>(conn)
        .map_err(|err| DieselError {
            table: "game_join_map_layout",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .collect();
    let attacker = map_layout::table
        .filter(map_layout::player.eq(user_id))
        .filter(map_layout::level_id.eq(level_id))
        .filter(map_layout::is_valid.eq(true))
        .select(map_layout::player)
        .first::<i32>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;
    let can_be_attacked = |defender_id: i32, map_valid: Option<bool>| {
        *no_of_times_attacked.get(&defender_id).unwrap_or(&0) < TOTAL_ATTACKS_ON_A_BASE
            && map_valid.unwrap_or(false)
            && !already_attacked.contains(&defender_id)
            && attacker.is_some()
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
            user::avatar_id,
            map_layout::is_valid.nullable(),
        ))
        .order_by(user::overall_rating.desc())
        .offset(offset)
        .limit(limit)
        .load::<(i32, String, i32, i32, Option<bool>)>(conn)
        .map_err(|err| DieselError {
            table: "user_join_map_layout",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(
            |(user_id, username, overall_rating, avatar, map_valid)| LeaderboardEntry {
                defender: UserDetail {
                    user_id,
                    username,
                    overall_rating,
                    avatar,
                },
                attacker: UserDetail {
                    user_id: user.id,
                    username: user.username.to_string(),
                    overall_rating: user.overall_rating,
                    avatar: user.avatar,
                },
                can_be_attacked: can_be_attacked(user_id, map_valid),
            },
        )
        .collect();

    Ok(LeaderboardResponse {
        leaderboard_entries,
        last_page,
    })
}

pub fn fetch_is_replay_allowed(
    game_id: i32,
    user_id: i32,
    conn: &mut PgConnection,
) -> Result<bool> {
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

pub fn fetch_replay(game_id: i32, conn: &mut PgConnection) -> Result<SimulationLog> {
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
