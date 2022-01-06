use super::robots::Robot;
use crate::models::{BlockType, MapSpaces, ShortestPath};
use chrono::{Duration, NaiveTime, Timelike};
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

const EMP_TIMEOUT: i32 = 5;
const GAME_MINUTES_PER_FRAME: i32 = 2;

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

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct SourceDest {
    source_x: i32,
    source_y: i32,
    dest_x: i32,
    dest_y: i32,
}

#[derive(Debug)]
pub struct BuildingsManager {
    buildings: HashMap<i32, Building>,
    building_types: HashMap<i32, BuildingType>,
    shortest_paths: HashMap<SourceDest, Vec<(i32, i32)>>,
    pub impacted_buildings: HashMap<i32, HashSet<i32>>,
    pub is_impacted: HashSet<i32>,
    pub buildings_grid: [[i32; 40]; 40],
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
            .map(|(t, w)| (*t, *w))
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
    ) -> HashMap<SourceDest, Vec<(i32, i32)>> {
        use crate::schema::shortest_path::dsl::*;
        let results = shortest_path
            .filter(base_id.eq(map_id))
            .load::<ShortestPath>(conn)
            .expect("Couldn't get ShortestPaths");
        let mut shortest_paths: HashMap<SourceDest, Vec<(i32, i32)>> = HashMap::new();
        for path in results {
            let path_list: Vec<(i32, i32)> = path.pathlist[1..path.pathlist.len() - 1]
                .split("),(")
                .map(|s| {
                    let path_coordinate: Vec<i32> = s
                        .split(',')
                        .map(|x| x.parse().expect("Invalid Path Coordinate"))
                        .collect();
                    (path_coordinate[0], path_coordinate[1])
                })
                .collect();
            shortest_paths.insert(
                SourceDest {
                    source_x: path.source_x,
                    source_y: path.source_y,
                    dest_x: path.dest_x,
                    dest_y: path.dest_y,
                },
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

    // Returns a matrix with each element containing the map_space id of the building in that location
    fn get_building_grid(conn: &PgConnection, map_id: i32) -> [[i32; 40]; 40] {
        use crate::schema::block_type::dsl::{block_type as block_type_table, id};
        let map_spaces: Vec<MapSpaces> = BuildingsManager::get_building_map_spaces(conn, map_id);
        let mut building_grid: [[i32; 40]; 40] = [[0; 40]; 40];

        for map_space in map_spaces {
            let BlockType { width, height, .. } = block_type_table
                .filter(id.eq(map_space.blk_type))
                .first::<BlockType>(conn)
                .expect("Couldn't get block type");
            let MapSpaces {
                x_coordinate,
                y_coordinate,
                rotation,
                ..
            } = map_space;

            match rotation {
                0 | 180 => {
                    for i in x_coordinate..x_coordinate + width {
                        for j in y_coordinate..y_coordinate + height {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                90 | 270 => {
                    for i in x_coordinate..x_coordinate + height {
                        for j in y_coordinate..y_coordinate + width {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                _ => panic!("Invalid Map Space Rotation"),
            };
        }

        building_grid
    }

    // get new instance with map_id
    pub fn new(conn: &PgConnection, map_id: i32) -> Self {
        let map_spaces = BuildingsManager::get_building_map_spaces(conn, map_id);
        let building_types = BuildingsManager::get_building_types(conn);
        let mut buildings: HashMap<i32, Building> = HashMap::new();
        let impacted_buildings: HashMap<i32, HashSet<i32>> = HashMap::new();
        let is_impacted: HashSet<i32> = HashSet::new();
        let buildings_grid: [[i32; 40]; 40] = BuildingsManager::get_building_grid(conn, map_id);

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

        let shortest_paths = BuildingsManager::get_shortest_paths(conn, map_id);
        BuildingsManager {
            buildings,
            building_types,
            shortest_paths,
            impacted_buildings,
            is_impacted,
            buildings_grid,
        }
    }
}

// Methods
#[allow(dead_code)]
impl BuildingsManager {
    pub fn damage_building(&mut self, time: i32, building_id: i32) {
        let BuildingsManager {
            impacted_buildings,
            is_impacted,
            ..
        } = self;

        impacted_buildings
            .entry(time)
            .or_insert_with(HashSet::<i32>::new);
        impacted_buildings
            .get_mut(&time)
            .unwrap()
            .insert(building_id);
        is_impacted.insert(building_id);
    }

    pub fn revive_buildings(&mut self, time: i32) {
        let time = time - EMP_TIMEOUT;
        let BuildingsManager {
            impacted_buildings,
            is_impacted,
            ..
        } = self;

        if impacted_buildings.contains_key(&time) {
            for building in impacted_buildings.get(&time).unwrap() {
                is_impacted.remove(building);
            }
        }
        impacted_buildings.remove(&time);
    }

    fn get_adjusted_weight(distance: &usize, weight: &i32) -> f32 {
        *weight as f32 / *distance as f32
    }

    fn choose_weighted(choices: &[i32], weights: &[f32]) -> i32 {
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        choices[dist.sample(&mut rng)]
    }

    fn get_hour(time: i32) -> i32 {
        NaiveTime::from_hms(9, 0, 0)
            .overflowing_add_signed(Duration::minutes((GAME_MINUTES_PER_FRAME * time) as i64))
            .0
            .hour() as i32
    }

    // get id of building using weighted random given starting co-ordinate
    fn get_weighted_random_building(&self, x: i32, y: i32, time: i32) -> i32 {
        let mut choices = vec![];
        let mut weights = vec![];

        for building in self.buildings.values() {
            if building.absolute_entrance_x == x && building.absolute_entrance_y == y {
                continue;
            }
            let shortest_path_length = match self.shortest_paths.get(&SourceDest {
                source_x: x,
                source_y: y,
                dest_x: building.absolute_entrance_x,
                dest_y: building.absolute_entrance_y,
            }) {
                Some(v) => v.len(),
                None => panic!("shortest path not found"),
            };
            let weight = self
                .building_types
                .get(&building.map_space.blk_type)
                .expect("Couldn't get block type")
                .weights
                .get(&BuildingsManager::get_hour(time))
                .expect("Couldn't get weight at time");
            let adjusted_weight =
                BuildingsManager::get_adjusted_weight(&shortest_path_length, weight);
            choices.push(building.map_space.id);
            weights.push(adjusted_weight);
        }
        BuildingsManager::choose_weighted(&choices, &weights)
    }

    pub fn assign_building(&self, robot: &mut Robot, time: i32) {
        let destination_id =
            self.get_weighted_random_building(robot.x_position, robot.y_position, time);
        robot.destination = destination_id;
    }

    pub fn assign_initial_buildings(&self, robots: &mut HashMap<i32, Robot>, time: i32) {
        let mut weights = vec![];
        let mut choices = vec![];
        for building in self.buildings.values() {
            let weight = self
                .building_types
                .get(&building.map_space.blk_type)
                .expect("Couldn't get block type")
                .weights
                .get(&BuildingsManager::get_hour(time))
                .expect("Couldn't get weight at time");
            weights.push(weight);
            choices.push(building.map_space.id);
        }
        let dist = WeightedIndex::new(weights).unwrap();
        let mut rng = thread_rng();
        for robot in robots.values_mut() {
            robot.destination = choices[dist.sample(&mut rng)];
        }
    }
}
