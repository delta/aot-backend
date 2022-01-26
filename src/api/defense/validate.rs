/// Functions to check if a base layout is valid
use crate::models::*;
use petgraph::{self, prelude::*, Graph};
use std::collections::{HashMap, HashSet};

//returns equivalent left and right coordinates of current position
//and also equivalent height and width assuming that dimensions of block in current position along x axis is width and similarly for height
fn get_absolute_coordinates(
    rot: i32,
    coord: (i32, i32),
    dimen: (i32, i32),
) -> (i32, i32, i32, i32) {
    let (x, y, width, height) = (coord.0, coord.1, dimen.0, dimen.1);
    match rot {
        0 => (x, y, width, height),
        90 => (x - height + 1, y, height, width),
        180 => (x - width + 1, y - height + 1, width, height),
        270 => (x, y - width + 1, height, width),
        _ => (-1, -1, -1, -1),
    }
}

fn get_absolute_entrance(map_space: &NewMapSpaces, block_type: &BlockType) -> (i32, i32) {
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

//checks overlaps of blocks and also within map size
pub fn is_valid_update_layout(
    map_spaces: &[NewMapSpaces],
    map: &MapLayout,
    blocks: &[BlockType],
) -> bool {
    let mut occupied_positions: HashSet<(i32, i32)> = HashSet::new();
    let blocks: HashMap<i32, BlockType> = blocks
        .iter()
        .map(|block| (block.id, block.clone()))
        .collect();

    for map_space in map_spaces {
        let blk_type = &map_space.blk_type;
        if !blocks.contains_key(blk_type) {
            return false;
        }
        if map_space.map_id != map.id {
            return false;
        }
        let block: &BlockType = blocks.get(blk_type).unwrap();
        let (x, y, width, height) = get_absolute_coordinates(
            map_space.rotation,
            (map_space.x_coordinate, map_space.y_coordinate),
            (block.width, block.height),
        );
        if x == -1 {
            return false;
        }

        for i in 0..width {
            for j in 0..height {
                if (0..40).contains(&(x + i)) && (0..40).contains(&(y + j)) {
                    if occupied_positions.contains(&(x + i, y + j)) {
                        return false;
                    }
                    occupied_positions.insert((x + i, y + j));
                } else {
                    return false;
                }
            }
        }
    }

    true
}

// checks if no of buildings are within level constraints and if the city is connected
pub fn is_valid_save_layout(
    map_spaces: &[NewMapSpaces],
    road_id: i32,
    level_constraints: &mut HashMap<i32, i32>,
    blocks: &[BlockType],
) -> bool {
    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let mut map_grid: HashMap<(i32, i32), NodeIndex> = HashMap::new();

    let blocks: HashMap<i32, BlockType> = blocks
        .iter()
        .map(|block| (block.id, block.clone()))
        .collect();

    for map_space in map_spaces {
        let NewMapSpaces {
            blk_type,
            x_coordinate,
            y_coordinate,
            ..
        } = *map_space;

        // check for level constraints
        if let Some(block_constraint) = level_constraints.get_mut(&blk_type) {
            if *block_constraint > 0 {
                *block_constraint -= 1;
            } else {
                return false;
            }
        }

        // add roads and entrances to graph
        if blk_type == road_id {
            map_grid.insert((x_coordinate, y_coordinate), graph.add_node(()));
        } else {
            let block = blocks.get(&blk_type).unwrap();
            let entrance = get_absolute_entrance(map_space, block);
            map_grid.insert(entrance, graph.add_node(()));
        }
    }

    for (coordinates, node_index) in &map_grid {
        let (x, y) = *coordinates;
        if map_grid.contains_key(&(x + 1, y)) {
            let node_index_right = map_grid.get(&(x + 1, y)).unwrap();
            graph.add_edge(*node_index, *node_index_right, ());
        }
        if map_grid.contains_key(&(x, y + 1)) {
            let node_index_down = map_grid.get(&(x, y + 1)).unwrap();
            graph.add_edge(*node_index, *node_index_down, ());
        }
    }

    petgraph::algo::connected_components(&graph) == 1
}
