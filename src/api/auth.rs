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
    cfg.service(web::resource("/verify").route(web::post().to(verification)));
    cfg.service(web::resource("/logout").route(web::get().to(logout)));
    cfg.service(web::resource("/resetverify").route(web::post().to(resetverify)));
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

#[derive(Debug, Deserialize)]
pub struct AuthData {
    pub username: String,
    pub password: String,
}

//Handler for POST /login
async fn login(
    data: web::Json<AuthData>,
    session: Session,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    match is_signed_in(&session) {
        true => {
            let response = get_current_user(&session)
                .map(|userdata| HttpResponse::Ok().json(userdata))
                .unwrap();

            Ok(response)
        }
        false => handle_sign_in(data.into_inner(), &session, &pool),
    }
}

//Handler for POST /register
async fn register(db: web::Data<Pool>, item: web::Json<InputUser>) -> Result<HttpResponse, Error> {
    let check = check_user(&db, &item);

    match check {
        Ok(Some(string)) => Ok(HttpResponse::Ok().body(string)),
        Ok(None) => {
            let result = add_user(db, item);

            match result {
                Ok(userdata) => Ok(web::block(move || send_otp(&userdata.phone))
                    .await
                    .map(|otp| HttpResponse::Ok().json(otp))
                    .map_err(|_| {
                        HttpResponse::InternalServerError().body("Internal Server Error")
                    })?),
                Err(err) => Ok(HttpResponse::InternalServerError().body(err.to_string())),
            }
        }
        Err(_) => Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
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
            .map_err(|_| HttpResponse::InternalServerError().body("Internal Server Error"))?),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
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

//Handler for reset verification
async fn resetverify(
    data: web::Json<AuthData>,
    pool: web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let result = find_user(data.into_inner(), &pool);

    match result {
        Ok(Some(userdata)) => Ok(web::block(move || send_otp(&userdata.phone))
            .await
            .map(|otp| HttpResponse::Ok().json(otp))
            .map_err(|_| HttpResponse::InternalServerError().body("Internal Server Error"))?),
        Ok(None) => Ok(HttpResponse::MovedPermanently()
            .header(LOCATION, "/verify")
            .finish()),
        Err(_) => Ok(HttpResponse::InternalServerError().body("Internal Server Error")),
    }
}

//Handler for login
fn handle_sign_in(
    data: AuthData,
    session: &Session,
    pool: &web::Data<Pool>,
) -> Result<HttpResponse, Error> {
    let result = find_user(data, pool);

    match result {
        Ok(Some(userdata)) => {
            set_current_user(session, userdata);

            Ok(HttpResponse::Ok().body("Successfully Logged In"))
        }
        Ok(None) => Ok(HttpResponse::MovedPermanently()
            .header(LOCATION, "/verify")
            .finish()),
        Err(err) => {
            if err.to_string() == "NotFound" {
                Ok(HttpResponse::InternalServerError().body("Invalid Username or Password"))
            } else {
                Ok(HttpResponse::InternalServerError().body("Internal server error"))
            }
        }
    }
}

//function to fetch and check data from DB
fn find_user(
    data: AuthData,
    pool: &web::Data<Pool>,
) -> Result<Option<User>, diesel::result::Error> {
    let mut items = user
        .filter(username.eq(&data.username))
        .load::<User>(&pool.get().unwrap())?;

    if let Some(userdata) = items.pop() {
        if userdata.is_verified {
            if bcrypt::verify(&data.password, &userdata.password) {
                Ok(Some(userdata))
            } else {
                Err(diesel::result::Error::NotFound)
            }
        } else {
            Ok(None)
        }
    } else {
        Err(diesel::result::Error::NotFound)
    }
}

//function to check data exist or not
fn check_user(
    db: &web::Data<Pool>,
    data: &web::Json<InputUser>,
) -> Result<Option<String>, diesel::result::Error> {
    let mut items = user
        .filter(username.eq(&data.username))
        .or_filter(phone.eq(&data.phone))
        .or_filter(email.eq(&data.email))
        .load::<User>(&db.get().unwrap())?;

    if let Some(userdata) = items.pop() {
        if userdata.phone == data.phone {
            Ok(Some(String::from("Phone number already exist")))
        } else if userdata.email == data.email {
            Ok(Some(String::from("Email already exist")))
        } else {
            Ok(Some(String::from("Username already exist")))
        }
    } else {
        Ok(None)
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
    let api_key = std::env::var("TWOFACTOR_API_TOKEN").expect("TWOFACTOR_API_TOKEN must be set");
    let template_name =
        std::env::var("TWOFACTOR_TEMPLATE_NAME").expect("TWOFACTOR_TEMPLATE_NAME must be set");
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
    let api_key = std::env::var("TWOFACTOR_API_TOKEN").expect("TWOFACTOR_API_TOKEN must be set");
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
