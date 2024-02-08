use crate::error::DieselError;
use crate::models::AttackerType;
use crate::models::BlockCategory;
use crate::models::BuildingType;
use crate::models::DefenderType;
use crate::models::EmpType;
use crate::models::ItemCategory;
use crate::models::MineType;
use crate::schema::map_layout;
use crate::schema::map_spaces;
use crate::schema::user;
use crate::schema::{
    attacker_type, available_blocks, block_type, building_type, defender_type, emp_type, mine_type,
};
use crate::util::function;
use anyhow::Ok;
use anyhow::Result;
use diesel::dsl::exists;
use diesel::prelude::*;
use diesel::select;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct BuildingTypeResponse {
    id: i32,
    block_id: i32,
    name: String,
    width: i32,
    height: i32,
    capacity: i32,
    level: i32,
    cost: i32,
    hp: i32,
    next_level_stats: Option<NextLevelBuildingTypeResponse>,
}
#[derive(Serialize, Deserialize)]

struct NextLevelBuildingTypeResponse {
    id: i32,
    block_id: i32,
    name: String,
    width: i32,
    height: i32,
    capacity: i32,
    level: i32,
    cost: i32,
    hp: i32,
}
#[derive(Serialize, Deserialize)]

struct AttackerTypeResponse {
    id: i32,
    max_health: i32,
    speed: i32,
    amt_of_emps: i32,
    level: i32,
    cost: i32,
    name: String,
    next_level_stats: Option<NextLevelAttackerTypeResponse>,
}
#[derive(Serialize, Deserialize)]

struct NextLevelAttackerTypeResponse {
    id: i32,
    max_health: i32,
    speed: i32,
    amt_of_emps: i32,
    level: i32,
    cost: i32,
    name: String,
}
#[derive(Serialize, Deserialize)]

struct DefenderTypeResponse {
    id: i32,
    block_id: i32,
    speed: i32,
    damage: i32,
    radius: i32,
    level: i32,
    cost: i32,
    name: String,
    next_level_stats: Option<NextLevelDefenderTypeResponse>,
}
#[derive(Serialize, Deserialize)]

struct NextLevelDefenderTypeResponse {
    id: i32,
    block_id: i32,
    speed: i32,
    damage: i32,
    radius: i32,
    level: i32,
    cost: i32,
    name: String,
}
#[derive(Serialize, Deserialize)]

struct EmpTypeResponse {
    id: i32,
    att_type: String,
    attack_radius: i32,
    attack_damage: i32,
    cost: i32,
    name: String,
    level: i32,
    next_level_stats: Option<NextLevelEmpTypeResponse>,
}
#[derive(Serialize, Deserialize)]

struct NextLevelEmpTypeResponse {
    id: i32,
    att_type: String,
    attack_radius: i32,
    attack_damage: i32,
    cost: i32,
    name: String,
    level: i32,
}
#[derive(Serialize, Deserialize)]

struct MineTypeResponse {
    id: i32,
    block_id: i32,
    radius: i32,
    damage: i32,
    level: i32,
    cost: i32,
    name: String,
    next_level_stats: Option<NextLevelMineTypeResponse>,
}
#[derive(Serialize, Deserialize)]

struct NextLevelMineTypeResponse {
    id: i32,
    block_id: i32,
    radius: i32,
    damage: i32,
    level: i32,
    cost: i32,
    name: String,
}
#[derive(Serialize, Deserialize)]

struct InventoryResponse {
    buildings: Vec<BuildingTypeResponse>,
    attackers: Vec<AttackerTypeResponse>,
    defenders: Vec<DefenderTypeResponse>,
    mines: Vec<MineTypeResponse>,
    emps: Vec<EmpTypeResponse>,
}

pub(crate) fn get_inventory(player_id: i32, conn: &mut PgConnection) -> Result<InventoryResponse> {
    let buildings = get_building_types(player_id, conn)?;
    let attackers = get_attacker_types(player_id, conn)?;
    let defenders = get_defender_types(player_id, conn)?;
    let mines = get_mine_types(player_id, conn)?;
    let emps = get_emp_types(player_id, conn)?;

    Ok(InventoryResponse {
        buildings,
        attackers,
        defenders,
        mines,
        emps,
    })
}

fn get_building_types(
    player_id: i32,
    conn: &mut PgConnection,
) -> Result<Vec<BuildingTypeResponse>> {
    let joined_table = available_blocks::table
        .inner_join(block_type::table.inner_join(building_type::table))
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Building))
        .select((building_type::all_columns, block_type::id));

    let buildings = joined_table
        .load::<(BuildingType, i32)>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(building_type, block_id)| {
            if building_type.level >= 3 {
                // The building is at max level
                BuildingTypeResponse {
                    id: building_type.id,
                    block_id,
                    name: building_type.name,
                    width: building_type.width,
                    height: building_type.height,
                    capacity: building_type.capacity,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: None,
                }
            } else {
                let next_level = building_type.level + 1;
                let next_level_stats = building_type::table
                    .filter(building_type::name.eq(&building_type.name))
                    .filter(building_type::level.eq(next_level))
                    .first::<BuildingType>(conn)
                    .map_err(|err| DieselError {
                        table: "building_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(BuildingType {
                        id: 0,
                        name: "".to_string(),
                        width: 0,
                        height: 0,
                        capacity: 0,
                        level: 0,
                        cost: 0,
                        hp: 0,
                    });
                BuildingTypeResponse {
                    id: building_type.id,
                    block_id,
                    name: building_type.name,
                    width: building_type.width,
                    height: building_type.height,
                    capacity: building_type.capacity,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: Some(NextLevelBuildingTypeResponse {
                        id: next_level_stats.id,
                        block_id,
                        name: next_level_stats.name,
                        width: next_level_stats.width,
                        height: next_level_stats.height,
                        capacity: next_level_stats.capacity,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                        hp: next_level_stats.hp,
                    }),
                }
            }
        })
        .collect();

    Ok(buildings)
}

fn get_attacker_types(
    player_id: i32,
    conn: &mut PgConnection,
) -> Result<Vec<AttackerTypeResponse>> {
    let joined_table = available_blocks::table
        .inner_join(attacker_type::table)
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Attacker))
        .select(attacker_type::all_columns);

    let attackers = joined_table
        .load::<AttackerType>(conn)
        .map_err(|err| DieselError {
            table: "attacker_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|attacker_type| {
            if attacker_type.level >= 3 {
                // The attacker is at max level
                AttackerTypeResponse {
                    id: attacker_type.id,
                    max_health: attacker_type.max_health,
                    speed: attacker_type.speed,
                    amt_of_emps: attacker_type.amt_of_emps,
                    level: attacker_type.level,
                    cost: attacker_type.cost,
                    name: attacker_type.name,
                    next_level_stats: None,
                }
            } else {
                let next_level = attacker_type.level + 1;
                let next_level_stats = attacker_type::table
                    .filter(attacker_type::name.eq(&attacker_type.name))
                    .filter(attacker_type::level.eq(next_level))
                    .first::<AttackerType>(conn)
                    .map_err(|err| DieselError {
                        table: "attacker_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(AttackerType {
                        id: 0,
                        max_health: 0,
                        speed: 0,
                        amt_of_emps: 0,
                        level: 0,
                        cost: 0,
                        name: "".to_string(),
                    });
                AttackerTypeResponse {
                    id: attacker_type.id,
                    max_health: attacker_type.max_health,
                    speed: attacker_type.speed,
                    amt_of_emps: attacker_type.amt_of_emps,
                    level: attacker_type.level,
                    cost: attacker_type.cost,
                    name: attacker_type.name,
                    next_level_stats: Some(NextLevelAttackerTypeResponse {
                        id: next_level_stats.id,
                        max_health: next_level_stats.max_health,
                        speed: next_level_stats.speed,
                        amt_of_emps: next_level_stats.amt_of_emps,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                        name: next_level_stats.name,
                    }),
                }
            }
        })
        .collect();

    Ok(attackers)
}

fn get_defender_types(
    player_id: i32,
    conn: &mut PgConnection,
) -> Result<Vec<DefenderTypeResponse>> {
    let joined_table = available_blocks::table
        .inner_join(block_type::table.inner_join(defender_type::table))
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Defender))
        .select((defender_type::all_columns, block_type::id));

    let defenders = joined_table
        .load::<(DefenderType, i32)>(conn)
        .map_err(|err| DieselError {
            table: "defender_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(defender_type, block_id)| {
            if defender_type.level >= 3 {
                //the defender is at max level
                DefenderTypeResponse {
                    id: defender_type.id,
                    block_id,
                    speed: defender_type.speed,
                    damage: defender_type.damage,
                    radius: defender_type.radius,
                    level: defender_type.level,
                    cost: defender_type.cost,
                    name: defender_type.name,
                    next_level_stats: None,
                }
            } else {
                let next_level = defender_type.level + 1;
                let next_level_stats = defender_type::table
                    .filter(defender_type::name.eq(&defender_type.name))
                    .filter(defender_type::level.eq(next_level))
                    .first::<DefenderType>(conn)
                    .map_err(|err| DieselError {
                        table: "defender_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(DefenderType {
                        id: 0,
                        speed: 0,
                        damage: 0,
                        radius: 0,
                        level: 0,
                        cost: 0,
                        name: "".to_string(),
                    });
                DefenderTypeResponse {
                    id: defender_type.id,
                    block_id,
                    speed: defender_type.speed,
                    damage: defender_type.damage,
                    radius: defender_type.radius,
                    level: defender_type.level,
                    cost: defender_type.cost,
                    name: defender_type.name,
                    next_level_stats: Some(NextLevelDefenderTypeResponse {
                        id: next_level_stats.id,
                        block_id,
                        speed: next_level_stats.speed,
                        damage: next_level_stats.damage,
                        radius: next_level_stats.radius,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                        name: next_level_stats.name,
                    }),
                }
            }
        })
        .collect();

    Ok(defenders)
}

fn get_mine_types(player_id: i32, conn: &mut PgConnection) -> Result<Vec<MineTypeResponse>> {
    let joined_table = available_blocks::table
        .inner_join(block_type::table.inner_join(mine_type::table))
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Mine))
        .select((mine_type::all_columns, block_type::id));

    let mines = joined_table
        .load::<(MineType, i32)>(conn)
        .map_err(|err| DieselError {
            table: "mine_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(mine_type, block_id)| {
            if mine_type.level >= 3 {
                //mine is at max level
                MineTypeResponse {
                    id: mine_type.id,
                    block_id,
                    radius: mine_type.radius,
                    damage: mine_type.damage,
                    level: mine_type.level,
                    cost: mine_type.cost,
                    name: mine_type.name,
                    next_level_stats: None,
                }
            } else {
                let next_level = mine_type.level + 1;
                let next_level_stats = mine_type::table
                    .filter(mine_type::name.eq(&mine_type.name))
                    .filter(mine_type::level.eq(next_level))
                    .first::<MineType>(conn)
                    .map_err(|err| DieselError {
                        table: "mine_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(MineType {
                        id: 0,
                        radius: 0,
                        damage: 0,
                        level: 0,
                        cost: 0,
                        name: "".to_string(),
                    });
                MineTypeResponse {
                    id: mine_type.id,
                    block_id,
                    radius: mine_type.radius,
                    damage: mine_type.damage,
                    level: mine_type.level,
                    cost: mine_type.cost,
                    name: mine_type.name,
                    next_level_stats: Some(NextLevelMineTypeResponse {
                        id: next_level_stats.id,
                        block_id,
                        radius: next_level_stats.radius,
                        damage: next_level_stats.damage,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                        name: next_level_stats.name,
                    }),
                }
            }
        })
        .collect();

    Ok(mines)
}

fn get_emp_types(player_id: i32, conn: &mut PgConnection) -> Result<Vec<EmpTypeResponse>> {
    let joined_table = available_blocks::table
        .inner_join(emp_type::table)
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Emp))
        .select(emp_type::all_columns);

    let emps = joined_table
        .load::<EmpType>(conn)
        .map_err(|err| DieselError {
            table: "emp_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|emp_type| {
            if emp_type.level >= 3 {
                // The emp is at max level
                EmpTypeResponse {
                    id: emp_type.id,
                    att_type: emp_type.att_type,
                    attack_radius: emp_type.attack_radius,
                    attack_damage: emp_type.attack_damage,
                    cost: emp_type.cost,
                    name: emp_type.name,
                    level: emp_type.level,
                    next_level_stats: None,
                }
            } else {
                let next_level = emp_type.level + 1;
                let next_level_stats = emp_type::table
                    .filter(emp_type::name.eq(&emp_type.name))
                    .filter(emp_type::level.eq(next_level))
                    .first::<EmpType>(conn)
                    .map_err(|err| DieselError {
                        table: "emp_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(EmpType {
                        id: 0,
                        att_type: "".to_string(),
                        attack_radius: 0,
                        attack_damage: 0,
                        cost: 0,
                        name: "".to_string(),
                        level: 0,
                    });
                EmpTypeResponse {
                    id: emp_type.id,
                    att_type: emp_type.att_type,
                    attack_radius: emp_type.attack_radius,
                    attack_damage: emp_type.attack_damage,
                    cost: emp_type.cost,
                    name: emp_type.name,
                    level: emp_type.level,
                    next_level_stats: Some(NextLevelEmpTypeResponse {
                        id: next_level_stats.id,
                        att_type: next_level_stats.att_type,
                        attack_radius: next_level_stats.attack_radius,
                        attack_damage: next_level_stats.attack_damage,
                        cost: next_level_stats.cost,
                        name: next_level_stats.name,
                        level: next_level_stats.level,
                    }),
                }
            }
        })
        .collect();

    Ok(emps)
}

fn upgrade_building(player_id: i32, conn: &mut PgConnection, block_id: i32) -> Result<()> {
    let user_artifacts = get_user_artifacts(player_id, conn)?;

    //check if the given block id is a building
    //check if the given user has the block id
    let exists = select(exists(
        available_blocks::table
            .inner_join(block_type::table)
            .filter(available_blocks::user_id.eq(player_id))
            .filter(available_blocks::block_type_id.eq(block_id))
            .filter(available_blocks::category.eq(ItemCategory::Block))
            .filter(block_type::category.eq(BlockCategory::Building)),
    ))
    .get_result::<bool>(conn)?;

    if exists == false {
        return Err(anyhow::anyhow!(
            "either Block is not a building or the user does not have the block"
        ));
    }

    let joined_table = available_blocks::table
        .inner_join(block_type::table.inner_join(building_type::table))
        .filter(available_blocks::user_id.eq(player_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Building));

    let (cost, level, name): (i32, i32, String) = joined_table
        .filter(available_blocks::block_type_id.eq(block_id))
        .select((
            building_type::cost,
            building_type::level,
            building_type::name,
        ))
        .first::<(i32, i32, String)>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    let max_level: i64 = building_type::table
        .filter(building_type::name.eq(&name))
        .count()
        .get_result::<i64>(conn)
        .map_err(|err| DieselError {
            table: "building_type",
            function: function!(),
            error: err,
        })?;

    if level >= max_level as i32 {
        return Err(anyhow::anyhow!("Building is at max level"));
    };
    if cost > user_artifacts {
        return Err(anyhow::anyhow!("Not enough artifacts"));
    };

    let next_level_block_id: i32 = joined_table
        .filter(building_type::name.eq(name))
        .filter(building_type::level.eq(level + 1))
        .select(block_type::id)
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "available_blocks",
            function: function!(),
            error: err,
        })?;

    let id_of_map = map_layout::table
        .filter(map_layout::player.eq(player_id))
        .select(map_layout::id)
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "map_layout",
            function: function!(),
            error: err,
        })?;

    //updat the building level by replacing the blockid of the next level building in place of the block id of the current level building in the available blocks table
    conn.transaction(|conn| {
        diesel::update(
            available_blocks::table.filter(available_blocks::block_type_id.eq(block_id)),
        )
        .set(available_blocks::block_type_id.eq(next_level_block_id))
        .execute(conn)?;

        diesel::update(user::table.filter(user::id.eq(player_id)))
            .set(user::artifacts.eq(user_artifacts - cost))
            .execute(conn)?;

        //update map spaces
        diesel::update(
            map_spaces::table
                .filter(map_spaces::block_type_id.eq(block_id))
                .filter(map_spaces::map_id.eq(id_of_map)),
        )
        .set(map_spaces::block_type_id.eq(next_level_block_id))
        .execute(conn)?;

        diesel::result::QueryResult::Ok(())
    });
    Ok(())
}

fn get_user_artifacts(player_id: i32, conn: &mut PgConnection) -> Result<i32> {
    let artifacts = user::table
        .filter(user::id.eq(player_id))
        .select(user::artifacts)
        .first::<i32>(conn)
        .map_err(|err| DieselError {
            table: "user",
            function: function!(),
            error: err,
        })?;

    Ok(artifacts)
}
