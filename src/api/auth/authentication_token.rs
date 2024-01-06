use std::{
    env,
    future::{ready, Ready},
};

use actix_web::{
    cookie::Cookie, dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest,
};
use awc::error::HeaderValue;
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use reqwest::header::{AUTHORIZATION, COOKIE};
use serde::{Deserialize, Serialize};

use super::TokenClaims;

#[derive(Serialize, Deserialize)]
pub struct AuthenticationToken {
    pub id: i32,
}

impl FromRequest for AuthenticationToken {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        //get auth token from the autherization header

        let auth_header: Option<&HeaderValue> = req.headers().get(AUTHORIZATION);
        //todo better error handling if auth_header is missing
        let auth_token: String = match auth_header {
            Some(auth_header) => auth_header.to_str().unwrap().to_string(),
            None => {
                let auth_header = req.headers().get(COOKIE);
                if let Some(auth_header) = auth_header {
                    let cookie: Cookie = match auth_header.to_str().unwrap().parse() {
                        Ok(cookie) => cookie,
                        Err(_) => return ready(Err(ErrorUnauthorized("invalid cookie!!"))),
                    };
                    cookie.value().to_string()
                } else {
                    return ready(Err(ErrorUnauthorized("UnAuthrized!!")));
                }
            }
        };

        if auth_token.is_empty() {
            return ready(Err(ErrorUnauthorized("No auth token provided")));
        }

        let secret: String = env::var("JWT_SECRET").unwrap_or("".to_string());

        let decode = decode::<TokenClaims>(
            &auth_token,
            &DecodingKey::from_secret(secret.as_str().as_ref()),
            &Validation::new(Algorithm::HS256),
        );

        match decode {
            Ok(token) => ready(Ok(AuthenticationToken {
                id: token.claims.id,
            })),
            Err(_) => ready(Err(ErrorUnauthorized("UnAuthrized!!"))),
        }
    }
}
