use crate::api::auth::util::generate_otp;
use actix_web::client::Client;
use r2d2::PooledConnection;
use redis::Client as RClient;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TwoFactorResponse {
    #[serde(rename = "Status")]
    pub status: String,
    #[serde(rename = "Details")]
    pub details: String,
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
    mut redis_conn: PooledConnection<RClient>,
    user_id: i32,
) -> Result<TwoFactorResponse> {
    let api_id = std::env::var("PLIVO_AUTH_ID")?;
    let api_token = std::env::var("PLIVO_AUTH_TOKEN")?;
    let sender_id = std::env::var("PLIVO_SENDER_ID")?;
    let url = format!("https://api.plivo.com/v1/Account/{}/Message/", &api_id);
    let mut header = "Basic ".to_owned();
    let encoded_value = base64::encode(api_id + ":" + &api_token);
    header.push_str(&encoded_value);
    let mut otp_msg =
        "Your One Time Password(OTP) for verification from team Attack On Robots is ".to_owned();
    let otp = generate_otp();
    otp_msg.push_str(&otp);
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
    if response.status().is_success() {
        let mut key = user_id.to_string();
        key.push_str("-otp");
        redis_conn.set(&key, otp)?;
        redis_conn.expire(&key, 60)?;
        return Ok(TwoFactorResponse {
            status: "Success".to_owned(),
            details: "Message sent".to_owned(),
        });
    }
    return Err(anyhow::anyhow!("Error in sending OTP").into());
}

pub async fn verify_otp(
    otp: &str,
    mut redis_conn: PooledConnection<RClient>,
    user_id: i32,
) -> Result<TwoFactorResponse> {
    let mut key = user_id.to_string();
    key.push_str("-otp");
    match redis_conn.get::<String, String>(key) {
        Ok(res) => {
            if res == *otp {
                Ok(TwoFactorResponse {
                    status: "Success".to_owned(),
                    details: "OTP Matched".to_owned(),
                })
            } else {
                Ok(TwoFactorResponse {
                    status: "Success".to_owned(),
                    details: "OTP MisMatch".to_owned(),
                })
            }
        }
        Err(_) => Ok(TwoFactorResponse {
            status: "Error".to_owned(),
            details: "OTP Expired".to_owned(),
        }),
    }
}
