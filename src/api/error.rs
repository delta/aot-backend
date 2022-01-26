#[derive(Debug)]
pub struct DieselError(diesel::result::Error);

impl From<diesel::result::Error> for DieselError {
    fn from(error: diesel::result::Error) -> Self {
        DieselError(error)
    }
}

impl From<DieselError> for actix_web::Error {
    fn from(_: DieselError) -> Self {
        actix_web::error::ErrorInternalServerError("Internal Server Error")
    }
}
