use actix_web::{web, HttpResponse, Responder};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/list").route(web::get().to(list_leaderboard)));
}

async fn list_leaderboard() -> impl Responder {
    HttpResponse::Ok().body("Todo")
}
