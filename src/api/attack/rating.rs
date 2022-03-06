use crate::{
    constants::{K_FACTOR, MAX_SCORE},
    models::Game,
};
use diesel::prelude::*;
use diesel::PgConnection;

fn expected_score(player_rating: f32, opponent_rating: f32) -> f32 {
    1.0 / (1.0 + 10_f32.powf((opponent_rating - player_rating) / 400.0))
}

fn new_attacker_rating(old_rating: f32, expected_score: f32, score_ratio: f32) -> f32 {
    old_rating + K_FACTOR * score_ratio * (1.0 - expected_score)
}

fn new_defender_rating(old_rating: f32, expected_score: f32, score_ratio: f32) -> f32 {
    old_rating + K_FACTOR * score_ratio * -expected_score
}

impl Game {
    pub fn update_rating(&self, conn: &PgConnection) -> Result<(f32, f32), diesel::result::Error> {
        use crate::schema::user;

        let score_ratio = self.attack_score as f32 / MAX_SCORE as f32;
        let Game {
            attack_id,
            defend_id,
            ..
        } = self;
        let attacker_rating = user::table
            .find(attack_id)
            .select(user::overall_rating)
            .first::<f32>(conn)?;
        let defender_rating = user::table
            .find(defend_id)
            .select(user::overall_rating)
            .first::<f32>(conn)?;
        let expected_attacker_score = expected_score(attacker_rating, defender_rating);
        let expected_defender_score = expected_score(defender_rating, attacker_rating);
        let new_attacker_rating =
            new_attacker_rating(attacker_rating, expected_attacker_score, score_ratio);
        let new_defender_rating =
            new_defender_rating(defender_rating, expected_defender_score, score_ratio);
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
