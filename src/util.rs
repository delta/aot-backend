use crate::error::*;
use crate::models::User;
use actix_session::Session;
use actix_web::{http::header::CONTENT_TYPE, HttpRequest};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

pub fn get_connection_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let type_name = type_name_of(f);
        &type_name[..type_name.len() - 3].trim_end_matches("::{{closure}}")
    }};
}

pub(crate) use function;
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn is_json_request(req: &HttpRequest) -> bool {
    req.headers().get(CONTENT_TYPE).map_or(false, |header| {
        header
            .to_str()
            .map_or(false, |content_type| "application/json" == content_type)
    })
}

pub fn is_signed_in(session: &Session) -> bool {
    matches!(get_current_user(session), Ok(_))
}

pub fn set_current_user(session: &Session, user: User) {
    // serializing to string is alright for this case,
    // but binary would be preferred in production use-cases.
    session
        .set("user", serde_json::to_string(&user).unwrap())
        .unwrap();
}
pub fn get_current_user(session: &Session) -> Result<User, AuthError> {
    let err = AuthError::AuthenticationError(String::from("Could not retrieve user from session"));
    let session_result = session.get::<String>("user"); // Returns Result<Option<String>, Error>
    if session_result.is_err() {
        return Err(err);
    }
    session_result
        .unwrap()
        .map_or(Err(err.clone()), |user_str| {
            serde_json::from_str(&user_str).map_err(|_| err)
        })
}
