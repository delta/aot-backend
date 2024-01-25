use crate::error::DieselError;
use crate::models::{AttackerType, BuildingType, User, MineType, DefenderType};
use anyhow::Result;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;


fn deduct_user_artifacts(user_id: i32, upgrade_cost: i32, conn: &PgConnection) -> Result<()> {
    use crate::schema::{user};
    diesel::update(user::table.filter(user::id.eq(user_id)))
        .set(user::artifacts.eq(user::artifacts - upgrade_cost))
        .execute(conn)?
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;

    Ok(())
}



fn update_available_blocks(user_id: i32, variant: i32, conn: &PgConnection) -> Result<()> {
    use crate::schema::{available_blocks, building_type, user};

    let name = building_type::table
        .select(building_type::name)
        .filter(building_type::id.eq(variant))
        .first::<String>(conn)?
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    let new_level = building_type::table
        .select(building_type::level)
        .filter(building_type::id.eq(variant))
        .first::<i32>(conn)?
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;  

    let old_level_row = building_type::table
        .filter(building_type::name.eq(&name).and(building_type::level.eq(new_level - 1)))
        .first::<BuildingType>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    diesel::update(available_blocks::table.filter(
        available_blocks::user_id.eq(user_id)
        .and(available_blocks::block_type_id.eq(old_level_row.id))
    ))
    .set(available_blocks::block_type_id.eq(variant))
    .execute(conn)?
    .map_err(|err| DieselError {
        table: "available_blocks",
        function: function!(),
        error: err,
    })?;

    Ok(())
}

fn update_available_blocks_attacker(user_id: i32, variant: i32, conn: &PgConnection) -> Result<()> {
    use crate::schema::{available_blocks, attacker_type, user};

    let attacker_name = attacker_type::table
        .select(attacker_type::name)
        .filter(attacker_type::id.eq(variant))
        .first::<String>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    let new_level = attacker_type::table
        .select(attacker_type::level)
        .filter(attacker_type::id.eq(variant))
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    let old_level_row = attacker_type::table
        .filter(attacker_type::name.eq(&attacker_name).and(attacker_type::level.eq(new_level - 1)))
        .first::<AttackerType>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    diesel::update(available_blocks::table.filter(
        available_blocks::user_id.eq(user_id)
        .and(available_blocks::block_type_id.eq(old_level_row.id))
    ))
    .set(available_blocks::block_type_id.eq(variant))
    .execute(conn)
    .map_err(|err| DieselError {
        table: "available_blocks",
        function: function!(),
        error: err,
    })?;

    Ok(())
}


pub enum NextLevelResult {
    Building(BuildingType),
    Mine(MineType),
    Defender(DefenderType),
}

pub fn get_next_level(block_id: i32, conn: &PgConnection) -> Result<Option<NextLevelResult>> {
    use crate::schema::{block_type, building_type};

    let old_level = building_type::table
        .select(building_type::level)
        .filter(building_type::id.eq(block_id))
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    let name = building_type::table
        .select(building_type::name)
        .filter(building_type::id.eq(block_id))
        .first::<String>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    let category = block_type::table
        .select(block_type::category)
        .filter(block_type::id.eq(block_id))
        .first::<String>(conn)
        .map_err(|err| DieselError {
            table: "block_type",
            function: function!(),
            error: err,
        })?;

    let next_level_id = building_type::table
        .filter(
            building_type::name.eq(name)
                .and(building_type::level.eq(old_level + 1)),
        )
        .select(building_type::id)
        .first::<i32>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    let next_level_row = match category {
        "building" | "bank" => {
            building_type::table
                .filter(building_type::id.eq(next_level_id))
                .first::<BuildingType>(conn)
                .optional()
                .map_err(|err| DieselError {
                    table: "building_type",
                    function: function!(),
                    error: err,
                })
                .map(NextLevelResult::Building)
        }
        "defender" => {
            defender_type::table
                .filter(defender_type::id.eq(next_level_id))
                .first::<DefenderType>(conn)
                .optional()
                .map_err(|err| DieselError {
                    table: "defender_type",
                    function: function!(),
                    error: err,
                })
                .map(NextLevelResult::Defender)
        }
        "mine" => {
            mine_type::table
                .filter(mine_type::id.eq(next_level_id))
                .first::<MineType>(conn)
                .optional()
                .map_err(|err| DieselError {
                    table: "mine_type",
                    function: function!(),
                    error: err,
                })
                .map(NextLevelResult::Mine)
        }
        _ => None,
    };

    Ok(next_level_row)
}

pub fn get_next_level_attacker(block_id: i32, conn: &PgConnection) -> Result<Option<AttackerType>> {
    use crate::schema::attacker_type;

    let old_level = attacker_type::table
        .select(attacker_type::level)
        .filter(attacker_type::id.eq(block_id))
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    let name = attacker_type::table
        .select(attacker_type::name)
        .filter(attacker_type::id.eq(block_id))
        .first::<String>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    let next_level_row = attacker_type::table
        .filter(
            attacker_type::name.eq(&name)
                .and(attacker_type::level.eq(old_level + 1)),
        )
        .first::<AttackerType>(conn)
        .optional()
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?;

    Ok(next_level_row)
}