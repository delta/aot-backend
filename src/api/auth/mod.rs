use self::pragyan::PragyanMessage;
use crate::api::error;
use actix_session::Session;
use actix_web::error::{ErrorBadRequest, ErrorUnauthorized};
use actix_web::web::{self, Data, Json};
use actix_web::Responder;
use actix_web::{HttpResponse, Result};
use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use pwhash::bcrypt;
use serde::Deserialize;

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

#[derive(Deserialize)]
struct OtpRequest {
    username: String,
    recaptcha: String,
}

#[derive(Deserialize)]
struct OtpVerifyRequest {
    username: String,
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

type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;

async fn login(
    request: web::Json<LoginRequest>,
    session: Session,
    pool: Data<Pool>,
) -> Result<impl Responder> {
    if session::is_signed_in(&session) {
        return Ok("Already signed in");
    }
    let username = request.username.clone();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::get_user(&conn, &username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(user) = user {
        if !user.is_pragyan && bcrypt::verify(&request.password, &user.password) {
            if user.is_verified {
                session
                    .set("user", user.id)
                    .map_err(|err| error::handle_error(err.into()))?;
                return Ok("Successfully Logged In");
            } else {
                // Account not verified
                return Err(ErrorUnauthorized("App account not verified"));
            }
        }
    }

    let LoginRequest { username, password } = request.into_inner();
    // Pragyan users need to login with email
    let email = username.clone();
    let pragyan_auth = web::block(|| pragyan::auth(email, password))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    match pragyan_auth.status_code {
        200 => {
            if let PragyanMessage::Success(pragyan_user) = pragyan_auth.message {
                let user_id = web::block(move || {
                    let conn = pool.get()?;
                    let email = username.clone();
                    let name = pragyan_user.user_fullname;
                    util::get_pragyan_user(&conn, &email, &name)
                })
                .await
                .map_err(|err| error::handle_error(err.into()))?;
                session
                    .set("user", user_id)
                    .map_err(|err| error::handle_error(err.into()))?;
                Ok("Successfully Logged In")
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

async fn sendotp(pool: Data<Pool>, request: Json<OtpRequest>) -> Result<impl Responder> {
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let username = request.username.clone();
    let user = web::block(move || util::get_user(&conn, &username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if let Some(user) = user {
        if user.is_verified {
            return Err(ErrorBadRequest("Account already verified"));
        }
    } else {
        return Err(ErrorBadRequest("User not found"));
    }

    let request = request.into_inner();

    let is_valid_recatpcha = web::block(|| otp::verify_recaptcha(request.recaptcha))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let username = request.username.clone();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let phone_number = web::block(move || util::get_user_ph_no(&conn, &username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    let two_factor_response = web::block(move || {
        let template_name = std::env::var("TWOFACTOR_VERIFY_TEMPLATE")?;
        otp::send_otp(&phone_number, &template_name)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    if two_factor_response.status == "Success" {
        web::block(move || {
            let conn = pool.get()?;
            let username = request.username.clone();
            util::set_otp_session_id(&conn, &username, &two_factor_response.details)
        })
        .await
        .map_err(|err| error::handle_error(err.into()))?;
        Ok("OTP sent successfully")
    } else {
        Err(ErrorBadRequest("Invalid phone number"))
    }
}

async fn verify(pool: Data<Pool>, request: Json<OtpVerifyRequest>) -> Result<impl Responder> {
    let OtpVerifyRequest {
        username,
        otp,
        recaptcha,
    } = request.into_inner();
    if otp.len() < 4 || otp.len() > 6 {
        return Err(ErrorBadRequest("Invalid OTP"));
    }
    let _username = username.clone();
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let user = web::block(move || util::get_user(&conn, &_username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("User not found"));
    }

    let is_valid_recatpcha = web::block(|| otp::verify_recaptcha(recaptcha))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let _username = username.clone();
    let session_id = web::block(move || util::get_otp_session_id(&conn, &_username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    let two_factor_response = web::block(move || otp::verify_otp(&session_id, &otp))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    match two_factor_response.details.as_str() {
        "OTP Matched" => {
            web::block(move || {
                let conn = pool.get()?;
                util::verify_user(&conn, &username)
            })
            .await
            .map_err(|err| error::handle_error(err.into()))?;
            Ok("Account successfully verified")
        }
        "OTP Mismatch" => Err(ErrorUnauthorized("OTP Mismatch")),
        "OTP Expired" => Err(ErrorUnauthorized("OTP Expired")),
        _ => Err(ErrorBadRequest("Invalid SessionId")),
    }
}

async fn send_resetpw_otp(
    pool: Data<Pool>,
    request: Json<ResetPwRequest>,
) -> Result<impl Responder> {
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let phone_number = request.phone_number.clone();
    let user = web::block(move || util::get_user_with_phone(&conn, &phone_number))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("Invalid phone number"));
    }

    let request = request.into_inner();

    let is_valid_recatpcha = web::block(|| otp::verify_recaptcha(request.recaptcha))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let two_factor_response = web::block(move || {
        let template_name = std::env::var("TWOFACTOR_RESETPW_TEMPLATE")?;
        let phone_number = request.phone_number;
        otp::send_otp(&phone_number, &template_name)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    if two_factor_response.status == "Success" {
        web::block(move || {
            let conn = pool.get()?;
            let username = user.unwrap().username;
            util::set_otp_session_id(&conn, &username, &two_factor_response.details)
        })
        .await
        .map_err(|err| error::handle_error(err.into()))?;
        Ok("OTP sent successfully")
    } else {
        Err(ErrorBadRequest("Invalid phone number"))
    }
}

async fn reset_pw(pool: Data<Pool>, request: Json<ResetPwVerifyRequest>) -> Result<impl Responder> {
    let ResetPwVerifyRequest {
        phone_number,
        otp,
        password,
        recaptcha,
    } = request.into_inner();
    if otp.len() < 4 || otp.len() > 6 {
        return Err(ErrorBadRequest("Invalid OTP"));
    }
    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let phone = phone_number.clone();
    let user = web::block(move || util::get_user_with_phone(&conn, &phone))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if user.is_none() {
        return Err(ErrorBadRequest("Invalid phone number"));
    }

    let is_valid_recatpcha = web::block(|| otp::verify_recaptcha(recaptcha))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    if !is_valid_recatpcha {
        return Err(ErrorUnauthorized("Invalid reCAPTCHA"));
    }

    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let username = user.unwrap().username;
    let session_id = web::block(move || util::get_otp_session_id(&conn, &username))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    let two_factor_response = web::block(move || otp::verify_otp(&session_id, &otp))
        .await
        .map_err(|err| error::handle_error(err.into()))?;
    match two_factor_response.details.as_str() {
        "OTP Matched" => {
            web::block(move || {
                let conn = pool.get()?;
                util::reset_password(&conn, &phone_number, &password)
            })
            .await
            .map_err(|err| error::handle_error(err.into()))?;
            Ok("Password reset successfully")
        }
        "OTP Mismatch" => Err(ErrorUnauthorized("OTP Mismatch")),
        "OTP Expired" => Err(ErrorUnauthorized("OTP Expired")),
        _ => Err(ErrorBadRequest("Invalid SessionId")),
    }
}
