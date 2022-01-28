use actix_web::error::ErrorInternalServerError;
use log;

pub fn handle_error(err: Box<dyn std::error::Error>) -> actix_web::Error {
    log::error!("{:?}", err);
    ErrorInternalServerError("Internal Server Error")
}
