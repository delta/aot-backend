use std::{
    env,
    future::{ready, Ready},
};

use actix_session::SessionExt;
use actix_web::{dev::Payload, error::ErrorUnauthorized, Error, FromRequest, HttpRequest};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};

use super::TokenClaims;

pub struct AuthenticationToken {
    pub id: i32,
}

impl FromRequest for AuthenticationToken {
    type Error = Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();

        let auth_token: String = match session.get::<String>("token") {
            Ok(auth_token) => match auth_token {
                Some(token) => token,
                None => return ready(Err(ErrorUnauthorized("UnAuthrized!!"))),
            },
            Err(_) => return ready(Err(ErrorUnauthorized("UnAuthrized!!"))),
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
            Err(_) => ready(Err(ErrorUnauthorized("you are UnAuthrized!!"))),
        }
    }
}
