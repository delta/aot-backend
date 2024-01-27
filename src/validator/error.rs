#[derive(Debug, Display, Error)]
pub enum AuthError {
    Session,
    UnVerified,
    UserNotFound,
    Internal(Box<dyn std::error::Error + Send + Sync>),
}

impl ResponseError for AuthError {
    fn error_response(&self) -> actix_web::HttpResponse {
        match self {
            AuthError::Session => ErrorUnauthorized("Session Error. Please login again.").into(),
            AuthError::UnVerified => ErrorUnauthorized("Please verify your account.").into(),
            AuthError::UserNotFound => ErrorNotFound("User Not Found").into(),
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
    #[display(fmt = "{self:?}")]
    InvalidBuildingType(i32),
    OverlappingBlocks,
    BlockOutsideMap,
    RoundRoad,
    BlockCountExceeded(i32),
    BlocksUnused(String),
    NotConnected(String),
}

impl ResponseError for BaseInvalidError {
    fn error_response(&self) -> actix_web::HttpResponse {
        let response_body = match self {
            BaseInvalidError::InvalidBlockType(block_type) => {
                format!("Invalid block type: {block_type}")
            }
            BaseInvalidError::InvalidBuildingType(building_id) => {
                format!("City has invalid building of type {building_id} placed")
            }
            BaseInvalidError::OverlappingBlocks => {
                "City has overlapping roads or buildings".to_string()
            }
            BaseInvalidError::BlockOutsideMap => {
                "A road or building is placed outside of city".to_string()
            }
            BaseInvalidError::BlockCountExceeded(block_type) => {
                format!("You have exceeded the maximum number of building of type {block_type}")
            }
            BaseInvalidError::BlocksUnused(block_type) => {
                format!("You have some unused {block_type} buildings. Use all of them.")
            }
            BaseInvalidError::NotConnected(no_path_info) => no_path_info.to_string(),
            BaseInvalidError::RoundRoad => "A 4x4 Square Cannot have all as Road".to_string(),
        };
        ErrorBadRequest(response_body).into()
    }
}

pub fn handle_error(err: Box<dyn std::error::Error>) -> actix_web::Error {
    log::error!("{}", err);
    ErrorInternalServerError("Internal Server Error")
}
