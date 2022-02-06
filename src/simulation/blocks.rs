use super::robots::Robot;
use crate::constants::*;
use crate::error::DieselError;
use crate::models::{BlockType, MapSpaces, ShortestPath};
use crate::simulation::error::*;
use crate::util::function;
use anyhow::{Context, Result};
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use rand::distributions::WeightedIndex;
use rand::prelude::*;
use std::collections::{HashMap, HashSet};

#[derive(Debug)]
struct BuildingType {
    block_type: BlockType,
    weights: HashMap<i32, i32>,
}

#[derive(Debug)]
pub struct Building {
    map_space: MapSpaces,
    pub absolute_entrance_x: i32,
    pub absolute_entrance_y: i32,
    pub weight: i32,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct SourceDest {
    pub source_x: i32,
    pub source_y: i32,
    pub dest_x: i32,
    pub dest_y: i32,
}

#[derive(Debug)]
pub struct BuildingsManager {
    pub buildings: HashMap<i32, Building>,
    building_types: HashMap<i32, BuildingType>,
    pub shortest_paths: HashMap<SourceDest, Vec<(i32, i32)>>,
    impacted_buildings: HashMap<i32, HashSet<i32>>,
    is_impacted: HashSet<i32>,
    pub buildings_grid: [[i32; MAP_SIZE]; MAP_SIZE],
}

// Associated functions
impl BuildingsManager {
    // Get all map_spaces for this map excluding roads
    fn get_building_map_spaces(conn: &PgConnection, map_id: i32) -> Result<Vec<MapSpaces>> {
        use crate::schema::{block_type, map_spaces};

        let road_id: i32 = block_type::table
            .filter(block_type::name.eq("road"))
            .select(block_type::id)
            .first(conn)
            .map_err(|err| DieselError {
                table: "block_type",
                function: function!(),
                error: err,
            })?;

        Ok(map_spaces::table
            .filter(map_spaces::map_id.eq(map_id))
            .filter(map_spaces::blk_type.ne(road_id))
            .load::<MapSpaces>(conn)
            .map_err(|err| DieselError {
                table: "map_spaces",
                function: function!(),
                error: err,
            })?)
    }

    // get time: weight HashMap given block_type id
    fn get_weights(conn: &PgConnection, b_id: i32) -> Result<HashMap<i32, i32>> {
        use crate::schema::building_weights::dsl::*;
        Ok(building_weights
            .filter(building_id.eq(b_id))
            .select((time, weight))
            .load::<(i32, i32)>(conn)
            .map_err(|err| DieselError {
                table: "building_weights",
                function: function!(),
                error: err,
            })?
            .iter()
            .map(|(t, w)| (*t, *w))
            .collect())
    }

    // get all building_types with their weights
    fn get_building_types(conn: &PgConnection) -> Result<HashMap<i32, BuildingType>> {
        use crate::schema::block_type::dsl::*;
        block_type
            .load::<BlockType>(conn)
            .map_err(|err| DieselError {
                table: "block_type",
                function: function!(),
                error: err,
            })?
            .iter()
            .map(|x| match Self::get_weights(conn, x.id) {
                Ok(weights) => Ok((
                    x.id,
                    BuildingType {
                        block_type: x.clone(),
                        weights,
                    },
                )),
                Err(err) => Err(err),
            })
            .collect()
    }

    // get all shortest paths with string pathlist converted to vector of i32 tuples
    fn get_shortest_paths(
        conn: &PgConnection,
        map_id: i32,
    ) -> Result<HashMap<SourceDest, Vec<(i32, i32)>>> {
        use crate::schema::shortest_path::dsl::*;
        let results = shortest_path
            .filter(base_id.eq(map_id))
            .load::<ShortestPath>(conn)
            .map_err(|err| DieselError {
                table: "shortest_path",
                function: function!(),
                error: err,
            })?;
        let mut shortest_paths: HashMap<SourceDest, Vec<(i32, i32)>> = HashMap::new();
        for path in results {
            let path_list: Vec<(i32, i32)> = path.pathlist[1..path.pathlist.len() - 1]
                .split(")(")
                .map(|s| {
                    let path_coordinate: Vec<i32> =
                        s.split(',').map(|x| x.parse().unwrap()).collect();
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
        Ok(shortest_paths)
    }

    // get absolute entrance location (x, y) in map with map_space and block_type
    fn get_absolute_entrance(map_space: &MapSpaces, block_type: &BlockType) -> Result<(i32, i32)> {
        match map_space.rotation {
            0 => Ok((
                map_space.x_coordinate + block_type.entrance_x,
                map_space.y_coordinate + block_type.entrance_y,
            )),
            90 => Ok((
                map_space.x_coordinate - block_type.entrance_y,
                map_space.y_coordinate + block_type.entrance_x,
            )),
            180 => Ok((
                map_space.x_coordinate - block_type.entrance_x,
                map_space.y_coordinate - block_type.entrance_y,
            )),
            270 => Ok((
                map_space.x_coordinate + block_type.entrance_y,
                map_space.y_coordinate - block_type.entrance_x,
            )),
            _ => Err(MapSpaceRotationError {
                map_space_id: map_space.id,
            }
            .into()),
        }
    }

    // Returns a matrix with each element containing the map_space id of the building in that location
    fn get_building_grid(conn: &PgConnection, map_id: i32) -> Result<[[i32; MAP_SIZE]; MAP_SIZE]> {
        use crate::schema::block_type;

        let map_spaces: Vec<MapSpaces> = Self::get_building_map_spaces(conn, map_id)?;
        let mut building_grid: [[i32; MAP_SIZE]; MAP_SIZE] = [[0; MAP_SIZE]; MAP_SIZE];

        for map_space in map_spaces {
            let BlockType { width, height, .. } = block_type::table
                .filter(block_type::id.eq(map_space.blk_type))
                .first::<BlockType>(conn)
                .map_err(|err| DieselError {
                    table: "block_type",
                    function: function!(),
                    error: err,
                })?;
            let MapSpaces {
                x_coordinate,
                y_coordinate,
                rotation,
                ..
            } = map_space;

            match rotation {
                0 => {
                    for i in x_coordinate..x_coordinate + width {
                        for j in y_coordinate..y_coordinate + height {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                90 => {
                    for i in x_coordinate..x_coordinate - height {
                        for j in y_coordinate..y_coordinate + width {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                180 => {
                    for i in x_coordinate..x_coordinate - width {
                        for j in y_coordinate..y_coordinate - height {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                270 => {
                    for i in x_coordinate..x_coordinate + height {
                        for j in y_coordinate..y_coordinate - width {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                _ => {
                    return Err(MapSpaceRotationError {
                        map_space_id: map_space.id,
                    }
                    .into())
                }
            };
        }

        Ok(building_grid)
    }

    // get new instance with map_id
    pub fn new(conn: &PgConnection, map_id: i32) -> Result<Self> {
        let map_spaces = Self::get_building_map_spaces(conn, map_id)?;
        let building_types = Self::get_building_types(conn)?;
        let mut buildings: HashMap<i32, Building> = HashMap::new();
        let impacted_buildings: HashMap<i32, HashSet<i32>> = HashMap::new();
        let is_impacted: HashSet<i32> = HashSet::new();
        let buildings_grid: [[i32; MAP_SIZE]; MAP_SIZE] = Self::get_building_grid(conn, map_id)?;

        for map_space in map_spaces {
            let (absolute_entrance_x, absolute_entrance_y) = Self::get_absolute_entrance(
                &map_space,
                &building_types[&map_space.blk_type].block_type,
            )?;
            let weight = *building_types
                .get(&map_space.blk_type)
                .ok_or(KeyError {
                    key: map_space.blk_type,
                    hashmap: "building_types".to_string(),
                })?
                .weights
                .get(&9)
                .ok_or(KeyError {
                    key: 9,
                    hashmap: format!("building_types[{}].weights", map_space.blk_type),
                })?;
            buildings.insert(
                map_space.id,
                Building {
                    map_space,
                    absolute_entrance_x,
                    absolute_entrance_y,
                    weight,
                },
            );
        }

        let shortest_paths = Self::get_shortest_paths(conn, map_id)?;
        Ok(BuildingsManager {
            buildings,
            building_types,
            shortest_paths,
            impacted_buildings,
            is_impacted,
            buildings_grid,
        })
    }

    fn get_adjusted_weight(distance: &usize, weight: &i32) -> f32 {
        let adjusted_weight = *weight as f32 / *distance as f32;
        adjusted_weight.max(0.0)
    }

    fn choose_weighted(choices: &[i32], weights: &[f32]) -> Result<i32> {
        let dist =
            WeightedIndex::new(weights).with_context(|| format!("Weights: {:?}", weights))?;
        let mut rng = thread_rng();
        Ok(choices[dist.sample(&mut rng)])
    }
}

// Methods
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

    // get id of building using weighted random given starting co-ordinate
    pub fn get_weighted_random_building(&self, x: i32, y: i32) -> Result<i32> {
        let mut choices = vec![];
        let mut weights = vec![];

        for building in self.buildings.values() {
            let Building {
                map_space,
                absolute_entrance_x,
                absolute_entrance_y,
                weight,
            } = building;
            if *absolute_entrance_x == x && *absolute_entrance_y == y {
                continue;
            }
            if self.is_impacted.contains(&map_space.id) {
                continue;
            }
            let source_dest = SourceDest {
                source_x: x,
                source_y: y,
                dest_x: *absolute_entrance_x,
                dest_y: *absolute_entrance_y,
            };
            let shortest_path_length = match self.shortest_paths.get(&source_dest) {
                Some(v) => v.len(),
                None => return Err(ShortestPathNotFoundError(source_dest).into()),
            };
            let adjusted_weight = Self::get_adjusted_weight(&shortest_path_length, weight);
            choices.push(map_space.id);
            weights.push(adjusted_weight);
        }
        Self::choose_weighted(&choices, &weights)
    }

    pub fn assign_initial_buildings(&self, robots: &mut HashMap<i32, Robot>) -> Result<()> {
        let mut weights = vec![];
        let mut choices = vec![];
        for building in self.buildings.values() {
            weights.push(building.weight);
            choices.push(building.map_space.id);
        }
        let dist =
            WeightedIndex::new(&weights).with_context(|| format!("Weights: {:?}", weights))?;
        let mut rng = thread_rng();
        for robot in robots.values_mut() {
            robot.destination = choices[dist.sample(&mut rng)];
            let initial_building_id = choices[dist.sample(&mut rng)];
            let Building {
                absolute_entrance_x,
                absolute_entrance_y,
                ..
            } = self.buildings.get(&initial_building_id).unwrap();
            robot.x_position = *absolute_entrance_x;
            robot.y_position = *absolute_entrance_y;
        }
        Ok(())
    }

    pub fn update_building_weights(&mut self, hour: i32) -> Result<()> {
        for building in self.buildings.values_mut() {
            let weights = &self
                .building_types
                .get(&building.map_space.blk_type)
                .ok_or(KeyError {
                    key: building.map_space.blk_type,
                    hashmap: "building_types".to_string(),
                })?
                .weights;
            let weight = weights.get(&(hour - 1)).ok_or(KeyError {
                key: hour - 1,
                hashmap: format!("building_types[{}].weights", building.map_space.blk_type),
            })?;
            let change = weight - building.weight;
            building.weight = *weights.get(&hour).ok_or(KeyError {
                key: hour,
                hashmap: format!("building_types[{}].weights", building.map_space.blk_type),
            })?;
            building.weight += change;
        }
        Ok(())
    }
}
