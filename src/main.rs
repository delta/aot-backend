use crate::api::{attack, auth, defense, game, user};
use crate::constants::{ATTACK_END_TIME, ATTACK_START_TIME};
use actix_cors::Cors;
use actix_redis::RedisSession;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use chrono::NaiveTime;
use diesel_migrations::embed_migrations;
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Naming};

mod api;
mod constants;
mod error;
mod models;
mod schema;
mod simulation;
mod util;

#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

embed_migrations!();

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    flexi_logger::Logger::try_with_str("info, actix_web=info")
        .unwrap()
        .log_to_file(FileSpec::default().directory("./logs"))
        .append()
        .duplicate_to_stderr(Duplicate::All)
        .rotate(
            Criterion::Size(50 * 1024 * 1024),
            Naming::Timestamps,
            Cleanup::Never,
        )
        .start()
        .unwrap();

    assert!(NaiveTime::parse_from_str(ATTACK_START_TIME, "%H:%M:%S").is_ok());
    assert!(NaiveTime::parse_from_str(ATTACK_END_TIME, "%H:%M:%S").is_ok());

    let pg_pool = util::get_pg_conn_pool();
    let redis_pool = util::get_redis_conn_pool();
    let cookie_key = std::env::var("COOKIE_KEY").expect("COOKIE_KEY must be set");
    let frontend_url = std::env::var("FRONTEND_URL").expect("FRONTEND_URL must be set");
    let redis_url = std::env::var("REDIS_URL").unwrap();

    let conn = pg_pool.get().expect("Could not get connection from pool");
    embedded_migrations::run_with_output(&conn, &mut std::io::stdout()).expect("Migrations failed");

    HttpServer::new(move || {
        App::new()
            .wrap(
                RedisSession::new(&redis_url, cookie_key.as_ref())
                    .cookie_name("session")
                    .ttl(7 * 24 * 60 * 60),
            )
            .wrap(
                Cors::default()
                    .allowed_origin(&frontend_url)
                    .allow_any_header()
                    .allow_any_method()
                    .expose_any_header()
                    .supports_credentials()
                    .max_age(3600),
            )
            .wrap(middleware::Logger::new(
                "%t %{r}a %r %s %b %{Referer}i %{User-Agent}i %T",
            ))
            .data(pg_pool.clone())
            .data(redis_pool.clone())
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello from AoR")),
            )
            .service(web::scope("/attack").configure(attack::routes))
            .service(
                web::scope("/user")
                    .configure(user::routes)
                    .configure(auth::routes),
            )
            .service(web::scope("/base").configure(defense::routes))
            .service(web::scope("/game").configure(game::routes))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
