use actix_web::{web, HttpResponse, Responder};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/login").route(web::post().to(login)));
}

async fn login() -> impl Responder {
    HttpResponse::Ok().body("Todo")
}
