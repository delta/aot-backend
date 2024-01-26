use std::{
    env,
    future::{ready, Ready},
};

use actix_session::SessionExt;
use actix_web::{dev::Payload, web::Data, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use redis::Commands;

use crate::api::{error, RedisPool};

use super::TokenClaims;

pub struct AuthUser(pub i32);

impl FromRequest for AuthUser {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();
        let redis_pool: Data<RedisPool> = req.app_data::<Data<RedisPool>>().unwrap().clone();
        let mut redis_conn = match redis_pool.get() {
            Ok(conn) => conn,
            Err(_) => return ready(Err(error::AuthError::Session.into())),
        };

        let auth_token: String = match session.get::<String>("token") {
            Ok(auth_token) => match auth_token {
                Some(token) => token,
                None => return ready(Err(error::AuthError::Session.into())),
            },
            Err(_) => return ready(Err(error::AuthError::Session.into())),
        };

        if auth_token.is_empty() {
            return ready(Err(error::AuthError::Session.into()));
        }

        let secret: String = env::var("COOKIE_KEY").unwrap_or("".to_string());

        let token = match decode::<TokenClaims>(
            &auth_token,
            &DecodingKey::from_secret(secret.as_str().as_ref()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token) => token,
            Err(_) => return ready(Err(error::AuthError::Session.into())),
        };

        let user_id = token.claims.id;
        let device = token.claims.device;
        let device_from_token: String = match redis_conn.get(user_id) {
            Ok(device_id) => device_id,
            Err(_) => return ready(Err(error::AuthError::Session.into())),
        };
        if device != *device_from_token {
            ready(Err(error::AuthError::Session.into()))
        } else {
            ready(Ok(AuthUser(user_id)))
        }
    }
}
