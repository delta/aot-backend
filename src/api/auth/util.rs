use crate::api::RedisConn;
use crate::models::*;
use crate::schema::user::{self};
use crate::util::function;
use crate::{constants::INITIAL_RATING, error::DieselError};
use actix_web::cookie::{time::Duration as ActixWebDuration, Cookie};
use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;
use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use super::TokenClaims;

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
pub fn get_user_by_user_id(conn: &mut PgConnection, user_id: &i32) -> Result<Option<User>> {
    let user = user::table
        .filter(user::id.eq(user_id))
        .first::<User>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;
    Ok(user)
}

pub fn client() -> BasicClient {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_OAUTH_CLIENT_ID")
            .expect("Google oauth client id must be set!")
            .to_string(),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_OAUTH_CLIENT_SECRET")
            .expect("Google oauth client secret must be set!")
            .to_string(),
    );
    let auth_url = AuthUrl::new("https://accounts.google.com/o/oauth2/v2/auth".to_string())
        .expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new("https://www.googleapis.com/oauth2/v3/token".to_string())
        .expect("Invalid token endpoint URL");

    // Set up the client for the Google OAuth2 process.
    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(
        RedirectUrl::new("http://localhost:8000/user/gauth2/callback".to_string())
            .expect("Invalid redirect URL"),
    )
}

pub fn generate_jwt_token_and_cookie(id: i32) -> Result<(String, String, DateTime<Utc>)> {
    let jwt_secret = env::var("JWT_SECRET").expect("JWT secret must be set!");
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let jwt_max_age: i64 = env::var("JWT_MAX_AGE")
        .expect("JWT max age must be set!")
        .parse()
        .expect("JWT max age must be an integer!");
    let token_expiring_time = now + Duration::minutes(jwt_max_age);
    let exp = (token_expiring_time).timestamp() as usize;
    let claims: TokenClaims = TokenClaims { id, exp, iat };

    let token_result = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(jwt_secret.as_ref()),
    );
    let token = match token_result {
        Ok(token) => token,
        Err(e) => return Err(e.into()),
    };
    let cookie = Cookie::build("token", token.clone())
        .path("/")
        .max_age(ActixWebDuration::new(60 * jwt_max_age, 0))
        .http_only(true)
        .finish();

    Ok((token, cookie.to_string(), token_expiring_time))
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
pub fn get_oauth_user(
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
