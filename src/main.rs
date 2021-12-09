use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Naming};

use crate::api::{attack, auth, defense, stats};

mod api;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().ok();

    flexi_logger::Logger::try_with_str("actix_web=info")
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

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::new(
                "%{r}a %r %s %b %{Referer}i %{User-Agent}i %t",
            ))
            .data(pool.clone())
            .route(
                "/",
                web::get().to(|| HttpResponse::Ok().body("Hello from AOT")),
            )
            .route("/user/stats", web::get().to(stats::get_user_stats))
            .service(web::scope("/attack").configure(attack::routes))
            .service(web::scope("/user").configure(auth::routes))
            .service(web::scope("/base").configure(defense::routes))
    })
    .bind("0.0.0.0:8000")?
    .run()
    .await
}
