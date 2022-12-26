use self::pragyan::PragyanMessage;
use self::session::UnverifiedUser;
use super::{PgPool, RedisPool};
use crate::api::auth::otp::OtpVerificationResponse;
use crate::api::error;
use actix_session::Session;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::web::{self, Data, Json};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};

mod otp;
mod pragyan;
pub mod session;
mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)))
        .service(web::resource("/logout").route(web::post().to(logout)))
        .service(web::resource("/sendotp").route(web::post().to(sendotp)))
        .service(web::resource("/verify").route(web::post().to(verify)))
        .service(web::resource("/resetpw/sendotp").route(web::post().to(send_resetpw_otp)))
        .service(web::resource("/resetpw/verify").route(web::post().to(reset_pw)));
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    user_id: i32,
    username: String,
    name: String,
}

#[derive(Deserialize)]
struct OtpRequest {
    recaptcha: String,
}

#[derive(Deserialize)]
struct OtpVerifyRequest {
    otp: String,
    recaptcha: String,
}

#[derive(Deserialize)]
struct ResetPwRequest {
    phone_number: String,
    recaptcha: String,
}

#[derive(Deserialize)]
struct ResetPwVerifyRequest {
    phone_number: String,
    otp: String,
    password: String,
    recaptcha: String,
}

async fn login(
    request: web::Json<LoginRequest>,
    session: Session,
    pg_pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let username = request.username.clone();
    let mut pg_conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;

    let user = web::block(move || util::get_user_by_username(&mut pg_conn, &username))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(user) = user {
        if !user.is_pragyan && bcrypt::verify(&request.password, &user.password) {
            session::set(&session, user.id, user.is_verified)
                .map_err(|err| error::handle_error(err))?;
            if user.is_verified {
                return Ok(Json(LoginResponse {
                    user_id: user.id,
                    username: user.username,
                    name: user.name,
                }));
            }
            // Account not verified
            return Err(ErrorUnauthorized("App account not verified"));
        }
    }

    let LoginRequest { username, password } = request.into_inner();
    // Pragyan users need to login with email
    let email = username.to_lowercase();
    let pragyan_auth = pragyan::auth(email, password)
        .await
        .map_err(|err| error::handle_error(err))?;
    match pragyan_auth.status_code {
        200 => {
            if let PragyanMessage::Success(pragyan_user) = pragyan_auth.message {
                let name = pragyan_user.user_fullname.clone();
                let (user_id, username) = web::block(move || {
                    let mut conn = pg_pool.get()?;
                    let mut redis_conn = redis_pool.get()?;
                    let email = username.clone();
                    util::get_pragyan_user(&mut conn, &mut redis_conn, &email, &name)
                })
                .await?
                .map_err(|err| error::handle_error(err.into()))?;
                session::set(&session, user_id, true).map_err(|err| error::handle_error(err))?;
                Ok(Json(LoginResponse {
                    user_id,
                    username,
                    name: pragyan_user.user_fullname,
                }))
            } else {
                Err(anyhow::anyhow!(
                    "Unexpected error in Pragyan auth: {:?}",
                    pragyan_auth
                ))
                .map_err(|err| error::handle_error(err.into()))?
            }
        }
        203 => Err(ErrorUnauthorized("Pragyan account not verified")),
        _ => Err(ErrorUnauthorized(
            "Invalid username/Pragyan email or password",
        )),
    }
}

async fn logout(session: Session) -> impl Responder {
    session.clear();
    HttpResponse::NoContent().finish()
}

async fn sendotp(
    pool: Data<PgPool>,
    redis_pool: Data<RedisPool>,
    request: Json<OtpRequest>,
    user: UnverifiedUser,
) -> Result<impl Responder> {
    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let user_id = user.0;
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::get_user(&mut conn, user_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(ref user) = user {
        if user.is_verified {
            return Err(ErrorBadRequest("Account already verified"));
        }
    } else {
        return Err(ErrorBadRequest("User not found"));
    }
    let user = user.unwrap();

    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let phone_number = user.clone().phone;
    let duplicate_user = web::block(move || util::get_user_with_phone(&mut conn, &phone_number))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if duplicate_user.is_some() {
        return Err(ErrorBadRequest("Phone number already registered"));
    }

    let request = request.into_inner();
    let is_valid_recatpcha = otp::verify_recaptcha(request.recaptcha)
        .await
        .map_err(|err| error::handle_error(err))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let phone_number = user.phone;
    otp::send_otp(&phone_number, redis_conn, user_id, true)
        .await
        .map_err(|err| error::handle_error(err))?;
    Ok("OTP sent successfully")
}

async fn verify(
    pool: Data<PgPool>,
    request: Json<OtpVerifyRequest>,
    user: UnverifiedUser,
    session: Session,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let user_id = user.0;
    let OtpVerifyRequest { otp, recaptcha } = request.into_inner();
    if otp.len() != 5 {
        return Err(ErrorBadRequest("Invalid OTP"));
    }
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::get_user(&mut conn, user_id))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("User not found"));
    }

    let is_valid_recatpcha = otp::verify_recaptcha(recaptcha)
        .await
        .map_err(|err| error::handle_error(err))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let user_id = user.unwrap().id;
    let otp_response = otp::verify_otp(&otp, redis_conn, user_id)
        .await
        .map_err(|err| error::handle_error(err))?;
    match otp_response {
        OtpVerificationResponse::Match => {
            web::block(move || {
                let mut conn = pool.get()?;
                util::verify_user(&mut conn, user_id)
            })
            .await?
            .map_err(|err| error::handle_error(err.into()))?;
            session
                .insert("is_verified", true)
                .map_err(|err| error::handle_error(err.into()))?;
            Ok("Account successfully verified")
        }
        OtpVerificationResponse::Expired => Err(ErrorUnauthorized("OTP Expired")),
        OtpVerificationResponse::MisMatch => Err(ErrorUnauthorized("OTP Mismatch")),
    }
}

async fn send_resetpw_otp(
    pool: Data<PgPool>,
    request: Json<ResetPwRequest>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let phone_number = request.phone_number.clone();
    let user = web::block(move || util::get_user_with_phone(&mut conn, &phone_number))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("Invalid phone number"));
    }

    let request = request.into_inner();

    let is_valid_recatpcha = otp::verify_recaptcha(request.recaptcha)
        .await
        .map_err(|err| error::handle_error(err))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let phone_number = request.phone_number;
    otp::send_otp(&phone_number, redis_conn, user.unwrap().id, false)
        .await
        .map_err(|err| error::handle_error(err))?;
    Ok("OTP sent successfully")
}

async fn reset_pw(
    pg_pool: Data<PgPool>,
    request: Json<ResetPwVerifyRequest>,
    redis_pool: Data<RedisPool>,
) -> Result<impl Responder> {
    let ResetPwVerifyRequest {
        phone_number,
        otp,
        password,
        recaptcha,
    } = request.into_inner();
    if otp.len() != 5 {
        return Err(ErrorBadRequest("Invalid OTP"));
    }
    let mut pg_conn = pg_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let redis_conn = redis_pool
        .get()
        .map_err(|err| error::handle_error(err.into()))?;
    let phone = phone_number.clone();
    let user = web::block(move || util::get_user_with_phone(&mut pg_conn, &phone))
        .await?
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("Invalid phone number"));
    }

    let is_valid_recatpcha = otp::verify_recaptcha(recaptcha)
        .await
        .map_err(|err| error::handle_error(err))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }
    let user_id = user.unwrap().id;
    let otp_response = otp::verify_otp(&otp, redis_conn, user_id)
        .await
        .map_err(|err| error::handle_error(err))?;
    match otp_response {
        OtpVerificationResponse::Match => {
            web::block(move || {
                let mut conn = pg_pool.get()?;
                let redis_conn = redis_pool.get()?;
                util::reset_password(&mut conn, redis_conn, user_id, &password)
            })
            .await?
            .map_err(|err| error::handle_error(err.into()))?;
            Ok("Password reset successfully")
        }
        OtpVerificationResponse::Expired => Err(ErrorUnauthorized("OTP Expired")),
        OtpVerificationResponse::MisMatch => Err(ErrorUnauthorized("OTP Mismatch")),
    }
}
