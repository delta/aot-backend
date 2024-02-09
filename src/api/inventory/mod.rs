use self::util::{upgrade_attacker, upgrade_building, upgrade_defender, upgrade_emp, upgrade_mine};
use super::{auth::session::AuthUser, error, PgPool};
use actix_web::{
    error::ErrorBadRequest,
    web::{self, Json},
    HttpResponse, Responder, Result,
};
use serde::{Deserialize, Serialize};
pub mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/get").route(web::get().to(get_inventory)))
        .service(web::resource("/upgrade").route(web::post().to(upgrade)));
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

#[derive(Deserialize, Serialize)]
struct UpgradeStruct {
    pub item_type: String,
    pub item_id: i32,
}

async fn upgrade(
    pool: web::Data<PgPool>,
    user: AuthUser,
    req: Json<UpgradeStruct>,
) -> Result<impl Responder> {
    let user_id = user.0;
    let mut conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let item_type = &req.item_type;
    let item_id = req.item_id;

    match item_type.as_str() {
        "attacker" => upgrade_attacker(user_id, &mut conn, item_id),
        "building" => upgrade_building(user_id, &mut conn, item_id),
        "defender" => upgrade_defender(user_id, &mut conn, item_id),
        "emp" => upgrade_emp(user_id, &mut conn, item_id),
        "mine" => upgrade_mine(user_id, &mut conn, item_id),
        _ => return Err(ErrorBadRequest("Invalid item type")),
    }
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(HttpResponse::Ok().finish())
}
