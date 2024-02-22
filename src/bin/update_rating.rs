use aot_backend::schema::user;
use aot_backend::util;
use diesel::{prelude::*, update};

fn main() {
    let pool = util::get_pg_conn_pool();
    let mut conn = pool.get().expect("Could not retrieve connection from pool");

    update(user::table)
        .set(user::trophies.eq(1000))
        .execute(&mut conn)
        .expect("Could not update user ratings");
}
