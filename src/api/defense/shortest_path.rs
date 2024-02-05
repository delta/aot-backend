use crate::constants::*;
use crate::error::DieselError;
use crate::models::*;
use crate::schema::{block_type, map_spaces, shortest_path};
use crate::util::function;
use anyhow::Result;
use array2d::Array2D;
use diesel::prelude::*;
use diesel::RunQueryDsl;
use diesel::{PgConnection, QueryDsl};
use petgraph::algo::astar;
use petgraph::Graph;
use std::collections::HashMap;

const NO_BLOCK: i32 = -1;

//running shortest path simulation
pub fn run_shortest_paths(conn: &mut PgConnection, input_map_layout_id: i32) -> Result<()> {
    // reading map_spaces
    let roads_list: Vec<(i32, i32)> = map_spaces::table
        .inner_join(block_type::table)
        .filter(map_spaces::map_id.eq(input_map_layout_id))
        .filter(block_type::building_type.eq(ROAD_ID))
        .select((map_spaces::x_coordinate, map_spaces::y_coordinate))
        .load::<(i32, i32)>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?
        .to_vec();

    // initialising maps for index to nodes and nodes to index
    let mut node_to_index = HashMap::new();
    let mut index_to_node = HashMap::new();

    // initialising 2d array and petgraph Graph
    let mut graph_2d = Array2D::filled_with(NO_BLOCK, MAP_SIZE, MAP_SIZE);
    let mut graph = Graph::<usize, usize>::new();

    // Initialising nodes, filling 2d array and the node_to_index and index_to_node maps
    for road in &roads_list {
        let single_node = graph.add_node(0);
        let (road_x, road_y) = (road.0, road.1);
        graph_2d
            .set(road_x as usize, road_y as usize, ROAD_ID)
            .unwrap();
        node_to_index.insert(
            single_node,
            (road_x as usize) * MAP_SIZE + (road_y as usize),
        );
        index_to_node.insert(
            (road_x as usize) * MAP_SIZE + (road_y as usize),
            single_node,
        );
    }

    // adding edges to graph from 2d array (2 nearby nodes)
    for i in 0..MAP_SIZE {
        for j in 0..MAP_SIZE {
            if graph_2d[(i, j)] != NO_BLOCK {
                // i,j->i+1,j
                if i + 1 < MAP_SIZE && graph_2d[(i + 1, j)] != NO_BLOCK {
                    graph.extend_with_edges([(
                        index_to_node[&(i * MAP_SIZE + j)],
                        index_to_node[&((i + 1) * MAP_SIZE + j)],
                        1,
                    )]);
                    graph.extend_with_edges([(
                        index_to_node[&((i + 1) * MAP_SIZE + j)],
                        index_to_node[&(i * MAP_SIZE + j)],
                        1,
                    )]);
                }
                //i,j->i,j+1
                if j + 1 < MAP_SIZE && graph_2d[(i, j + 1)] != NO_BLOCK {
                    graph.extend_with_edges([(
                        index_to_node[&(i * MAP_SIZE + j)],
                        index_to_node[&(i * MAP_SIZE + (j + 1))],
                        1,
                    )]);
                    graph.extend_with_edges([(
                        index_to_node[&(i * MAP_SIZE + (j + 1))],
                        index_to_node[&(i * MAP_SIZE + j)],
                        1,
                    )]);
                }
            }
        }
    }

    // Astar algorithm between EVERY PAIR of nodes
    let mut shortest_paths = vec![];
    for i in &roads_list {
        for j in &roads_list {
            let (start_road_x, start_road_y) = (i.0, i.1);
            let (dest_road_x, dest_road_y) = (j.0, j.1);
            let start_node =
                index_to_node[&((start_road_x as usize) * MAP_SIZE + (start_road_y as usize))];
            let dest_node =
                index_to_node[&((dest_road_x as usize) * MAP_SIZE + (dest_road_y as usize))];
            let path = astar(
                &graph,
                start_node,
                |finish| finish == dest_node,
                |e| *e.weight(),
                |_| 0,
            );

            match path {
                Some(p) => {
                    let new_shortest_path_entry = NewShortestPath {
                        base_id: input_map_layout_id,
                        source_x: node_to_index[&start_node] as i32 % MAP_SIZE as i32,
                        source_y: node_to_index[&start_node] as i32 / MAP_SIZE as i32,
                        dest_x: node_to_index[&dest_node] as i32 % MAP_SIZE as i32,
                        dest_y: node_to_index[&dest_node] as i32 / MAP_SIZE as i32,
                        next_hop_x: node_to_index[&p.1[1]] as i32 % MAP_SIZE as i32,
                        next_hop_y: node_to_index[&p.1[1]] as i32 / MAP_SIZE as i32,
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

    // Writing entries to shortest_path table
    let chunks: Vec<&[NewShortestPath]> = shortest_paths.chunks(1000).collect();
    for chunk in chunks {
        diesel::insert_into(shortest_path::table)
            .values(chunk)
            .execute(conn)
            .map_err(|err| DieselError {
                table: "shortest_path",
                function: function!(),
                error: err,
            })?;
    }
    Ok(())
}
