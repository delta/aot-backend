use crate::api::auth::util::generate_otp;
use crate::api::RedisConn;
use redis::Commands;
use reqwest::Client;
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
    pub url: &'a str,
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
        .form(&ReCaptchaRequest { secret, response })
        .send()
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
    let callback_url = std::env::var("PLIVO_CALLBACK_URL")?;
    let api_token = std::env::var("PLIVO_AUTH_TOKEN")?;
    let sender_id = std::env::var("PLIVO_SENDER_ID")?;
    let url = format!("https://api.plivo.com/v1/Account/{}/Message/", &api_id);
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
    let user_name = std::env::var("PLIVO_AUTH_ID")?;
    let response = Client::default()
        .post(&url)
        .basic_auth(user_name, Some(api_token))
        .header("content-type", "application/json")
        .json(&OtpRequest {
            dst: input_phone,
            text: &otp_msg,
            src: &sender_id,
            message_type: "sms",
            url: &callback_url,
        })
        .send()
        .await?;
    if response.status().is_success() {
        let key = format!("{user_id}-otp");
        redis_conn.set(&key, otp)?;
        redis_conn.expire(&key, 120)?;
        return Ok(());
    }
    Err(anyhow::anyhow!("Error in sending OTP").into())
}

pub async fn verify_otp(
    otp: &str,
    mut redis_conn: RedisConn,
    user_id: i32,
) -> Result<OtpVerificationResponse> {
    let key = format!("{user_id}-otp");
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
