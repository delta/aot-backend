use crate::{
    constants::{BONUS_SCALE, HIGHEST_TROPHY, SCALE_FACTOR},
    models::Game,
};
use diesel::prelude::*;
use diesel::PgConnection;

fn expected_score(player_rating: f32, opponent_rating: f32) -> f32 {
    1.0 / (1.0 + 10_f32.powf((opponent_rating - player_rating) / 400.0))
}

fn baseline_trophies(ep: f32) -> f32 {
    (ep + 1.0) * SCALE_FACTOR
}

fn trophy_scale(old_attacker_rating: f32, old_defender_rating: f32) -> (f32, f32) {
    let hi_scale =
        HIGHEST_TROPHY / ((old_defender_rating - old_attacker_rating).abs() + HIGHEST_TROPHY);
    let attacker_rating_scale = ((old_attacker_rating * hi_scale) * 0.5 / HIGHEST_TROPHY) + 0.75;
    let defender_rating_scale = ((old_defender_rating * hi_scale) * 0.5 / HIGHEST_TROPHY) + 0.75;
    (attacker_rating_scale, defender_rating_scale)
}

/* change rating datatype to int */
#[allow(dead_code)]
fn new_rating(
    old_attacker_rating: i32,
    old_defender_rating: i32,
    attack_score: f32,
    defence_score: f32,
) -> (i32, i32) {
    let ea = expected_score(old_attacker_rating as f32, old_defender_rating as f32);
    let eb = 1.0 - ea;
    let mut new_attacker_rating: i32;
    let mut new_defender_rating: i32;
    let (attacker_rating_scale, defender_rating_scale) =
        trophy_scale(old_attacker_rating as f32, old_defender_rating as f32);

    if attack_score > 0.0 {
        new_attacker_rating = (attack_score * baseline_trophies(eb) * attacker_rating_scale) as i32;
    } else {
        new_attacker_rating = (attack_score * baseline_trophies(ea) * attacker_rating_scale) as i32;
    }
    if defence_score > 0.0 {
        new_defender_rating =
            (defence_score * baseline_trophies(ea) * defender_rating_scale) as i32;
    } else {
        new_defender_rating =
            (defence_score * baseline_trophies(eb) * defender_rating_scale) as i32;
    }

    new_attacker_rating += old_attacker_rating;
    new_defender_rating += old_defender_rating;

    (new_attacker_rating, new_defender_rating)
}

#[allow(dead_code)]
fn bonus_trophies(
    attacker_rating: &mut i32,
    defender_rating: &mut i32,
    (live_attackers, used_defenders, used_mines): (i32, i32, i32),
) {
    *attacker_rating += BONUS_SCALE * live_attackers;
    *defender_rating += BONUS_SCALE * (used_defenders + used_mines) / 2;
}

impl Game {
    #[allow(dead_code)]
    pub fn update_rating(
        &self,
        metrics: (i32, i32, i32),
        conn: &mut PgConnection,
    ) -> Result<(i32, i32, i32, i32), diesel::result::Error> {
        use crate::schema::user;

        let attack_score = self.attack_score as f32 / 100_f32;
        let defence_score = self.defend_score as f32 / 100_f32;

        let Game {
            attack_id,
            defend_id,
            ..
        } = self;
        let attacker_rating = user::table
            .find(attack_id)
            .select(user::trophies)
            .first::<i32>(conn)?;
        let defender_rating = user::table
            .find(defend_id)
            .select(user::trophies)
            .first::<i32>(conn)?;
        let (mut new_attacker_rating, mut new_defender_rating) = new_rating(
            attacker_rating,
            defender_rating,
            attack_score,
            defence_score,
        );
        bonus_trophies(&mut new_attacker_rating, &mut new_defender_rating, metrics);

        diesel::update(user::table.filter(user::id.eq(attack_id)))
            .set(user::trophies.eq(new_attacker_rating))
            .execute(conn)?;
        diesel::update(user::table.filter(user::id.eq(defend_id)))
            .set(user::trophies.eq(new_defender_rating))
            .execute(conn)?;
        Ok((
            new_attacker_rating,
            new_defender_rating,
            (new_attacker_rating - attacker_rating),
            (new_defender_rating - defender_rating),
        ))
    }
}
