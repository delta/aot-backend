use crate::api::util::{can_show_replay, get_current_levels_fixture};
use crate::error::DieselError;
use crate::models::{Game, LevelsFixture, MapLayout, SimulationLog};
use crate::util::function;
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use serde::{Deserialize, Serialize};

#[derive(Queryable, Deserialize, Serialize)]
pub struct UserDetail {
    pub user_id: i32,
    pub username: String,
    pub trophies: i32,
    pub avatar_id: i32,
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
    pub user_id: i32,
    pub name: String,
    pub trophies: i32,
    pub artifacts: i32,
    pub attacks_won: i32,
    pub defenses_won: i32,
    pub avatar_url: String,
}

pub fn get_leaderboard(
    page: i64,
    limit: i64,
    conn: &mut PgConnection,
) -> Result<LeaderboardResponse> {
    use crate::schema::{map_layout, user};

    let level_id: i32 = get_current_levels_fixture(conn)?.id;

    let total_entries: i64 = user::table
        .count()
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    let off_set: i64 = (page - 1) * limit;
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
            user::name,
            user::trophies,
            user::artifacts,
            user::attacks_won,
            user::defenses_won,
            user::avatar_id,
            //map_layout::is_valid.nullable(),
        ))
        .order_by(user::trophies.desc())
        .offset(off_set)
        .limit(limit)
        .load::<(i32, String, i32, i32, i32, i32, i32)>(conn)
        .map_err(|err| DieselError {
            table: "user_join_map_layout",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(
            |(id, name, trophies, artifacts, attacks_won, defenses_won, avatar_id)| {
                LeaderboardEntry {
                    user_id: id,
                    name,
                    trophies,
                    artifacts,
                    attacks_won,
                    defenses_won,
                    avatar_url: avatar_id.to_string(),
                }
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
