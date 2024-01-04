use crate::error::DieselError;
use crate::models::{Game, LevelsFixture};
use crate::util::function;
use anyhow::Result;
use chrono::Local;
use diesel::prelude::*;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use super::game::util::UserDetail;

#[derive(Deserialize, Serialize)]
pub struct GameHistoryResponse {
    pub games: Vec<GameHistoryEntry>,
}

#[derive(Deserialize, Serialize)]
pub struct GameHistoryEntry {
    pub game: Game,
    pub attacker: UserDetail,
    pub defender: UserDetail,
    pub is_replay_available: bool,
}

pub fn can_show_replay(requested_user: i32, game: &Game, levels_fixture: &LevelsFixture) -> bool {
    let current_date = Local::now().naive_local();
    requested_user == game.attack_id // user requesting history if an attacker or defender
        || requested_user == game.defend_id
        || current_date > levels_fixture.start_date // game happened in previous rounds
}

pub fn get_current_levels_fixture(conn: &mut PgConnection) -> Result<LevelsFixture> {
    use crate::schema::levels_fixture;
    let current_date = Local::now().naive_local();
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
