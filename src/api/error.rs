use actix_web::{
    error::{ErrorBadRequest, ErrorInternalServerError, ErrorUnauthorized},
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

#[derive(Debug, Display, Error)]
pub enum BaseInvalidError {
    InvalidBlockType(i32),
    #[display(fmt = "{:?}", self)]
    InvalidRotation(String, i32),
    OverlappingBlocks,
    BlockOutsideMap,
    BlockCountExceeded(String),
    BlocksUnused(String),
    NotConnected,
}

impl ResponseError for BaseInvalidError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let response_body = match self {
            BaseInvalidError::InvalidBlockType(block_type) => {
                format!("Invalid block type: {}", block_type)
            }
            BaseInvalidError::InvalidRotation(block_type, rotation) => {
                format!("Invalid rotation {} for a block of type {}", rotation, block_type)
            }
            BaseInvalidError::OverlappingBlocks => "City has overlapping roads or buildings".to_string(),
            BaseInvalidError::BlockOutsideMap => "A road or building is placed outside of city".to_string(),
            BaseInvalidError::BlockCountExceeded(block_type) => {
                format!("You have exceeded the maximum number of {} buildings", block_type)
            }
            BaseInvalidError::BlocksUnused(block_type) => {
                format!("You have some unused {} buildings. Use all of them.", block_type)
            }
            BaseInvalidError::NotConnected => "City is not fully connected. Make sure all buildings are reachable from one another.".to_string(),
        };
        ErrorBadRequest(response_body).into()
    }
}

pub fn handle_error(err: Box<dyn std::error::Error>) -> actix_web::Error {
    log::error!("{}", err);
    ErrorInternalServerError("Internal Server Error")
}
