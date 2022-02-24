use crate::api::{attack, auth, defense, game, user};
use crate::constants::{ATTACK_END_TIME, ATTACK_START_TIME};
use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use chrono::NaiveTime;
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

    let pool = util::get_connection_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %t",
            ))
            .wrap(
                CookieSession::signed(
                    std::env::var("COOKIE_KEY")
                        .expect("COOKIE_KEY must be set")
                        .as_ref(),
                )
                .name("session")
                .secure(false)
                .expires_in(30 * 24 * 60 * 60),
            )
            .data(pool.clone())
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello from AOT")),
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
