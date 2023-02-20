/// CRUD functions
use super::MapSpacesEntry;
use crate::api::defense::shortest_path::run_shortest_paths;
use crate::api::util::GameHistoryEntry;
use crate::api::{self, attack};
use crate::constants::{DEFENSE_END_TIME, DEFENSE_START_TIME, DRONE_LIMIT_PER_BASE, ROAD_ID};
use crate::models::*;
use crate::util::function;
use crate::{api::util::GameHistoryResponse, error::DieselError};
use anyhow::{Ok, Result};
use chrono::{Local, NaiveTime};
use diesel::dsl::exists;
use diesel::{prelude::*, select};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize)]
pub struct MineTypeResponse {
    pub id: i32,
    pub radius: i32,
    pub damage: i32,
    pub building_id: i32,
}

#[derive(Serialize)]
pub struct DiffuserTypeResponse {
    pub id: i32,
    pub radius: i32,
    pub speed: i32,
    pub building_id: i32,
}

#[derive(Serialize)]
pub struct DefenderTypeResponse {
    pub id: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub building_id: i32,
    pub block: BlockTypeResponse,
}

#[derive(Serialize)]
pub struct BlockTypeResponse {
    pub id: i32,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub entrance_x: i32,
    pub entrance_y: i32,
    pub capacity: i32,
    pub building_id: i32,
}

#[derive(Serialize)]
pub struct DefenseResponse {
    pub map_spaces: Vec<MapSpaces>,
    pub blocks: Vec<BlockTypeResponse>,
    pub levels_fixture: LevelsFixture,
    pub level_constraints: Vec<LevelConstraints>,
    pub attack_type: Vec<AttackType>,
    pub defender_types: Vec<DefenderTypeResponse>,
    pub diffuser_types: Vec<DiffuserTypeResponse>,
    pub mine_types: Vec<MineTypeResponse>,
    pub attacker_types: Vec<AttackerType>,
    pub no_of_drones: i32,
}

#[derive(Deserialize, Serialize)]
pub struct DefenceHistoryResponse {
    pub games: Vec<Game>,
}

pub fn is_defense_allowed_now() -> bool {
    let start_time = NaiveTime::parse_from_str(DEFENSE_START_TIME, "%H:%M:%S").unwrap();
    let end_time = NaiveTime::parse_from_str(DEFENSE_END_TIME, "%H:%M:%S").unwrap();
    let current_time = Local::now().naive_local().time();
    current_time >= start_time || current_time <= end_time
}

pub fn defender_exists(defender: i32, conn: &mut PgConnection) -> Result<bool> {
    use crate::schema::user;

    Ok(select(exists(user::table.filter(user::id.eq(defender))))
        .get_result(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?)
}

pub fn fetch_map_layout(conn: &mut PgConnection, player: &i32) -> Result<MapLayout> {
    use crate::schema::map_layout;

    let level_id = &api::util::get_current_levels_fixture(conn)?.id;
    let layout = map_layout::table
        .filter(map_layout::player.eq(player))
        .filter(map_layout::level_id.eq(level_id))
        .first::<MapLayout>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;

    if let Some(layout) = layout {
        Ok(layout)
    } else {
        let new_map_layout = NewMapLayout { player, level_id };
        Ok(diesel::insert_into(map_layout::table)
            .values(&new_map_layout)
            .get_result(conn)
            .map_err(|err| DieselError {
                table: "map_layout",
                function: function!(),
                error: err,
            })?)
    }
}

pub fn fetch_map_layout_from_game(
    conn: &mut PgConnection,
    game_id: i32,
) -> Result<Option<MapLayout>> {
    use crate::schema::{game, map_layout};

    let map_layout_id = game::table
        .select(game::map_layout_id)
        .find(game_id)
        .first::<i32>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?;

    if let Some(map_layout_id) = map_layout_id {
        let map_layout = map_layout::table
            .find(map_layout_id)
            .first::<MapLayout>(conn)
            .map_err(|err| DieselError {
                table: "map_layout",
                function: function!(),
                error: err,
            })?;
        Ok(Some(map_layout))
    } else {
        Ok(None)
    }
}

pub fn get_details_from_map_layout(
    conn: &mut PgConnection,
    map: MapLayout,
) -> Result<DefenseResponse> {
    use crate::schema::{attack_type, level_constraints, levels_fixture, map_spaces};

    let map_spaces = map_spaces::table
        .filter(map_spaces::map_id.eq(map.id))
        .load::<MapSpaces>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?;
    let blocks = fetch_building_blocks(conn)?;
    let levels_fixture = levels_fixture::table
        .find(map.level_id)
        .first::<LevelsFixture>(conn)
        .map_err(|err| DieselError {
            table: "levels_fixture",
            function: function!(),
            error: err,
        })?;
    let level_constraints = level_constraints::table
        .filter(level_constraints::level_id.eq(map.level_id))
        .load::<LevelConstraints>(conn)
        .map_err(|err| DieselError {
            table: "level_constraints",
            function: function!(),
            error: err,
        })?;
    let attack_type = attack_type::table
        .load::<AttackType>(conn)
        .map_err(|err| DieselError {
            table: "attack_type",
            function: function!(),
            error: err,
        })?;

    let mine_types = fetch_mine_types(conn)?;
    let defender_types = fetch_defender_types(conn)?;
    let diffuser_types = fetch_diffuser_types(conn)?;
    let attacker_types = fetch_attacker_types(conn)?;

    Ok(DefenseResponse {
        map_spaces,
        blocks,
        levels_fixture,
        level_constraints,
        attack_type,
        mine_types,
        defender_types,
        diffuser_types,
        attacker_types,
        no_of_drones: -1,
    })
}

pub fn get_map_details_for_attack(
    attacker_id: i32,
    conn: &mut PgConnection,
    map: MapLayout,
) -> Result<DefenseResponse> {
    use crate::schema::{
        attack_type, building_type, level_constraints, levels_fixture, map_spaces,
    };

    let map_spaces = map_spaces::table
        .inner_join(building_type::table)
        .filter(map_spaces::map_id.eq(map.id))
        .load::<(MapSpaces, BuildingType)>(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(mut map_space, building_type)| {
            if building_type.blk_type == ROAD_ID {
                map_space.building_type = ROAD_ID;
                map_space
            } else {
                map_space
            }
        })
        .collect();
    let blocks = fetch_building_blocks(conn)?;
    let levels_fixture = levels_fixture::table
        .find(map.level_id)
        .first::<LevelsFixture>(conn)
        .map_err(|err| DieselError {
            table: "levels_fixture",
            function: function!(),
            error: err,
        })?;
    let level_constraints = level_constraints::table
        .filter(level_constraints::level_id.eq(map.level_id))
        .load::<LevelConstraints>(conn)
        .map_err(|err| DieselError {
            table: "level_constraints",
            function: function!(),
            error: err,
        })?;
    let attack_type = attack_type::table
        .load::<AttackType>(conn)
        .map_err(|err| DieselError {
            table: "attack_type",
            function: function!(),
            error: err,
        })?;

    let no_of_drones_used = attack::util::get_already_used_drone_count(map.id, attacker_id, conn)?;

    let mine_types = fetch_mine_types(conn)?;
    let defender_types = fetch_defender_types(conn)?;
    let diffuser_types = fetch_diffuser_types(conn)?;
    let attacker_types = fetch_attacker_types(conn)?;
    let no_of_drones = DRONE_LIMIT_PER_BASE - no_of_drones_used as i32;

    Ok(DefenseResponse {
        map_spaces,
        blocks,
        levels_fixture,
        level_constraints,
        attack_type,
        mine_types,
        defender_types,
        diffuser_types,
        attacker_types,
        no_of_drones,
    })
}

pub fn fetch_blocks(conn: &mut PgConnection) -> Result<Vec<BlockType>> {
    use crate::schema::block_type::dsl::*;

    Ok(block_type
        .load::<BlockType>(conn)
        .map_err(|err| DieselError {
            table: "block_type",
            function: function!(),
            error: err,
        })?)
}

pub fn put_base_details(
    maps: &[MapSpacesEntry],
    map: &MapLayout,
    conn: &mut PgConnection,
) -> Result<()> {
    use crate::schema::map_spaces::dsl::*;

    diesel::delete(map_spaces)
        .filter(map_id.eq(map.id))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?;
    let m: Vec<NewMapSpaces> = maps
        .iter()
        .map(|e| NewMapSpaces {
            map_id: map.id,
            x_coordinate: e.x_coordinate,
            y_coordinate: e.y_coordinate,
            rotation: e.rotation,
            building_type: e.building_type,
        })
        .collect();
    diesel::insert_into(map_spaces)
        .values(m)
        .on_conflict_do_nothing()
        .execute(conn)
        .map_err(|err| DieselError {
            table: "map_spaces",
            function: function!(),
            error: err,
        })?;

    Ok(())
}

pub fn get_level_constraints(
    conn: &mut PgConnection,
    map_level_id: i32,
) -> Result<HashMap<i32, i32>> {
    use crate::schema::level_constraints::dsl::*;

    Ok(level_constraints
        .filter(level_id.eq(map_level_id))
        .load::<LevelConstraints>(conn)
        .map_err(|err| DieselError {
            table: "level_constraints",
            function: function!(),
            error: err,
        })?
        .iter()
        .map(|constraint| (constraint.building_id, constraint.no_of_buildings))
        .collect())
}

pub fn set_map_valid(conn: &mut PgConnection, map_id: i32) -> Result<()> {
    use crate::schema::map_layout::dsl::*;

    diesel::update(map_layout.find(map_id))
        .set(is_valid.eq(true))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;

    Ok(())
}

pub fn set_map_invalid(conn: &mut PgConnection, map_id: i32) -> Result<()> {
    use crate::schema::map_layout::dsl::*;

    diesel::update(map_layout.find(map_id))
        .set(is_valid.eq(false))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;

    Ok(())
}

pub fn fetch_defense_history(
    defender_id: i32,
    user_id: i32,
    conn: &mut PgConnection,
) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games_result: Result<Vec<GameHistoryEntry>> = joined_table
        .filter(game::defend_id.eq(defender_id))
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            let player_name = api::util::get_username(game.attack_id, conn)?;
            Ok(GameHistoryEntry {
                game,
                player_name,
                is_replay_available,
            })
        })
        .collect();
    let games = games_result?;
    Ok(GameHistoryResponse { games })
}

pub fn fetch_top_defenses(user_id: i32, conn: &mut PgConnection) -> Result<GameHistoryResponse> {
    use crate::schema::{game, levels_fixture, map_layout};

    let joined_table = game::table.inner_join(map_layout::table.inner_join(levels_fixture::table));
    let games_result: Result<Vec<GameHistoryEntry>> = joined_table
        .order_by(game::defend_score.desc())
        .limit(10)
        .load::<(Game, (MapLayout, LevelsFixture))>(conn)
        .map_err(|err| DieselError {
            table: "game",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(game, (_, levels_fixture))| {
            let is_replay_available = api::util::can_show_replay(user_id, &game, &levels_fixture);
            let player_name = api::util::get_username(game.defend_id, conn)?;
            Ok(GameHistoryEntry {
                game,
                player_name,
                is_replay_available,
            })
        })
        .collect();
    let games = games_result?;
    Ok(GameHistoryResponse { games })
}

pub fn fetch_mine_types(conn: &mut PgConnection) -> Result<Vec<MineTypeResponse>> {
    use crate::schema::building_type;
    use crate::schema::mine_type;

    let joined_table = building_type::table.inner_join(mine_type::table);

    let mines: Result<Vec<MineTypeResponse>> = joined_table
        .load::<(BuildingType, MineType)>(conn)
        .map_err(|err| DieselError {
            table: "mine_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(building_type, mine_type)| {
            Ok(MineTypeResponse {
                id: mine_type.id,
                radius: mine_type.radius,
                damage: mine_type.damage,
                building_id: building_type.id,
            })
        })
        .collect();
    mines
}

pub fn fetch_diffuser_types(conn: &mut PgConnection) -> Result<Vec<DiffuserTypeResponse>> {
    use crate::schema::{building_type, diffuser_type};

    let joined_table = building_type::table.inner_join(diffuser_type::table);
    let diffusers: Result<Vec<DiffuserTypeResponse>> = joined_table
        .load::<(BuildingType, DiffuserType)>(conn)
        .map_err(|err| DieselError {
            table: "diffuser_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(building_type, diffuser_type)| {
            Ok(DiffuserTypeResponse {
                id: diffuser_type.id,
                radius: diffuser_type.radius,
                speed: diffuser_type.speed,
                building_id: building_type.id,
            })
        })
        .collect();

    diffusers
}

pub fn fetch_defender_types(conn: &mut PgConnection) -> Result<Vec<DefenderTypeResponse>> {
    use crate::schema::{block_type, building_type, defender_type};

    let joined_table = building_type::table
        .inner_join(defender_type::table)
        .inner_join(block_type::table);
    let defenders: Result<Vec<DefenderTypeResponse>> = joined_table
        .load::<(BuildingType, DefenderType, BlockType)>(conn)
        .map_err(|err| DieselError {
            table: "defender_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(building_type, defender_type, block_type)| {
            Ok(DefenderTypeResponse {
                id: defender_type.id,
                radius: defender_type.radius,
                speed: defender_type.speed,
                damage: defender_type.damage,
                building_id: building_type.id,
                block: BlockTypeResponse {
                    id: block_type.id,
                    name: block_type.name,
                    width: block_type.width,
                    height: block_type.height,
                    entrance_x: block_type.entrance_x,
                    entrance_y: block_type.entrance_y,
                    capacity: block_type.capacity,
                    building_id: building_type.id,
                },
            })
        })
        .collect();
    defenders
}

pub fn fetch_building_blocks(conn: &mut PgConnection) -> Result<Vec<BlockTypeResponse>> {
    use crate::schema::{block_type, building_type};

    let joined_table = building_type::table
        .filter(building_type::building_category.eq(BuildingCategory::Building))
        .inner_join(block_type::table);
    let blocks: Vec<BlockTypeResponse> = joined_table
        .load::<(BuildingType, BlockType)>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(building_type, block_type)| BlockTypeResponse {
            id: block_type.id,
            name: block_type.name,
            width: block_type.width,
            height: block_type.height,
            entrance_x: block_type.entrance_x,
            entrance_y: block_type.entrance_y,
            capacity: block_type.capacity,
            building_id: building_type.id,
        })
        .collect();
    Ok(blocks)
}

pub fn fetch_buildings(conn: &mut PgConnection) -> Result<HashMap<i32, BuildingType>> {
    use crate::schema::building_type::dsl::*;
    Ok(building_type
        .load::<BuildingType>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|building| (building.id, building))
        .collect::<HashMap<i32, BuildingType>>())
}

pub fn fetch_attacker_types(conn: &mut PgConnection) -> Result<Vec<AttackerType>> {
    use crate::schema::attacker_type::dsl::*;
    Ok(attacker_type
        .load::<AttackerType>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?)
}

pub fn calculate_shortest_paths(
    conn: &mut PgConnection,
    map_id: i32,
    blocks: &Vec<BlockType>,
) -> Result<()> {
    use crate::schema::shortest_path::dsl::*;

    diesel::delete(shortest_path.filter(base_id.eq(map_id)))
        .execute(conn)
        .map_err(|err| DieselError {
            table: "shortest_path",
            function: function!(),
            error: err,
        })?;
    run_shortest_paths(conn, map_id, blocks)?;

    Ok(())
}
