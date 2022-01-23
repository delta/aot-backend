use actix_session::CookieSession;
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Naming};

use crate::api::{attack, auth, defense, stats};

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

    let pool = util::get_connection_pool();

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %t",
            ))
            .wrap(
                CookieSession::signed(&[0; 32])
                    .domain("0.0.0.0:8000")
                    .name("auth")
                    .secure(false),
            )
            .data(pool.clone())
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello from AOT")),
            )
            .route("/user/{id}/stats", web::get().to(stats::get_user_stats))
            .service(web::scope("/attack").configure(attack::routes))
            .service(web::scope("/user").configure(auth::routes))
            .service(web::scope("/base").configure(defense::routes))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
