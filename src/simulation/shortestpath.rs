use crate::models::*;
use crate::schema::{block_type, map_spaces, shortest_path};
use array2d::Array2D;
use diesel::prelude::*;
use diesel::{PgConnection, QueryDsl};
use petgraph::algo::astar;
use petgraph::Graph;
use std::collections::HashMap;

// function to get absolute coordinates
fn get_absolute_coordinates(
    rotation: i32,
    x_coordinate: i32,
    y_coordinate: i32,
    entrance_x: i32,
    entrance_y: i32,
) -> (i32, i32) {
    match rotation {
        0 => (x_coordinate + entrance_x, y_coordinate + entrance_y),
        90 => (x_coordinate - entrance_y, y_coordinate + entrance_x),
        180 => (x_coordinate - entrance_x, y_coordinate - entrance_y),
        270 => (x_coordinate + entrance_y, y_coordinate - entrance_x),
        _ => panic!("Invalid Map Space Rotation"),
    }
}

//running shortest path simulation
pub fn run_shortest_paths(
    conn: &PgConnection,
    input_map_layout_id: i32,
    map_size: usize,
    road_id: i32,
) {
    // reading map_spaces
    let mapspaces_list = map_spaces::table
        .filter(map_spaces::map_id.eq(input_map_layout_id))
        .load::<MapSpaces>(conn)
        .expect("Couldn't get spaces");

    // reading blocks_list
    let blocks_list = block_type::table
        .load::<BlockType>(conn)
        .expect("Couldn't get road id");

    // initialising map for types of blocks
    let mut map = HashMap::new();

    // initialising maps for index to nodes and nodes to index
    let mut node_to_index = HashMap::new();
    let mut index_to_node = HashMap::new();

    // filling block types in map
    for p in blocks_list {
        map.insert(p.id, (p.width, p.height, p.entrance_x, p.entrance_y));
    }

    // initialising 2d array and petgraph Graph
    let mut graph_2d = Array2D::filled_with(0, map_size, map_size);
    let mut graph = Graph::<usize, usize>::new();

    // Initialising nodes, filling 2d array and the node_to_index and index_to_node maps
    for i in &mapspaces_list {
        let single_node = graph.add_node(0);
        let (absolute_entrance_x, absolute_entrance_y) = get_absolute_coordinates(
            i.rotation,
            i.x_coordinate,
            i.y_coordinate,
            map[&i.blk_type].2,
            map[&i.blk_type].3,
        );
        graph_2d
            .set(
                absolute_entrance_y as usize,
                absolute_entrance_x as usize,
                i.blk_type as usize,
            )
            .unwrap();
        node_to_index.insert(
            single_node,
            (absolute_entrance_y as usize) * map_size + (absolute_entrance_x as usize),
        );
        index_to_node.insert(
            (absolute_entrance_y as usize) * map_size + (absolute_entrance_x as usize),
            single_node,
        );
    }

    // // Uncomment below lines to print 2d array (map grid)
    // for row_iter in graph_2d.rows_iter() {
    //     for element in row_iter {
    //         print!("{} ", element);
    //     }
    //     println!();
    // }

    // adding edges to graph from 2d array (2 nearby nodes)
    for i in 0..map_size {
        for j in 0..map_size {
            if graph_2d[(i, j)] != 0 {
                // i,j->i+1,j
                if i + 1 < map_size && graph_2d[(i + 1, j)] != 0 {
                    graph.extend_with_edges(&[(
                        index_to_node[&(i * map_size + j)],
                        index_to_node[&((i + 1) * map_size + j)],
                        1,
                    )]);
                    graph.extend_with_edges(&[(
                        index_to_node[&((i + 1) * map_size + j)],
                        index_to_node[&(i * map_size + j)],
                        1,
                    )]);
                }
                //i,j->i,j+1
                if j + 1 < map_size && graph_2d[(i, j + 1)] != 0 {
                    graph.extend_with_edges(&[(
                        index_to_node[&(i * map_size + j)],
                        index_to_node[&(i * map_size + (j + 1))],
                        1,
                    )]);
                    graph.extend_with_edges(&[(
                        index_to_node[&(i * map_size + (j + 1))],
                        index_to_node[&(i * map_size + j)],
                        1,
                    )]);
                }
            }
        }
    }

    // // Uncomment to print graph
    // println!("{:?}",&graph);

    // Astar algorithm between EVERY PAIR of nodes
    let mut shortest_paths = vec![];
    for i in &mapspaces_list {
        for j in &mapspaces_list {
            if j.blk_type != road_id {
                let (start_absolute_entrance_x, start_absolute_entrance_y) =
                    get_absolute_coordinates(
                        i.rotation,
                        i.x_coordinate,
                        i.y_coordinate,
                        map[&i.blk_type].2,
                        map[&i.blk_type].3,
                    );
                let (dest_absolute_entrance_x, dest_absolute_entrance_y) = get_absolute_coordinates(
                    j.rotation,
                    j.x_coordinate,
                    j.y_coordinate,
                    map[&j.blk_type].2,
                    map[&j.blk_type].3,
                );
                let start_node = index_to_node[&((start_absolute_entrance_y as usize) * map_size
                    + (start_absolute_entrance_x as usize))];
                let dest_node = index_to_node[&((dest_absolute_entrance_y as usize) * map_size
                    + (dest_absolute_entrance_x as usize))];
                let path = astar(
                    &graph,
                    start_node,
                    |finish| finish == dest_node,
                    |e| *e.weight(),
                    |_| 0,
                );

                match path {
                    Some(p) => {
                        let mut path_string = String::new();

                        // Building the path string
                        for i in p.1 {
                            path_string.push('(');
                            path_string.push_str(&(node_to_index[&i] % map_size).to_string());
                            path_string.push(',');
                            path_string.push_str(
                                &(node_to_index[&i] as i32 / map_size as i32).to_string(),
                            );
                            path_string.push(')');
                        }

                        let new_shortest_path_entry = NewShortestPath {
                            base_id: input_map_layout_id,
                            source_x: node_to_index[&start_node] as i32 % map_size as i32,
                            source_y: node_to_index[&start_node] as i32 / map_size as i32,
                            dest_x: node_to_index[&dest_node] as i32 % map_size as i32,
                            dest_y: node_to_index[&dest_node] as i32 / map_size as i32,
                            pathlist: path_string,
                        };

                        shortest_paths.push(new_shortest_path_entry);
                    }
                    None => println!(
                        "No path found between {} and {}",
                        node_to_index[&start_node], node_to_index[&dest_node]
                    ),
                };
            }
        }
    }

    // Writing entries to shortest_path table
    let chunks: Vec<&[NewShortestPath]> = shortest_paths.chunks(1000).collect();
    for chunk in chunks {
        diesel::insert_into(shortest_path::table)
            .values(chunk)
            .execute(conn)
            .expect("Error saving shortest path.");
    }

    println!("Shortest path simulation completed!");
}
