use super::{auth::session, error};
use actix_session::Session;
use actix_web::{error::ErrorBadRequest, web, Responder, Result};
use diesel::{
    r2d2::{ConnectionManager, Pool},
    PgConnection,
};
use util::LeaderboardQuery;

mod util;

type DbPool = Pool<ConnectionManager<PgConnection>>;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/leaderboard").route(web::get().to(list_leaderboard)))
        .service(web::resource("/{game_id}/replay").route(web::get().to(get_replay)));
}

async fn list_leaderboard(
    session: Session,
    query: web::Query<LeaderboardQuery>,
    pool: web::Data<DbPool>,
) -> Result<impl Responder> {
    let user_id = session::get_current_user(&session)?;

    let page = query.page.unwrap_or(1);
    let limit = query.limit.unwrap_or(20);
    if page <= 0 || limit <= 0 {
        return Err(ErrorBadRequest("Invalid query params"));
    }
    let response = web::block(move || {
        let conn = pool.get()?;
        util::get_leaderboard(page, limit, user_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}

async fn get_replay(
    game_id: web::Path<i32>,
    pool: web::Data<DbPool>,
    session: Session,
) -> Result<impl Responder> {
    let user_id = session::get_current_user(&session)?;
    let game_id = game_id.0;

    let conn = pool.get().map_err(|err| error::handle_error(err.into()))?;
    let is_replay_allowed =
        web::block(move || util::fetch_is_replay_allowed(game_id, user_id, &conn))
            .await
            .map_err(|err| error::handle_error(err.into()))?;

    if !is_replay_allowed {
        return Err(ErrorBadRequest("Requested replay is not available"));
    }

    let response = web::block(move || {
        let conn = pool.get()?;
        util::fetch_replay(game_id, &conn)
    })
    .await
    .map_err(|err| error::handle_error(err.into()))?;
    Ok(web::Json(response))
}
