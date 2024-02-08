use actix_web::{web, Responder, Result};

use super::{auth::session::AuthUser, error, PgPool};

pub mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get").route(web::get().to(get_inventory)))
        .service(web::resource("/upgrade").route(web::get().to(upgrade)));
}

async fn get_inventory(user: AuthUser, pool: web::Data<PgPool>) -> Result<impl Responder> {
    let user_id = user.0;
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::get_inventory(user_id, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;

    Ok(web::Json(response))
}

// async fn upgrade() -> Result<impl Responder> {
//     todo!()
// }
