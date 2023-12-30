use crate::constants::*;
use crate::error::DieselError;
use crate::models::{BlockType, MapSpaces, ShortestPath};
use crate::simulation::error::*;
use crate::simulation::BuildingStats;
use crate::util::function;
use anyhow::Result;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use std::collections::HashMap;

#[derive(Debug)]
struct BuildingClass {
    block_type: BlockType,
    // capacity: i32,
}

#[derive(Debug)]
pub struct Building {
    map_space: MapSpaces,
    pub absolute_entrance_x: i32,
    pub absolute_entrance_y: i32,
    pub population: i32,
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
    pub shortest_paths: HashMap<SourceDest, Vec<(i32, i32)>>,
    pub buildings_grid: [[i32; MAP_SIZE]; MAP_SIZE],
}

// Associated functions
impl BuildingsManager {
    // Get all map_spaces for this map excluding roads
    fn get_building_map_spaces(conn: &mut PgConnection, map_id: i32) -> Result<Vec<MapSpaces>> {
        use crate::schema::{block_type, building_type, map_spaces};

        Ok(map_spaces::table
            .inner_join(building_type::table.inner_join(block_type::table))
            .filter(map_spaces::map_id.eq(map_id))
            .filter(block_type::id.ne(ROAD_ID))
            .select(map_spaces::all_columns)
            .load::<MapSpaces>(conn)
            .map_err(|err| DieselError {
                table: "map_spaces",
                function: function!(),
                error: err,
            })?)
    }

    /* fn get_road_map_spaces(conn: &mut PgConnection, map_id: i32) -> Result<Vec<MapSpaces>> {
        use crate::schema::{block_type, building_type, map_spaces};

        Ok(map_spaces::table
            .inner_join(building_type::table.inner_join(block_type::table))
            .filter(map_spaces::map_id.eq(map_id))
            .filter(block_type::id.eq(ROAD_ID))
            .select(map_spaces::all_columns)
            .load::<MapSpaces>(conn)
            .map_err(|err| DieselError {
                table: "map_spaces",
                function: function!(),
                error: err,
            })?)
    } */

    // get all building_types
    fn get_building_types(conn: &mut PgConnection) -> Result<HashMap<i32, BuildingClass>> {
        use crate::schema::block_type::dsl::*;
        block_type
            .load::<BlockType>(conn)
            .map_err(|err| DieselError {
                table: "block_type",
                function: function!(),
                error: err,
            })?
            .iter()
            .map(|x| {
                Ok((
                    x.id,
                    BuildingClass {
                        block_type: x.clone(),
                        // capacity: x.capacity,
                    },
                ))
            })
            .collect()
    }

    // get all shortest paths with string pathlist converted to vector of i32 tuples
    fn get_shortest_paths(
        conn: &mut PgConnection,
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
    pub fn get_absolute_entrance(
        map_space: &MapSpaces,
        block_type: &BlockType,
    ) -> Result<(i32, i32)> {
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

    //Returns Hashmap of building id and block type
    fn get_building_block_map(conn: &mut PgConnection) -> Result<HashMap<i32, BlockType>> {
        use crate::schema::{block_type, building_type};

        Ok(building_type::table
            .inner_join(block_type::table)
            .select((building_type::id, block_type::all_columns))
            .load::<(i32, BlockType)>(conn)
            .map_err(|err| DieselError {
                table: "block_type",
                function: function!(),
                error: err,
            })?
            .into_iter()
            .collect())
    }

    // Returns a matrix with each element containing the map_space id of the building in that location
    fn get_building_grid(
        conn: &mut PgConnection,
        map_id: i32,
        building_block_map: &HashMap<i32, BlockType>,
    ) -> Result<[[i32; MAP_SIZE]; MAP_SIZE]> {
        let map_spaces: Vec<MapSpaces> = Self::get_building_map_spaces(conn, map_id)?;
        let mut building_grid: [[i32; MAP_SIZE]; MAP_SIZE] = [[0; MAP_SIZE]; MAP_SIZE];

        for map_space in map_spaces {
            let BlockType { width, height, .. } = building_block_map
                .get(&map_space.building_type)
                .ok_or(KeyError {
                    key: map_space.building_type,
                    hashmap: "building_block_map".to_string(),
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
                    for i in x_coordinate - height + 1..=x_coordinate {
                        for j in y_coordinate..y_coordinate + width {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                180 => {
                    for i in x_coordinate - width + 1..=x_coordinate {
                        for j in y_coordinate - height + 1..=y_coordinate {
                            building_grid[i as usize][j as usize] = map_space.id;
                        }
                    }
                }
                270 => {
                    for i in x_coordinate..x_coordinate + height {
                        for j in y_coordinate - width + 1..=y_coordinate {
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

    fn get_block_id(
        building_id: &i32,
        building_block_map: &HashMap<i32, BlockType>,
    ) -> Result<i32> {
        Ok(building_block_map
            .get(building_id)
            .ok_or(KeyError {
                key: *building_id,
                hashmap: "building_block_map".to_string(),
            })?
            .id)
    }

    // get new instance with map_id
    pub fn new(conn: &mut PgConnection, map_id: i32) -> Result<Self> {
        let map_spaces = Self::get_building_map_spaces(conn, map_id)?;
        let building_types = Self::get_building_types(conn)?;
        let building_block_map = Self::get_building_block_map(conn)?;
        let mut buildings: HashMap<i32, Building> = HashMap::new();
        let buildings_grid: [[i32; MAP_SIZE]; MAP_SIZE] =
            Self::get_building_grid(conn, map_id, &building_block_map)?;
        // let road_map_spaces: Vec<MapSpaces> = Self::get_road_map_spaces(conn, map_id)?;

        for map_space in map_spaces {
            let blk_type = Self::get_block_id(&map_space.building_type, &building_block_map)?;

            let (absolute_entrance_x, absolute_entrance_y) =
                Self::get_absolute_entrance(&map_space, &building_types[&blk_type].block_type)?;
            buildings.insert(
                map_space.id,
                Building {
                    map_space,
                    absolute_entrance_x,
                    absolute_entrance_y,
                    population: 0,
                },
            );
        }

        let shortest_paths = Self::get_shortest_paths(conn, map_id)?;
        Ok(BuildingsManager {
            buildings,
            shortest_paths,
            buildings_grid,
        })
    }
}

// Methods
impl BuildingsManager {
    pub fn get_building_stats(&self) -> Vec<BuildingStats> {
        self.buildings
            .values()
            .map(|building| BuildingStats {
                mapsace_id: building.map_space.id,
                population: building.population,
            })
            .collect()
    }
}
