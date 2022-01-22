use crate::diesel::QueryDsl;
use crate::diesel::RunQueryDsl;
use crate::models::{NewUser, User};
use crate::schema::user::dsl::*;
use crate::util::*;
use actix_session::Session;
use actix_web::{http::header::LOCATION, web, Error, HttpRequest, HttpResponse};
use diesel::dsl::{insert_into, update};
use diesel::expression_methods::ExpressionMethods;
use pwhash::bcrypt;
use serde::{Deserialize, Serialize};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)));
    cfg.service(web::resource("/register").route(web::post().to(register)));
    cfg.service(web::resource("/register/verify").route(web::post().to(verification)));
    cfg.service(web::resource("/logout").route(web::get().to(logout)));
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InputUser {
    pub name: String,
    pub email: String,
    pub phone: String,
    pub username: String,
    pub password: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize)]
pub struct OtpSessionId {
    pub Status: String,
    pub Details: String,
}

#[derive(Deserialize)]
pub struct Otp {
    pub otp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}

#[allow(non_snake_case)]
//Handler for POST /login
async fn login(
    data: web::Json<AuthData>,
    session: Session,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    match is_signed_in(&session) {
        true => {
            let response = get_current_user(&session)
                .map(|User| HttpResponse::Ok().json(User))
                .unwrap();

            Ok(response)
        }
        false => handle_sign_in(data.into_inner(), &session, &pool),
    }
}

//Handler for POST /register
#[allow(non_snake_case)]
async fn register(db: web::Data<Pool>, item: web::Json<InputUser>) -> Result<HttpResponse, Error> {
    let result = web::block(move || add_user(db, item)).await;
    match result {
        Ok(User) => Ok(web::block(move || send_otp(&User.phone))
            .await
            .map(|OtpSessionId| HttpResponse::Ok().json(OtpSessionId))
            .map_err(|err| HttpResponse::InternalServerError().body(err.to_string()))?),
        Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
    }
}

//Handler for POST /register/verify
async fn verification(
    db: web::Data<Pool>,
    item: web::Json<InputUser>,
    data: web::Json<OtpSessionId>,
    otp: web::Json<Otp>,
) -> Result<HttpResponse, Error> {
    let result = web::block(move || verify_otp(&data.Details, &otp.otp)).await;
    match result {
        Ok(_) => Ok(web::block(move || server_verify_update(db, &item.username))
            .await
            .map(|_| HttpResponse::Ok().body("Account Successfully verified"))
            .map_err(|err| HttpResponse::InternalServerError().body(err.to_string()))?),
        Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
    }
}

//Handler for /logout
pub async fn logout(session: Session, req: HttpRequest) -> HttpResponse {
    session.clear();
    match is_json_request(&req) {
        true => HttpResponse::NoContent().finish(),
        false => HttpResponse::MovedPermanently()
            .header(LOCATION, "/user/login")
            .finish(),
    }
}

//Handler for login
#[allow(unreachable_patterns)]
#[allow(non_snake_case)]
fn handle_sign_in(
    data: AuthData,
    session: &Session,
    pool: &web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let result = find_user(data, pool);

    match result {
        Ok(Some(User)) => {
            set_current_user(session, User);

            Ok(HttpResponse::Ok().body("Successfully Logged In"))
        }
        Ok(None) => Ok(HttpResponse::Ok().body("Invalid Password or Account Not verified...")),
        Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
    }
}

//function to fetch and check data from DB
#[allow(non_snake_case)]
fn find_user(
    data: AuthData,
    pool: &web::Data<Pool>,
) -> Result<Option<User>, diesel::result::Error> {
    let mut items = user
        .filter(username.eq(&data.username))
        .load::<User>(&pool.get().unwrap())?;

    if let Some(User) = items.pop() {
        if User.is_verified {
            if bcrypt::verify(&data.password, &User.password) {
                Ok(Some(User))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    } else {
        Err(diesel::result::Error::NotFound)
    }
}

//function to insert data into DB
fn add_user(
    db: web::Data<Pool>,
    data: web::Json<InputUser>,
) -> Result<User, diesel::result::Error> {
    let conn = db.get().unwrap();
    let hasbedpassword = bcrypt::hash(&data.password).unwrap();
    let new_user = NewUser {
        name: &data.name,
        email: &data.email,
        phone: &data.phone,
        username: &data.username,
        overall_rating: &0,
        is_pragyan: &false,
        password: &hasbedpassword,
        is_verified: &false,
        highest_rating: &0,
    };
    let res = insert_into(user).values(&new_user).get_result(&conn)?;
    Ok(res)
}

//function to send OTP for verification
fn send_otp(inputphone: &str) -> Result<OtpSessionId, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let api_key = std::env::var("API_TOKEN").expect("API_TOKEN must be set");
    let template_name = std::env::var("TEMPLATE_NAME").expect("TEMPLATE_NAME must be set");
    let url = format!(
        "https://2factor.in/API/V1/{}/SMS/{}/AUTOGEN/{}",
        &api_key, &inputphone, &template_name
    );
    let data: OtpSessionId = client
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()?
        .json()?;
    Ok(data)
}

//function to verify OTP that sent to user registered mobile number
fn verify_otp(sessionid: &str, otpdata: &str) -> Result<OtpSessionId, reqwest::Error> {
    let client = reqwest::blocking::Client::new();
    let api_key = std::env::var("API_TOKEN").expect("API_TOKEN must be set");
    let url = format!(
        "http://2factor.in/API/V1/{}/SMS/VERIFY/{}/{}",
        &api_key, &sessionid, &otpdata
    );
    let data: OtpSessionId = client
        .get(&url)
        .header("content-type", "application/x-www-form-urlencoded")
        .send()?
        .json()?;
    Ok(data)
}

//function to update the verification into DB
fn server_verify_update(
    db: web::Data<Pool>,
    inputusername: &str,
) -> Result<(), diesel::result::Error> {
    let conn = db.get().unwrap();
    let _res = update(user.filter(username.eq(&inputusername)))
        .set(is_verified.eq(&true))
        .execute(&conn);
    Ok(())
}
