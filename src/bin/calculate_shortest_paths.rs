use aot_backend::simulation::shortestpath::run_shortest_paths;
use aot_backend::util;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use rayon::prelude::*;
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        panic!("Usage: calculate_shortest_paths [LEVEL_ID]");
    }
    let level_id: i32 = args[1].parse().expect("Enter a valid level_id");

    let pool = util::get_connection_pool();
    let conn = &*pool.get().unwrap();

    use aot_backend::schema::map_layout;

    let map_ids = map_layout::table
        .filter(map_layout::level_id.eq(level_id))
        .select(map_layout::id)
        .load::<i32>(conn)
        .expect("Couldn't get map_ids for given level");

    println!("Calculating shortest paths for level {}\n", level_id);
    map_ids.par_iter().enumerate().for_each(|(pos, map_id)| {
        println!(
            "({}/{}) Calculating shortest paths for map_id: {}..",
            pos + 1,
            map_ids.len(),
            map_id
        );
        run_shortest_paths(&*pool.get().unwrap(), *map_id);
    });
    println!(
        "\nCalculated shortest paths for level {} successfully!",
        level_id
    );
}
