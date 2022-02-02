use crate::models::*;
use crate::schema::{game, user};
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use std::collections::HashMap;

pub fn calculate_probablity(rating1: f64, rating2: f64) -> f64 {
    let base: i32 = 10;
    let factor: f64 = 400.0;
    let denominator = 1.0 + f64::powf(base.into(), (rating1 - rating2) / factor);
    1.0 / denominator
}

pub fn get_elo(rating: f64, k: f64, actual: f64, expected: f64) -> f64 {
    rating + k * (actual - expected)
}

pub fn update_user_rating(conn: &PgConnection, user_id: i32, user_rating: i32) -> Result<()> {
    use crate::schema::user::dsl::*;

    diesel::update(user.find(user_id))
        .set(overall_rating.eq(user_rating))
        .execute(conn)?;

    Ok(())
}

pub fn rating(conn: &PgConnection, game_id: i32) -> () {
    const K: f64 = 30.0;

    let Game {
        id,
        attack_id,
        defend_id,
        map_layout_id,
        attack_score,
        defend_score,
    } = game::table
        .find(game_id)
        .first::<Game>(conn)
        .expect("Couldn't get game");

    let attacker_initial_rating = user::table
        .select(user::overall_rating)
        .find(attack_id)
        .first::<i32>(conn)
        .expect("Couldn't get attackrating");

    let defender_initial_rating = user::table
        .select(user::overall_rating)
        .find(defend_id)
        .first::<i32>(conn)
        .expect("Couldn't get defendrating");

    let mut actual1: f64 = 0.0;
    let mut actual2: f64 = 0.0;

    if attack_score > defend_score {
        actual1 = 1.0;
        actual2 = 0.0;
    } else if attack_score < defend_score {
        actual1 = 0.0;
        actual2 = 1.0;
    } else {
        actual1 = 0.0;
        actual2 = 0.0;
    }

    let a_rating = get_elo(
        attacker_initial_rating as f64,
        K,
        1.0,
        calculate_probablity(
            defender_initial_rating as f64,
            attacker_initial_rating as f64,
        ),
    );
    let d_rating = get_elo(
        defender_initial_rating as f64,
        K,
        1.0,
        calculate_probablity(
            attacker_initial_rating as f64,
            defender_initial_rating as f64,
        ),
    );

    update_user_rating(conn, attack_id, a_rating as i32);
    update_user_rating(conn, defend_id, d_rating as i32);
}
