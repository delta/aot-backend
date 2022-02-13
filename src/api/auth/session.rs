use crate::api::error::AuthError;
use actix_session::Session;

pub fn is_signed_in(session: &Session) -> bool {
    matches!(get_current_user(session), Ok(_))
}

pub fn get_current_user(session: &Session) -> Result<i32, AuthError> {
    let user_id = get_unverified_user(session)?;
    session
        .get::<bool>("is_verified")
        .map_err(|_| AuthError::UnVerifiedError)?
        .ok_or(AuthError::UnVerifiedError)?;
    Ok(user_id)
}

pub fn get_unverified_user(session: &Session) -> Result<i32, AuthError> {
    let user_id = session
        .get::<i32>("user")
        .map_err(|_| AuthError::SessionError)?
        .ok_or(AuthError::SessionError)?;
    Ok(user_id)
}
