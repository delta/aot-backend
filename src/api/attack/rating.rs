use crate::{
    constants::{HEALTH, K_FACTOR},
    models::Game,
};
use diesel::prelude::*;
use diesel::PgConnection;

fn expected_score(player_rating: f32, opponent_rating: f32) -> f32 {
    1.0 / (1.0 + 10_f32.powf((opponent_rating - player_rating) / 400.0))
}

fn new_rating(old_rating: f32, expected_score: f32, actual_score: f32) -> f32 {
    old_rating + K_FACTOR * (actual_score - expected_score)
}

impl Game {
    pub fn update_rating(
        &self,
        rating_factor: f32,
        no_of_robots: i32,
        conn: &PgConnection,
    ) -> Result<(f32, f32), diesel::result::Error> {
        use crate::schema::user;

        let max_score = 2 * HEALTH * no_of_robots;
        let attack_score = (self.attack_score.max(0) as f32 / max_score as f32).powf(rating_factor);

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
            new_rating(attacker_rating, expected_attacker_score, attack_score);
        let new_defender_rating = new_rating(
            defender_rating,
            expected_defender_score,
            1_f32 - attack_score,
        );
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
