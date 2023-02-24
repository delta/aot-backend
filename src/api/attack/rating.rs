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

fn trophy_scale(old_ar: f32, old_dr: f32) -> (f32, f32) {
    let hi_scale = HIGHEST_TROPHY / ((old_dr - old_ar).abs() + HIGHEST_TROPHY);
    let ars = ((old_ar * hi_scale) * 0.5 / HIGHEST_TROPHY) + 0.75;
    let drs = ((old_dr * hi_scale) * 0.5 / HIGHEST_TROPHY) + 0.75;
    (ars, drs)
}

/* change rating datatype to int */
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
    let (ra_scale, rd_scale) = trophy_scale(old_attacker_rating as f32, old_defender_rating as f32);

    if attack_score > 0.0 {
        new_attacker_rating = (attack_score * baseline_trophies(eb) * ra_scale) as i32;
    } else {
        new_attacker_rating = (attack_score * baseline_trophies(ea) * ra_scale) as i32;
    }
    if defence_score > 0.0 {
        new_defender_rating = (defence_score * baseline_trophies(ea) * rd_scale) as i32;
    } else {
        new_defender_rating = (defence_score * baseline_trophies(eb) * rd_scale) as i32;
    }

    new_attacker_rating += old_attacker_rating;
    new_defender_rating += old_defender_rating;

    (new_attacker_rating, new_defender_rating)
}

fn bonus_trophies(
    attacker_rating: i32,
    defender_rating: i32,
    metrics: (f32, f32, f32, f32, f32, f32, f32, f32),
) -> (i32, i32) {
    let mut new_attack_rating = (BONUS_SCALE * metrics.0 / metrics.4) as i32;
    let mut new_defence_rating = (BONUS_SCALE * (metrics.1 + metrics.2 + metrics.3)
        / (metrics.5 + metrics.6 + metrics.7)) as i32;
    new_attack_rating += attacker_rating;
    new_defence_rating += defender_rating;

    (new_attack_rating, new_defence_rating)
}

impl Game {
    pub fn update_rating(
        &self,
        metrics: (f32, f32, f32, f32, f32, f32, f32, f32),
        conn: &mut PgConnection,
    ) -> Result<(i32, i32), diesel::result::Error> {
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
            .select(user::overall_rating)
            .first::<i32>(conn)?;
        let defender_rating = user::table
            .find(defend_id)
            .select(user::overall_rating)
            .first::<i32>(conn)?;
        let (mut new_attacker_rating, mut new_defender_rating) = new_rating(
            attacker_rating,
            defender_rating,
            attack_score,
            defence_score,
        );
        (new_attacker_rating, new_defender_rating) =
            bonus_trophies(new_attacker_rating, new_defender_rating, metrics);

        diesel::update(user::table.filter(user::id.eq(attack_id)))
            .set(user::overall_rating.eq(new_attacker_rating))
            .execute(conn)?;
        diesel::update(user::table.filter(user::id.eq(defend_id)))
            .set(user::overall_rating.eq(new_defender_rating))
            .execute(conn)?;
        if new_attacker_rating > attacker_rating {
            diesel::update(user::table.filter(user::id.eq(attack_id)))
                .set(user::highest_rating.eq(new_attacker_rating))
                .execute(conn)?;
        }
        if new_defender_rating > defender_rating {
            diesel::update(user::table.filter(user::id.eq(defend_id)))
                .set(user::highest_rating.eq(new_defender_rating))
                .execute(conn)?;
        }
        Ok((
            (new_attacker_rating - attacker_rating),
            (new_defender_rating - defender_rating),
        ))
    }
}
