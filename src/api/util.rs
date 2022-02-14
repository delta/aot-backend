use crate::{
    constants::END_HOUR,
    models::{Game, LevelsFixture},
};
use chrono::{Local, NaiveTime};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct GameHistoryResponse {
    pub games: Vec<GameHistoryEntry>,
}

#[derive(Deserialize, Serialize)]
pub struct GameHistoryEntry {
    pub game: Game,
    pub is_replay_available: bool,
}

pub fn can_show_replay(requested_user: i32, game: &Game, levels_fixture: &LevelsFixture) -> bool {
    let end_time = NaiveTime::from_hms(END_HOUR, 0, 0);
    let current_date = Local::now().naive_local().date();
    let current_time = Local::now().naive_local().time();
    let is_current_round_over = current_time > end_time;
    is_current_round_over // current round is over
        || requested_user == game.attack_id // user requesting history if an attacker or defender
        || requested_user == game.defend_id
        || current_date > levels_fixture.start_date // game happened in previous rounds
}
