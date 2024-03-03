use anyhow::{anyhow, Ok, Result};
use aot_backend::schema::{artifact, map_layout, map_spaces, user};
use aot_backend::util;
use diesel::dsl::sum;
use diesel::prelude::*;
use diesel::QueryDsl;

fn main() -> Result<()> {
    let pool = util::get_pg_conn_pool();
    let mut conn = pool.get().expect("Could not retrieve connection from pool");

    let users = user::table
        .select(user::id)
        .load::<i32>(&mut conn)
        .expect("Could not get user ids");

    let _ = Ok(users.into_iter().try_for_each(|user_id| {
        let total_artifacts_in_user_base: Option<i64> = map_spaces::table
            .inner_join(artifact::table)
            .inner_join(map_layout::table)
            .filter(map_layout::player.eq(user_id))
            .select(sum(artifact::count))
            .first(&mut conn)
            .map_err(|err| anyhow!("Error getting sum of artifacts for user: {}", err))?;

        diesel::update(user::table.filter(user::id.eq(user_id)))
            .set(user::artifacts.eq(total_artifacts_in_user_base.unwrap_or(750) as i32))
            .execute(&mut conn)
            .map(|_| ())
            .map_err(|err| anyhow!("Error updating user artifacts: {}", err))
    }))?;

    println!("Adjusted artifacts for all users");

    Ok(())
}
