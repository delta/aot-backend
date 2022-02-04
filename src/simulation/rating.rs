use crate::models::*;
use crate::schema::{game, user};
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};

fn calculate_probablity(rating1: f32, rating2: f32) -> f32 {
    let base: f32 = 10.0;
    let factor: f32 = 400.0;
    let denominator = 1.0 + f32::powf(base, (rating1 - rating2) / factor);
    1.0 / denominator
}

fn get_elo(rating: f32, k: f32, actual: f32, expected: f32) -> f32 {
    rating + k * (actual - expected)
}

fn update_user_rating(conn: &PgConnection, user_id: i32, user_rating: f32) -> Result<()> {
    use crate::schema::user::dsl::*;

    diesel::update(user.find(user_id))
        .set(overall_rating.eq(user_rating))
        .execute(conn)?;

    Ok(())
}

pub fn rating(conn: &PgConnection, game_id: i32) -> () {
    const K: f32 = 30.0;

    let Game {
        id: _,
        attack_id,
        defend_id,
        map_layout_id: _,
        attack_score,
        defend_score,
    } = game::table
        .find(game_id)
        .first::<Game>(conn)
        .expect("Couldn't get game");

    let attacker_initial_rating = user::table
        .select(user::overall_rating)
        .find(attack_id)
        .first::<f32>(conn)
        .expect("Couldn't get attack rating");

    let defender_initial_rating = user::table
        .select(user::overall_rating)
        .find(defend_id)
        .first::<f32>(conn)
        .expect("Couldn't get defend rating");

    let a_rating = get_elo(
        attacker_initial_rating as f32,
        K,
        attack_score as f32,
        calculate_probablity(
            defender_initial_rating as f32,
            attacker_initial_rating as f32,
        ),
    );
    let d_rating = get_elo(
        defender_initial_rating as f32,
        K,
        defend_score as f32,
        calculate_probablity(
            attacker_initial_rating as f32,
            defender_initial_rating as f32,
        ),
    );

    update_user_rating(conn, attack_id, a_rating);
    update_user_rating(conn, defend_id, d_rating);
}
