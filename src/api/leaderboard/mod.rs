use super::{error, PgPool};
use actix_web::{error::ErrorBadRequest, web, Responder, Result};
use util::LeaderboardQuery;

pub mod util;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("").route(web::get().to(list_leaderboard)));
}

async fn list_leaderboard(
    //Removed
    query: web::Query<LeaderboardQuery>,
    pool: web::Data<PgPool>,
) -> Result<impl Responder> {
    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    if page <= 0 || limit <= 0 {
        return Err(ErrorBadRequest("Invalid query params"));
    }
    let response = web::block(move || {
        let mut conn = pool.get()?;
        util::get_leaderboard(page, limit, &mut conn)
    })
    .await?
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}
