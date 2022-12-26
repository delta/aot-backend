use crate::api::auth::util::generate_otp;
use crate::api::RedisConn;
use awc::Client;
use redis::Commands;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct ReCaptchaRequest {
    secret: String,
    response: String,
}

#[derive(Debug, Deserialize)]
struct ReCaptchaResponse {
    success: bool,
    score: f32,
}

#[derive(Debug, Serialize)]
pub struct OtpRequest<'a> {
    pub dst: &'a str,
    pub text: &'a str,
    #[serde(rename = "type")]
    pub message_type: &'a str,
    pub src: &'a str,
}

pub enum OtpVerificationResponse {
    MisMatch,
    Match,
    Expired,
}

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub async fn verify_recaptcha(response: String) -> Result<bool> {
    let secret = std::env::var("RECAPTCHA_SECRET")?;
    let recaptcha_response: ReCaptchaResponse = Client::default()
        .post("https://www.google.com/recaptcha/api/siteverify")
        .send_form(&ReCaptchaRequest { secret, response })
        .await?
        .json()
        .await?;
    Ok(recaptcha_response.success && recaptcha_response.score > 0.5)
}

pub async fn send_otp(
    input_phone: &str,
    mut redis_conn: RedisConn,
    user_id: i32,
    is_account_verification: bool,
) -> Result<()> {
    let api_id = std::env::var("PLIVO_AUTH_ID")?;
    let api_token = std::env::var("PLIVO_AUTH_TOKEN")?;
    let sender_id = std::env::var("PLIVO_SENDER_ID")?;
    let url = format!("https://api.plivo.com/v1/Account/{}/Message/", &api_id);
    let header = format!("Basic {}", base64::encode(api_id + ":" + &api_token));
    let otp = generate_otp();
    let otp_msg = match is_account_verification {
        true => format!(
            "Your One Time Password(OTP) for Account verification from team Attack On Robots is {}",
            &otp
        ),
        false => format!(
            "Your One Time Password(OTP) for Resetting password from team Attack On Robots is {}",
            &otp
        ),
    };
    let response = Client::default()
        .post(&url)
        .insert_header(("authorization", header))
        .insert_header(("content-type", "application/json"))
        .send_json(&OtpRequest {
            dst: input_phone,
            text: &otp_msg,
            src: &sender_id,
            message_type: "sms",
        })
        .await?;
    if response.status().is_success() {
        let key = format!("{}-otp", user_id);
        redis_conn.set(&key, otp)?;
        redis_conn.expire(&key, 120)?;
        return Ok(());
    }
    return Err(anyhow::anyhow!("Error in sending OTP").into());
}

pub async fn verify_otp(
    otp: &str,
    mut redis_conn: RedisConn,
    user_id: i32,
) -> Result<OtpVerificationResponse> {
    let key = format!("{}-otp", user_id);
    match redis_conn.exists::<&str, bool>(&key) {
        Ok(res) => {
            if res {
                match redis_conn.get::<&str, String>(&key) {
                    Ok(res) => {
                        if res == otp {
                            Ok(OtpVerificationResponse::Match)
                        } else {
                            Ok(OtpVerificationResponse::MisMatch)
                        }
                    }
                    Err(err) => Err(err.into()),
                }
            } else {
                Ok(OtpVerificationResponse::Expired)
            }
        }
        Err(err) => Err(err.into()),
    }
}
