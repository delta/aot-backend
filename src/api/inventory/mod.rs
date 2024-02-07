use super::{auth::session::AuthUser, error, PgPool};
use actix_web::{web, HttpResponse, Responder, Result};

pub mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get").route(web::get().to(get_inventory)))
        .service(web::resource("/upgrade").route(web::post().to(upgrade_item)));
}

async fn get_inventory(user: AuthUser, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let user_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::fetch_inventory(user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(HttpResponse::Ok().json(response))
}

async fn upgrade_item(user: AuthUser, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let user_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::upgrade_item(user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(HttpResponse::Ok().json(response))
}
