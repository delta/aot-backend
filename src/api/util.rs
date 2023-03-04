use crate::error::DieselError;
use crate::util::function;
use crate::{
    constants::ATTACK_END_TIME,
    models::{Game, LevelsFixture},
};
use anyhow::Result;
use chrono::{Local, NaiveTime};
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
    let end_time = NaiveTime::parse_from_str(ATTACK_END_TIME, "%H:%M:%S").unwrap();
    let current_date = Local::now().naive_local();
    let current_time = current_date.time();
    let is_current_round_over = current_time > end_time;
    is_current_round_over // current round is over
        || requested_user == game.attack_id // user requesting history if an attacker or defender
        || requested_user == game.defend_id
        || current_date > levels_fixture.start_date // game happened in previous rounds
}

pub fn get_current_levels_fixture(conn: &mut PgConnection) -> Result<LevelsFixture> {
    use crate::schema::levels_fixture;
    // let current_date = Local::now().naive_local();
    let level: LevelsFixture = levels_fixture::table
        // .filter(levels_fixture::start_date.le(current_date))
        // .filter(levels_fixture::end_date.gt(current_date))
        .first(conn)
        .map_err(|err| DieselError {
            table: "levels_fixture",
            function: function!(),
            error: err,
        })?;
    Ok(level)
}
