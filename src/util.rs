use diesel::r2d2::ConnectionManager;
use diesel::PgConnection;
use r2d2::Pool;

pub fn get_pg_conn_pool() -> Pool<ConnectionManager<PgConnection>> {
    dotenv::dotenv().ok();
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(db_url);
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

pub fn get_redis_conn_pool() -> Pool<redis::Client> {
    dotenv::dotenv().ok();
    let redis_url = std::env::var("REDIS_URL").expect("REDIS_URL must be set");
    let manager =
        redis::Client::open(format!("redis://{redis_url}")).expect("Failed to create redis client");
    Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let type_name = type_name_of(f);
        &type_name[..type_name.len() - 3].trim_end_matches("::{{closure}}")
    }};
}

pub(crate) use function;
