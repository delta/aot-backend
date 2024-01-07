use crate::api::RedisConn;
use crate::models::*;
use crate::schema::user::{self};
use crate::util::function;
use crate::{constants::INITIAL_RATING, error::DieselError};
use anyhow::Result;
use diesel::prelude::*;
use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;

pub fn get_user_by_username(conn: &mut PgConnection, username: &str) -> Result<Option<User>> {
    let user = user::table
        .filter(user::username.eq(username))
        .first::<User>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(user)
}

pub fn get_pragyan_user(
    pg_conn: &mut PgConnection,
    redis_conn: &mut RedisConn,
    email: &str,
    name: &str,
) -> Result<User> {
    // Already logged in before
    if let Some(user) = user::table
        .filter(user::email.eq(&email))
        .first::<User>(pg_conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?
    {
        Ok(user)
    } else {
        // First login
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(4)
            .map(char::from)
            .collect();
        let username = &format!("{email}_{random_string}");
        let new_user = NewUser {
            name,
            email,
            phone: "",
            username,
            trophies: &INITIAL_RATING,
            is_pragyan: &true,
            password: "",
            is_verified: &false,
            highest_rating: &INITIAL_RATING,
            avatar: &0,
            otps_sent: &0,
        };
        let user: User = diesel::insert_into(user::table)
            .values(&new_user)
            .get_result(pg_conn)
            .map_err(|err| DieselError {
                table: "user",
                function: function!(),
                error: err,
            })?;
        redis_conn.set(user.id, 0)?;
        Ok(user)
    }
}
