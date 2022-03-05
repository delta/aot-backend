use crate::api::auth::util::generate_otp;
use crate::api::RedisConn;
use actix_web::client::Client;
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
}

#[derive(Debug, Serialize)]
pub struct OtpRequest {
    pub dst: String,
    pub text: String,
    #[serde(rename = "type")]
    pub message_type: String,
    pub src: String,
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
    Ok(recaptcha_response.success)
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
        .header("authorization", header)
        .header("content-type", "application/json")
        .send_json(&serde_json::json!(OtpRequest {
            dst: input_phone.to_owned(),
            text: otp_msg,
            src: sender_id,
            message_type: "sms".to_string(),
        }))
        .await?;
    log::info!("{:?}", response);
    if response.status().is_success() {
        let mut key = user_id.to_string();
        key.push_str("-otp");
        redis_conn.set(&key, otp)?;
        redis_conn.expire(&key, 120)?;
        return Ok(());
    }
    return Err(anyhow::anyhow!("Error in sending OTP").into());
}

pub async fn verify_otp(otp: &str, mut redis_conn: RedisConn, user_id: i32) -> Result<&str> {
    let mut key = user_id.to_string();
    key.push_str("-otp");
    match redis_conn.get::<String, String>(key) {
        Ok(res) => {
            if res == *otp {
                Ok("OTP Matched")
            } else {
                Ok("OTP MisMatch")
            }
        }
        Err(_) => Ok("OTP Expired"),
    }
}
