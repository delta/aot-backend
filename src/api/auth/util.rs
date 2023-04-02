use crate::api::RedisConn;
use crate::models::*;
use crate::schema::user::{self, otps_sent};
use crate::util::function;
use crate::{constants::INITIAL_RATING, error::DieselError};
use anyhow::Result;
use diesel::prelude::*;
use pwhash::bcrypt;
use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_user(conn: &mut PgConnection, id: i32) -> Result<Option<User>> {
    let user = user::table
        .find(id)
        .first::<User>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(user)
}

pub fn update_otp_count(conn: &mut PgConnection, id: i32) -> Result<()> {
    diesel::update(user::table.filter(user::id.eq(id)))
        .set(user::otps_sent.eq(otps_sent + 1))
        .execute(conn)?;
    Ok(())
}

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
            overall_rating: &INITIAL_RATING,
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

pub fn verify_user(conn: &mut PgConnection, id: i32) -> Result<()> {
    let user: User = diesel::update(user::table.find(id))
        .set(user::is_verified.eq(true))
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    // If some other user(s) have used the same phone number, but have not verified
    diesel::delete(user::table)
        .filter(user::phone.eq(&user.phone))
        .filter(user::username.ne(&user.username))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(())
}

pub fn get_user_with_phone(conn: &mut PgConnection, phone: &str) -> Result<Option<User>> {
    let user = user::table
        .filter(user::phone.eq(&phone))
        .filter(user::is_verified.eq(true))
        .first::<User>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(user)
}

pub fn reset_password(
    pg_conn: &mut PgConnection,
    mut redis_conn: RedisConn,
    user_id: i32,
    password: &str,
) -> Result<()> {
    let hashed_password = bcrypt::hash(password)?;
    diesel::update(user::table.find(user_id))
        .set(user::password.eq(&hashed_password))
        .execute(pg_conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    // Set reset password time in redis to invalidate sessions before this time
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    redis_conn.set(user_id, created_at)?;
    Ok(())
}

pub fn generate_otp() -> String {
    let otp = rand::thread_rng().gen_range(0..100000);
    format!("{otp:05}")
}
