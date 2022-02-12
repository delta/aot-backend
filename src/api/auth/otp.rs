use anyhow::Result;
use reqwest::blocking::Client;
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

pub fn verify_recaptcha(response: String) -> Result<bool> {
    let secret = std::env::var("RECAPTCHA_SECRET")?;
    let recaptcha_response: ReCaptchaResponse = Client::new()
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&ReCaptchaRequest { secret, response })
        .send()?
        .json()?;
    Ok(recaptcha_response.success)
}

pub fn send_otp(input_phone: &str, template_name: &str) -> Result<TwoFactorResponse> {
    let api_key = std::env::var("TWOFACTOR_API_TOKEN")?;
    let url = format!(
        "https://2factor.in/API/V1/{}/SMS/{}/AUTOGEN/{}",
        &api_key, &input_phone, &template_name
    );
    let response: TwoFactorResponse = Client::new()
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()?
        .json()?;
    if response.status == "Success"
        || response.details == "Invalid Phone Number - Length Mismatch(Expected >= 10)"
    {
        Ok(response)
    } else {
        Err(anyhow::anyhow!("Error in sending OTP: {:?}", response))
    }
}

pub fn verify_otp(session_id: &str, otp: &str) -> Result<TwoFactorResponse> {
    let api_key = std::env::var("TWOFACTOR_API_TOKEN")?;
    let url = format!(
        "http://2factor.in/API/V1/{}/SMS/VERIFY/{}/{}",
        &api_key, &session_id, &otp
    );
    let response: TwoFactorResponse = Client::new()
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()?
        .json()?;
    Ok(response)
}
