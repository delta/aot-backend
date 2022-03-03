use crate::api::RedisConn;
use crate::models::*;
use crate::schema::user;
use crate::util::function;
use crate::{constants::INITIAL_RATING, error::DieselError};
use anyhow::Result;
use diesel::prelude::*;
use pwhash::bcrypt;
use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_user(conn: &PgConnection, id: i32) -> Result<Option<User>> {
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

pub fn get_user_by_username(conn: &PgConnection, username: &str) -> Result<Option<User>> {
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

pub fn get_pragyan_user(conn: &PgConnection, email: &str, name: &str) -> Result<(i32, String)> {
    // Already logged in before
    if let Some(user) = user::table
        .filter(user::email.eq(&email))
        .first::<User>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?
    {
        Ok((user.id, user.username))
    } else {
        // First login
        let random_string: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(4)
            .map(char::from)
            .collect();
        let username = &format!("{}_{}", email, random_string);
        let new_user = NewUser {
            name,
            email,
            phone: "",
            username,
            overall_rating: &INITIAL_RATING,
            is_pragyan: &true,
            password: "",
            is_verified: &true,
            highest_rating: &INITIAL_RATING,
        };
        let user: User = diesel::insert_into(user::table)
            .values(&new_user)
            .get_result(conn)
            .map_err(|err| DieselError {
                table: "user",
                function: function!(),
                error: err,
            })?;
        Ok((user.id, user.username))
    }
}

pub fn set_otp_session_id(conn: &PgConnection, user_id: i32, session_id: &str) -> Result<()> {
    diesel::update(user::table.find(user_id))
        .set(user::otp_session_id.eq(&session_id))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(())
}

pub fn get_otp_session_id(conn: &PgConnection, user_id: i32) -> Result<String> {
    let session_id = user::table
        .find(user_id)
        .select(user::otp_session_id)
        .first::<String>(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(session_id)
}

pub fn verify_user(conn: &PgConnection, id: i32) -> Result<()> {
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

pub fn get_user_with_phone(conn: &PgConnection, phone: &str) -> Result<Option<User>> {
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
    pg_conn: &PgConnection,
    mut redis_conn: RedisConn,
    user_id: i32,
    password: &str,
) -> Result<()> {
    let hashed_password = bcrypt::hash(&password)?;
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
