use blocks::BuildingsManager;
use diesel::prelude::*;
use robots::RobotsManager;

#[allow(dead_code)]
pub mod attacker;
pub mod blocks;
pub mod emp;
pub mod robots;

const GAME_TIME: i32 = 120;

#[allow(dead_code)]
fn simulate(game_id: i32) {
    use crate::schema::game::dsl::*;
    let pool = crate::get_connection_pool();
    let conn = &*pool.get().unwrap();
    let map_id = game
        .filter(id.eq(game_id))
        .select(map_layout_id)
        .first::<i32>(conn)
        .unwrap_or_else(|_| panic!("Could not get map_id for game {}", game_id));

    let mut buildings_manager = BuildingsManager::new(conn, map_id);
    let mut robots_manager = RobotsManager::new();
    let emps = emp::get_emps(conn, game_id);

    for time in 1..GAME_TIME {
        emp::blast_emp(time, &emps, &mut robots_manager, &mut buildings_manager);
        buildings_manager.revive_buildings(time);
        todo!("Implement robot functions, return results")
    }
}
