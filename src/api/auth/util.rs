use crate::api::RedisConn;
use crate::models::*;
use crate::schema::user::{self};
use crate::util::function;
use crate::{constants::INITIAL_RATING, error::DieselError};
use anyhow::Result;
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use oauth2::basic::BasicClient;
use oauth2::{AuthUrl, ClientId, ClientSecret, RedirectUrl, TokenUrl};
use rand::{distributions::Alphanumeric, Rng};
use redis::Commands;
use std::env;

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

pub fn client() -> BasicClient {
    let google_client_id = ClientId::new(
        env::var("GOOGLE_OAUTH_CLIENT_ID").expect("Google oauth client id must be set!"),
    );
    let google_client_secret = ClientSecret::new(
        env::var("GOOGLE_OAUTH_CLIENT_SECRET").expect("Google oauth client secret must be set!"),
    );
    let auth_url = env::var("GOOGLE_OAUTH_AUTH_URL").expect("Google oauth auth URL must be set!");
    let token_url =
        env::var("GOOGLE_OAUTH_TOKEN_URL").expect("Google oauth token URL must be set!");

    let auth_url = AuthUrl::new(auth_url).expect("Invalid authorization endpoint URL");
    let token_url = TokenUrl::new(token_url).expect("Invalid token endpoint URL");

    // Set up the client for the Google OAuth2 process.
    let redirect_url =
        env::var("GOOGLE_OAUTH_REDIRECT_URL").expect("Google oauth redirect URL must be set!");

    BasicClient::new(
        google_client_id,
        Some(google_client_secret),
        auth_url,
        Some(token_url),
    )
    .set_redirect_uri(RedirectUrl::new(redirect_url).expect("Invalid redirect URL"))
}

pub fn generate_jwt_token(id: i32) -> Result<(String, String)> {
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

    Ok((token, exp.to_string()))
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
            username,
            is_pragyan: &true,
            attacks_won: &0,
            defenses_won: &0,
            trophies: &INITIAL_RATING,
            avatar_id: &0,
            artifacts: &0,
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
            username,
            is_pragyan: &true,
            attacks_won: &0,
            defenses_won: &0,
            trophies: &INITIAL_RATING,
            avatar_id: &0,
            artifacts: &0,
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
