use actix_web::{HttpResponse, Responder};

pub async fn get_user_stats() -> impl Responder {
    HttpResponse::Ok().body("Todo")
}
