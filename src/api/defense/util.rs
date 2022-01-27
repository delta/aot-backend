/// CRUD functions
use crate::{api::error::*, models::*};
use diesel::prelude::*;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
pub struct DefenseResponse {
    pub map_spaces: Vec<MapSpaces>,
    pub blocks: Vec<BlockType>,
    pub levels_fixture: LevelsFixture,
}

type Result<T> = std::result::Result<T, DieselError>;

pub fn fetch_map_layout(conn: &PgConnection, player_id: i32) -> Result<MapLayout> {
    use crate::schema::{levels_fixture, map_layout};
    use diesel::dsl::{date, now};

    let today = date(now.at_time_zone("Asia/Calcutta"));
    let level_id = levels_fixture::table
        .select(levels_fixture::id)
        .filter(levels_fixture::start_date.le(today))
        .filter(levels_fixture::end_date.gt(today))
        .first::<i32>(conn)?;

    Ok(map_layout::table
        .filter(map_layout::player.eq(player_id))
        .filter(map_layout::level_id.eq(level_id))
        .first::<MapLayout>(conn)?)
}

pub fn get_details_from_map_layout(map: MapLayout, conn: &PgConnection) -> Result<DefenseResponse> {
    use crate::schema::{levels_fixture, map_spaces};

    let map_spaces = map_spaces::table
        .filter(map_spaces::map_id.eq(map.id))
        .load::<MapSpaces>(conn)?;
    let levels_fixture = levels_fixture::table
        .filter(levels_fixture::id.eq(map.level_id))
        .first::<LevelsFixture>(conn)?;
    let blocks: Vec<BlockType> = fetch_blocks(conn)?;

    Ok(DefenseResponse {
        map_spaces,
        blocks,
        levels_fixture,
    })
}

pub fn fetch_blocks(conn: &PgConnection) -> Result<Vec<BlockType>> {
    use crate::schema::block_type::dsl::*;

    Ok(block_type.load::<BlockType>(conn)?)
}

pub fn put_base_details(maps: &[NewMapSpaces], map: &MapLayout, conn: &PgConnection) -> Result<()> {
    use crate::schema::map_spaces::dsl::*;

    diesel::delete(map_spaces)
        .filter(map_id.eq(map.id))
        .execute(conn)?;
    let m: Vec<NewMapSpaces> = maps
        .iter()
        .map(|e| NewMapSpaces {
            map_id: e.map_id,
            blk_type: e.blk_type,
            x_coordinate: e.x_coordinate,
            y_coordinate: e.y_coordinate,
            rotation: e.rotation,
        })
        .collect();
    diesel::insert_into(map_spaces)
        .values(m)
        .on_conflict_do_nothing()
        .execute(conn)?;

    Ok(())
}

pub fn get_road_id(conn: &PgConnection) -> Result<i32> {
    use crate::schema::block_type::dsl::*;

    Ok(block_type
        .filter(name.eq("road"))
        .select(id)
        .first::<i32>(conn)?)
}

pub fn get_level_constraints(conn: &PgConnection, map_level_id: i32) -> Result<HashMap<i32, i32>> {
    use crate::schema::level_constraints::dsl::*;

    Ok(level_constraints
        .filter(level_id.eq(map_level_id))
        .load::<LevelConstraints>(conn)?
        .iter()
        .map(|constraint| (constraint.block_id, constraint.no_of_buildings))
        .collect())
}

pub fn set_map_valid(conn: &PgConnection, map_id: i32) -> Result<()> {
    use crate::schema::map_layout::dsl::*;

    diesel::update(map_layout.find(map_id))
        .set(is_valid.eq(true))
        .execute(conn)?;

    Ok(())
}

pub fn set_map_invalid(conn: &PgConnection, map_id: i32) -> Result<()> {
    use crate::schema::map_layout::dsl::*;

    diesel::update(map_layout.find(map_id))
        .set(is_valid.eq(false))
        .execute(conn)?;

    Ok(())
}
