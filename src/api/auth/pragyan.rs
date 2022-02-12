use anyhow::Result;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct PragyanRequest {
    user_email: String,
    user_pass: String,
    event_id: String,
    event_secret: String,
}

#[derive(Debug, Deserialize)]
pub struct PragyanUser {
    pub user_fullname: String,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum PragyanMessage {
    Success(PragyanUser),
    Error(String),
}

#[derive(Debug, Deserialize)]
pub struct PragyanResponse {
    pub status_code: u16,
    pub message: PragyanMessage,
}

pub fn auth(user_email: String, user_pass: String) -> Result<PragyanResponse> {
    let pragyan_login_url = std::env::var("PRAGYAN_LOGIN_URL")?;
    let event_id = std::env::var("PRAGYAN_EVENT_ID")?;
    let event_secret = std::env::var("PRAGYAN_EVENT_SECRET")?;
    let pragyan_response: PragyanResponse = Client::new()
        .post(&pragyan_login_url)
        .form(&PragyanRequest {
            user_email,
            user_pass,
            event_id,
            event_secret,
        })
        .send()?
        .json()?;
    Ok(pragyan_response)
}
