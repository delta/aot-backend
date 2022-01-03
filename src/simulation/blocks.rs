use crate::models::{BlockType, MapSpaces, ShortestPath};
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use std::collections::HashMap;

#[derive(Debug)]
struct BuildingType {
    block_type: BlockType,
    weights: HashMap<i32, i32>,
}

#[derive(Debug)]
struct Building {
    map_space: MapSpaces,
    absolute_entrance_x: i32,
    absolute_entrance_y: i32,
}

pub struct BuildingsManager {
    buildings: HashMap<i32, Building>,
    building_types: HashMap<i32, BuildingType>,
    shortest_paths: HashMap<(i32, i32, i32, i32), Vec<(i32, i32)>>,
}

// Associated functions
impl BuildingsManager {
    // Get all map_spaces for this map excluding roads
    fn get_building_map_spaces(conn: &PgConnection, map_id: i32) -> Vec<MapSpaces> {
        use crate::schema::{block_type, map_spaces};

        let road_id: i32 = block_type::table
            .filter(block_type::name.eq("road"))
            .select(block_type::id)
            .first(conn)
            .expect("Couldn't get road id");

        map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .filter(map_spaces::blk_type.ne(road_id))
            .load::<MapSpaces>(conn)
            .expect("Couldn't get Map Spaces")
    }

    // get time: weight HashMap given block_type id
    fn get_weights(conn: &PgConnection, b_id: i32) -> HashMap<i32, i32> {
        use crate::schema::building_weights::dsl::*;
        building_weights
            .filter(building_id.eq(b_id))
            .select((time, weight))
            .load::<(i32, i32)>(conn)
            .expect("Couldn't get weights of building")
            .iter()
            .map(|(t, w)| (t.clone(), w.clone()))
            .collect()
    }

    // get all building_types with their weights
    fn get_building_types(conn: &PgConnection) -> HashMap<i32, BuildingType> {
        use crate::schema::block_type::dsl::*;
        block_type
            .load::<BlockType>(conn)
            .expect("Couldn't load building types")
            .iter()
            .map(|x| {
                (
                    x.id,
                    BuildingType {
                        block_type: x.clone(),
                        weights: BuildingsManager::get_weights(conn, x.id),
                    },
                )
            })
            .collect()
    }

    // get all shortest paths with string pathlist converted to vector of i32 tuples
    pub fn get_shortest_paths(
        conn: &PgConnection,
        map_id: i32,
    ) -> HashMap<(i32, i32, i32, i32), Vec<(i32, i32)>> {
        use crate::schema::shortest_path::dsl::*;
        let results = shortest_path
            .filter(base_id.eq(map_id))
            .load::<ShortestPath>(conn)
            .expect("Couldn't get ShortestPaths");
        let mut shortest_paths: HashMap<(i32, i32, i32, i32), Vec<(i32, i32)>> = HashMap::new();
        for path in results {
            let path_list: Vec<(i32, i32)> = path.pathlist[1..path.pathlist.len() - 1]
                .split("),(")
                .map(|s| {
                    let path_coordinate: Vec<i32> = s
                        .split(",")
                        .map(|x| x.parse().expect("Invalid Path Coordinate"))
                        .collect();
                    (path_coordinate[0], path_coordinate[1])
                })
                .collect();
            shortest_paths.insert(
                (path.source_x, path.source_y, path.dest_x, path.dest_y),
                path_list,
            );
        }
        shortest_paths
    }

    // get absolute entrance location (x, y) in map with map_space and block_type
    pub fn get_absolute_entrance(map_space: &MapSpaces, block_type: &BlockType) -> (i32, i32) {
        match map_space.rotation {
            0 => (
                map_space.x_coordinate + block_type.entrance_x,
                map_space.y_coordinate + block_type.entrance_y,
            ),
            90 => (
                map_space.x_coordinate - block_type.entrance_y,
                map_space.y_coordinate + block_type.entrance_x,
            ),
            180 => (
                map_space.x_coordinate - block_type.entrance_x,
                map_space.y_coordinate - block_type.entrance_y,
            ),
            270 => (
                map_space.x_coordinate + block_type.entrance_y,
                map_space.y_coordinate - block_type.entrance_x,
            ),
            _ => panic!("Invalid Map Space Rotation"),
        }
    }

    // get new instance with map_id
    pub fn new(conn: &PgConnection, map_id: i32) -> Self {
        let map_spaces = BuildingsManager::get_building_map_spaces(conn, map_id);
        let building_types = BuildingsManager::get_building_types(conn);
        let mut buildings: HashMap<i32, Building> = HashMap::new();
        for map_space in map_spaces {
            let (absolute_entrance_x, absolute_entrance_y) =
                BuildingsManager::get_absolute_entrance(
                    &map_space,
                    &building_types[&map_space.blk_type].block_type,
                );
            buildings.insert(
                map_space.id,
                Building {
                    map_space,
                    absolute_entrance_x,
                    absolute_entrance_y,
                },
            );
        }
        for b in &buildings {
            println!("{:?}", b);
        }
        let shortest_paths = BuildingsManager::get_shortest_paths(conn, map_id);
        BuildingsManager {
            buildings,
            building_types,
            shortest_paths,
        }
    }
}

// Methods
impl BuildingsManager {}
