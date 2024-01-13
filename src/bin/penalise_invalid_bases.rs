use aot_backend::api;
use aot_backend::constants::SCALE_FACTOR;
use aot_backend::schema::{map_layout, user};
use aot_backend::util;
use diesel::QueryDsl;
use diesel::{prelude::*, update};

fn main() {
    let pool = util::get_pg_conn_pool();
    let mut conn = pool.get().expect("Could not retrieve connection from pool");

    let level_id = api::util::get_current_levels_fixture(&mut conn)
        .expect("Could not get level id")
        .id;
    let invalid_users = user::table
        .left_join(
            map_layout::table.on(map_layout::player
                .eq(user::id)
                .and(map_layout::level_id.eq(level_id))
                .and(map_layout::is_valid.eq(true))),
        )
        .select(user::id)
        .filter(map_layout::is_valid.is_null())
        .load::<i32>(&mut conn)
        .expect("Could not get invalid users");

    update(user::table)
        .filter(user::id.eq_any(invalid_users))
        .set(user::trophies.eq(user::trophies - ((4.0 * SCALE_FACTOR) as i32)))
        .execute(&mut conn)
        .expect("Could not update user ratings");

    println!("Ratings have been updated");
}
