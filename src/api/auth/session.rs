use crate::api::error::SessionError;
use actix_session::Session;

pub fn is_signed_in(session: &Session) -> bool {
    matches!(get_current_user(session), Ok(_))
}

pub fn get_current_user(session: &Session) -> Result<i32, SessionError> {
    let user_id = session
        .get::<i32>("user")
        .map_err(|_| SessionError)?
        .ok_or(SessionError)?;
    Ok(user_id)
}
