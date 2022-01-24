use actix_web::web::{Data, Json};
use actix_web::{web, HttpResponse, Responder, Result};
use diesel::pg::PgConnection;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
type Pool = diesel::r2d2::Pool<ConnectionManager<PgConnection>>;
use crate::diesel::RunQueryDsl;
use crate::models::*;
use crate::schema::block_type::{dsl::*, id as blk_id};
use crate::schema::levels_fixture::{dsl::*, id};
use crate::schema::map_layout::dsl::*;
use crate::schema::map_spaces::dsl::*;
use diesel::expression_methods::ExpressionMethods;
use diesel::query_dsl::QueryDsl;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::resource("")
            .route(web::put().to(set_base_details))
            .route(web::get().to(get_base_details))
            .route(web::post().to(confirm_base_details)),
    );
}
#[derive(Deserialize, Clone)]
pub struct InputNewMapSpaces {
    pub map_id: i32,
    pub blk_type: i32,
    pub x_coordinate: i32,
    pub y_coordinate: i32,
    pub rotation: i32,
}

#[derive(Serialize)]
pub struct DefenseResponse {
    pub map_spaces: Vec<MapSpaces>,
    pub block_type: Vec<BlockType>,
    pub levels_fixture: LevelsFixture,
}

async fn get_base_details(pool: Data<Pool>) -> impl Responder {
    let connection = pool.get().expect("Failed to get connection");
    match fetch_map_layout(&connection, 2) {
        Ok(res) => match get_details_from_map_layout(res, connection) {
            Ok(res) => HttpResponse::Ok().json(res),
            Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

fn get_details_from_map_layout(
    map: MapLayout,
    connection: PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<DefenseResponse, diesel::result::Error> {
    let maps = match map_spaces
        .filter(map_id.eq(map.id))
        .load::<MapSpaces>(&connection)
    {
        Ok(vec) => vec,
        Err(err) => return Err(err),
    };

    let levelsfixture = match levels_fixture
        .filter(id.eq(map.id))
        .first::<LevelsFixture>(&connection)
    {
        Ok(res) => res,
        Err(err) => return Err(err),
    };

    let blocks: Vec<BlockType> = match fetch_blocks(&connection) {
        Ok(b) => b,
        Err(err) => return Err(err),
    };
    Ok(DefenseResponse {
        map_spaces: maps,
        block_type: blocks,
        levels_fixture: levelsfixture,
    })
}

async fn set_base_details(
    payload: Json<Vec<InputNewMapSpaces>>,
    pool: Data<Pool>,
) -> impl Responder {
    let connection = pool.get().expect("Failed to get connection");

    let payload = payload.into_inner();

    let mut blocks = match fetch_blocks(&connection) {
        Ok(res) => res,
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    match fetch_map_layout(&connection, 2) {
        Ok(res) => match validate_layout(&payload, &res, &mut blocks) {
            true => put_base_details(&payload, &res, connection),
            false => HttpResponse::BadRequest().json("Invalid Map Layout"),
        },
        Err(err) => HttpResponse::InternalServerError().json(err.to_string()),
    }
}

fn put_base_details(
    _maps: &[InputNewMapSpaces],
    map: &MapLayout,
    connection: PooledConnection<ConnectionManager<PgConnection>>,
) -> HttpResponse {
    match diesel::delete(map_spaces)
        .filter(map_id.eq(map.id))
        .execute(&connection)
    {
        Ok(_) => {}
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    let m: Vec<NewMapSpaces> = _maps
        .iter()
        .map(|e| NewMapSpaces {
            map_id: &e.map_id,
            blk_type: &e.blk_type,
            x_coordinate: &e.x_coordinate,
            y_coordinate: &e.y_coordinate,
            rotation: &e.rotation,
        })
        .collect();

    match diesel::insert_into(map_spaces)
        .values(m)
        .on_conflict_do_nothing()
        .execute(&connection)
    {
        Ok(_) => {}
        Err(err) => return HttpResponse::InternalServerError().json(err.to_string()),
    };

    HttpResponse::Ok().json("Saved Successfully")
}

async fn confirm_base_details(payload: Json<Vec<MapSpaces>>, pool: Data<Pool>) -> impl Responder {
    let connection = pool.get().expect("Failed to get connection");

    let payload = payload.into_inner();

    let m: Vec<InputNewMapSpaces> = payload
        .iter()
        .map(|e| InputNewMapSpaces {
            map_id: e.map_id,
            blk_type: e.blk_type,
            x_coordinate: e.x_coordinate,
            y_coordinate: e.y_coordinate,
            rotation: e.rotation,
        })
        .collect();
    let mut blocks = match fetch_blocks(&connection) {
        Ok(res) => res,
        Err(_) => todo!(),
    };
    match fetch_map_layout(&connection, 2) {
        Ok(res) => match validate_layout(&m, &res, &mut blocks) {
            true => check_road_connectivity(),
            false => HttpResponse::BadRequest().json("Invalid Map Layout"),
        },
        Err(_) => todo!(),
    }
}

fn fetch_blocks(
    connection: &PooledConnection<ConnectionManager<PgConnection>>,
) -> Result<Vec<BlockType>, diesel::result::Error> {
    match block_type
        .order_by(blk_id.asc())
        .load::<BlockType>(connection)
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

fn fetch_map_layout(
    connection: &PooledConnection<ConnectionManager<PgConnection>>,
    player_id: i32,
) -> Result<MapLayout, diesel::result::Error> {
    match map_layout
        .filter(player.eq(player_id))
        .first::<MapLayout>(connection)
    {
        Ok(res) => Ok(res),
        Err(err) => Err(err),
    }
}

//checks overlaps of blocks and also within map size
fn validate_layout(
    maps: &[InputNewMapSpaces],
    map: &MapLayout,
    blocks: &mut Vec<BlockType>,
) -> bool {
    let mut hash: HashMap<(i32, i32), &BlockType> = HashMap::new();

    for m in maps {
        match blocks.binary_search_by_key(&m.blk_type, |block| block.id) {
            Ok(res) => {
                if m.map_id != map.id {
                    return false;
                }
                let block: &BlockType = blocks.get(res).unwrap();
                let (x, y, wid, hei) = get_absolute_coordinates(
                    m.rotation,
                    (m.x_coordinate, m.y_coordinate),
                    (block.width, block.height),
                );
                if x == -1 {
                    return false;
                }
                let mut i = 0;
                let mut j;

                while i < hei {
                    j = 0;
                    while j < wid {
                        if x + i >= 0 && x + i <= 40 && y + j >= 0 && y + j <= 40 {
                            if let std::collections::hash_map::Entry::Vacant(e) =
                                hash.entry((x + i, y + j))
                            {
                                e.insert(block);
                            } else {
                                return false;
                            }
                        } else {
                            return false;
                        }
                        j += 1;
                    }
                    i += 1;
                }
            }
            Err(_) => {
                return false;
            }
        }
    }
    for m in maps {
        if !check_is_road_available(m, &mut hash) {
            return false;
        }
    }
    true
}

fn check_is_road_available(
    m: &InputNewMapSpaces,
    hash: &mut HashMap<(i32, i32), &BlockType>,
) -> bool {
    let mut x = 0;
    let mut y = 0;
    let blk: &BlockType = hash.get(&(m.x_coordinate, m.y_coordinate)).unwrap();
    if blk.name != "road" {
        match m.rotation {
            0 => {
                x = m.x_coordinate + blk.entrance_x;
                y = m.y_coordinate + blk.entrance_y;
            }
            90 => {
                x = m.x_coordinate - blk.entrance_y;
                y = m.y_coordinate + blk.entrance_x;
            }
            180 => {
                x = m.x_coordinate - blk.entrance_x;
                y = m.y_coordinate - blk.entrance_y;
            }
            270 => {
                x = m.x_coordinate - blk.entrance_y;
                y = m.y_coordinate - blk.entrance_x;
            }
            _ => {}
        }
        if hash.contains_key(&(x + 1, y)) {
            let b: &BlockType = hash.get(&(x + 1, y)).unwrap();
            if b.name == "road" {
                return true;
            }
        };
        if hash.contains_key(&(x - 1, y)) {
            let b: &BlockType = hash.get(&(x - 1, y)).unwrap();
            if b.name == "road" {
                return true;
            }
        };
        if hash.contains_key(&(x, y + 1)) {
            let b: &BlockType = hash.get(&(x, y + 1)).unwrap();
            if b.name == "road" {
                return true;
            }
        };
        if hash.contains_key(&(x, y - 1)) {
            let b: &BlockType = hash.get(&(x, y - 1)).unwrap();
            if b.name == "road" {
                return true;
            }
        };
    } else {
        return true;
    }
    false
}
//returns equivalent left and right coordinates of current position
//and also equivalent height and width assuming that dimensions of block in current position along x axis is width and similarly for height
fn get_absolute_coordinates(
    rot: i32,
    coord: (i32, i32),
    dimen: (i32, i32),
) -> (i32, i32, i32, i32) {
    let mut x = (coord.0, coord.1, dimen.0, dimen.1);
    match rot {
        0 => {}
        90 => {
            x.0 = coord.0 - dimen.1 + 1;
            x.1 = coord.1;
            x.2 = dimen.1;
            x.3 = dimen.0;
        }
        180 => {
            x.0 = coord.0 - dimen.0 + 1;
            x.1 = coord.1 - dimen.1 + 1;
            x.2 = dimen.0;
            x.3 = dimen.1;
        }
        270 => {
            x.0 = coord.0;
            x.1 = coord.1 - dimen.0 + 1;
            x.2 = dimen.1;
            x.3 = dimen.0;
        }
        _ => {
            x.0 = -1;
            x.1 = -1;
        }
    };
    x
}

fn check_road_connectivity() -> HttpResponse {
    todo!();
    //HttpResponse::Ok().json("road connected")
}
