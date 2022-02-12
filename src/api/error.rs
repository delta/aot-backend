use actix_web::error::ErrorInternalServerError;
use actix_web::ResponseError;
use derive_more::Display;
use log;
use thiserror::Error;

#[derive(Debug, Display, Error)]
pub struct SessionError;

impl ResponseError for SessionError {
    fn error_response(&self) -> actix_web::HttpResponse {
        actix_web::HttpResponse::Unauthorized().body("Session Error. Please login again.")
    }
}

pub fn handle_error(err: Box<dyn std::error::Error>) -> actix_web::Error {
    log::error!("{}", err);
    ErrorInternalServerError("Internal Server Error")
}
