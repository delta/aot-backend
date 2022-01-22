use actix_web::{error::ResponseError, HttpResponse};
use derive_more::Display;
use thiserror::Error;

#[derive(Clone, Debug, Display)]
pub enum AuthError {
    #[display(fmt = "AuthenticationError: {}", _0)]
    AuthenticationError(String),
}
impl ResponseError for AuthError {
    fn error_response(&self) -> HttpResponse {
        match self {
            AuthError::AuthenticationError(ref message) => {
                HttpResponse::Unauthorized().json(message)
            }
        }
    }
}

#[derive(Debug, Display, Error)]
#[display(fmt = "{:?}", self)]
pub struct DieselError<'a> {
    pub table: &'a str,
    pub function: &'a str,
    pub error: diesel::result::Error,
}
