use actix_web::{
    error::{ErrorInternalServerError, ErrorUnauthorized},
    ResponseError,
};
use derive_more::Display;
use log;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub enum AuthError {
    Session,
    UnVerified,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AuthError::Session => ErrorUnauthorized("Session Error. Please login again.").into(),
            AuthError::UnVerified => ErrorUnauthorized("Please verify your account.").into(),
            AuthError::Internal(err) => handle_error(err.to_string().into()).into(),
        }
    }
}

impl From<redis::RedisError> for AuthError {
    fn from(err: redis::RedisError) -> Self {
        AuthError::Internal(err.into())
    }
}

impl From<r2d2::Error> for AuthError {
    fn from(err: r2d2::Error) -> Self {
        AuthError::Internal(err.into())
    }
}

pub fn handle_error(err: Box<dyn std::error::Error>) -> actix_web::Error {
    log::error!("{}", err);
    ErrorInternalServerError("Internal Server Error")
}
