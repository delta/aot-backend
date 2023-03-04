use crate::api::{error::AuthError, RedisConn, RedisPool};
use actix_session::{Session, SessionExt};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};
use futures::future::{err, ok, Ready};
use redis::Commands;
use std::time::{SystemTime, UNIX_EPOCH};

pub struct AuthUser(pub i32);

impl FromRequest for AuthUser {
    type Error = AuthError;
    type Future = Ready<Result<AuthUser, AuthError>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();
        let pool = req
            .app_data::<Data<RedisPool>>()
            .unwrap()
            .get_ref()
            .to_owned();
        match pool.get() {
            Ok(conn) => match get_authenticated_user(&session, conn) {
                Ok(user) => ok(AuthUser(user)),
                Err(error) => err(error),
            },
            Err(error) => err(AuthError::Internal(error.into())),
        }
    }
}

pub struct UnverifiedUser(pub i32);

impl FromRequest for UnverifiedUser {
    type Error = AuthError;
    type Future = Ready<Result<UnverifiedUser, AuthError>>;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let session = req.get_session();
        let pool = req
            .app_data::<Data<RedisPool>>()
            .unwrap()
            .get_ref()
            .to_owned();
        match pool.get() {
            Ok(conn) => match get_unverified_user(&session, conn) {
                Ok(user) => ok(UnverifiedUser(user)),
                Err(error) => err(error),
            },
            Err(error) => err(AuthError::Internal(error.into())),
        }
    }
}

fn get_authenticated_user(session: &Session, conn: RedisConn) -> Result<i32, AuthError> {
    let user_id = get_unverified_user(session, conn)?;
    session
        .get::<bool>("is_verified")
        .map_err(|_| AuthError::UnVerified)?
        .ok_or(AuthError::UnVerified)?;
    Ok(user_id)
}

fn get_unverified_user(session: &Session, mut conn: RedisConn) -> Result<i32, AuthError> {
    let user_id = session
        .get::<i32>("user")
        .map_err(|_| AuthError::Session)?
        .ok_or(AuthError::Session)?;
    let created_at = session
        .get::<u64>("created_at")
        .map_err(|_| AuthError::Session)?
        .ok_or(AuthError::Session)?;
    let last_pw_reset: Option<u64> = conn.get(user_id)?;
    if let Some(last_pw_reset) = last_pw_reset {
        if last_pw_reset < created_at {
            return Ok(user_id);
        } else {
            return Err(AuthError::Session);
        }
    }
    conn.set(user_id, created_at)?;
    Ok(user_id)
}

pub fn set(
    session: &Session,
    user_id: i32,
    is_verified: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    session.insert("user", user_id)?;
    session.insert("is_verified", is_verified)?;
    let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    session.insert("created_at", created_at)?;
    session.renew();
    Ok(())
}
