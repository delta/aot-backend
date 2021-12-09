use actix_web::{web, HttpResponse, Responder};

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(get_base_details)));
}

async fn get_base_details() -> impl Responder {
    HttpResponse::Ok().body("Todo")
}
