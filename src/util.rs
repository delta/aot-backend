use diesel::r2d2::{self, ConnectionManager};
use diesel::PgConnection;

pub fn get_connection_pool() -> r2d2::Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().ok();

    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}
