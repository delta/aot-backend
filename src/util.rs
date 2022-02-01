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
