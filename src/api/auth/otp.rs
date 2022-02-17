use actix_web::client::Client;
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

pub async fn send_otp(input_phone: &str, template_name: &str) -> Result<TwoFactorResponse> {
    let api_key = std::env::var("TWOFACTOR_API_TOKEN")?;
    let url = format!(
        "https://2factor.in/API/V1/{}/SMS/{}/AUTOGEN/{}",
        &api_key, &input_phone, &template_name
    );
    let response: TwoFactorResponse = Client::default()
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()
        .await?
        .json()
        .await?;
    if response.status == "Success"
        || response.details == "Invalid Phone Number - Length Mismatch(Expected >= 10)"
    {
        Ok(response)
    } else {
        return Err(anyhow::anyhow!("Error in sending OTP: {:?}", response).into());
    }
}

pub async fn verify_otp(session_id: &str, otp: &str) -> Result<TwoFactorResponse> {
    let api_key = std::env::var("TWOFACTOR_API_TOKEN")?;
    let url = format!(
        "http://2factor.in/API/V1/{}/SMS/VERIFY/{}/{}",
        &api_key, &session_id, &otp
    );
    let response: TwoFactorResponse = Client::default()
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
