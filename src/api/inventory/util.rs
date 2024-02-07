use anyhow::Result;
use diesel::PgConnection;
use serde::{Deserialize, Serialize};

use crate::error::DieselError;
use crate::models::{
    AttackerType, BlockCategory, BuildingType, DefenderType, ItemCategory, MineType,
};
use crate::schema::{
    attacker_type, available_blocks, block_type, building_type, defender_type, mine_type,
};
use crate::util::function;
use diesel::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct MineTypeResponse {
    pub id: i32,
    pub radius: i32,
    pub damage: i32,
    pub level: i32,
    pub name: String,
    pub cost: i32,
    pub next_level_stats: Option<NextLevelMineTypeResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct NextLevelMineTypeResponse {
    pub id: i32,
    pub radius: i32,
    pub damage: i32,
    pub level: i32,
    pub name: String,
    pub cost: i32,
}

#[derive(Serialize, Deserialize)]
pub struct BuildingTypeResponse {
    pub id: i32,
    pub name: String,
    pub capacity: i32,
    pub level: i32,
    pub cost: i32,
    pub hp: i32,
    pub next_level_stats: Option<NextLevelBuildingTypeResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct NextLevelBuildingTypeResponse {
    pub id: i32,
    pub name: String,
    pub capacity: i32,
    pub level: i32,
    pub cost: i32,
    pub hp: i32,
}

#[derive(Serialize, Deserialize)]
pub struct DefenderTypeResponse {
    pub id: i32,
    pub speed: i32,
    pub damage: i32,
    pub radius: i32,
    pub name: String,
    pub level: i32,
    pub cost: i32,
    pub hp: i32,
    pub next_level_stats: Option<NextLevelDefenderTypeResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct NextLevelDefenderTypeResponse {
    pub id: i32,
    pub speed: i32,
    pub damage: i32,
    pub radius: i32,
    pub name: String,
    pub level: i32,
    pub cost: i32,
    pub hp: i32,
}

#[derive(Serialize, Deserialize)]
pub struct AttackerTypeResponse {
    pub id: i32,
    pub max_health: i32,
    pub speed: i32,
    pub amt_of_emps: i32,
    pub level: i32,
    pub cost: i32,
    pub next_level_stats: Option<NextLevelAttackerTypeResponse>,
}

#[derive(Serialize, Deserialize)]
pub struct NextLevelAttackerTypeResponse {
    pub id: i32,
    pub max_health: i32,
    pub speed: i32,
    pub amt_of_emps: i32,
    pub level: i32,
    pub cost: i32,
}

#[derive(Serialize, Deserialize)]
pub struct GetInventoryResponse {
    pub buildings: Vec<BuildingTypeResponse>,
    pub mines: Vec<MineTypeResponse>,
    pub attackers: Vec<AttackerTypeResponse>,
    pub defenders: Vec<DefenderTypeResponse>,
}

pub fn fetch_inventory(user_id: i32, conn: &mut PgConnection) -> Result<GetInventoryResponse> {
    //fetching available buildings of the user
    let joined_table = available_blocks::table
        .inner_join(block_type::table.inner_join(building_type::table))
        .filter(available_blocks::user_id.eq(user_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Building))
        .select(building_type::all_columns);

    let buildings: Vec<BuildingTypeResponse> = joined_table
        .load::<BuildingType>(conn)
        .map_err(|err| DieselError {
            table: "block_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|building_type| {
            if building_type.level < 3 {
                let next_level = building_type.level + 1;
                let next_level_stats = building_type::table
                    .filter(building_type::level.eq(next_level))
                    .filter(building_type::id.eq(building_type.id))
                    .first::<BuildingType>(conn)
                    .map_err(|err| DieselError {
                        table: "building_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(BuildingType {
                        id: -1,
                        name: "".to_string(),
                        width: -1,
                        height: -1,
                        level: -1,
                        cost: -1,
                        capacity: -1,
                        hp: -1,
                    });

                BuildingTypeResponse {
                    id: building_type.id,
                    name: building_type.name,
                    capacity: building_type.capacity,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: Some(NextLevelBuildingTypeResponse {
                        id: next_level_stats.id,
                        name: next_level_stats.name,
                        capacity: next_level_stats.capacity,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                        hp: next_level_stats.hp,
                    }),
                }
            } else {
                //building is at max level
                BuildingTypeResponse {
                    id: building_type.id,
                    name: building_type.name,
                    capacity: building_type.capacity,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: None,
                }
            }
        })
        .collect();

    //fetching available mines of the user
    let joined_table = available_blocks::table
        .inner_join(
            block_type::table
                .inner_join(mine_type::table)
                .inner_join(building_type::table),
        )
        .filter(available_blocks::user_id.eq(user_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Mine))
        .select((mine_type::all_columns, building_type::all_columns));

    let mines: Vec<MineTypeResponse> = joined_table
        .load::<(MineType, BuildingType)>(conn)
        .map_err(|err| DieselError {
            table: "mine_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(mine_type, building_type)| {
            if building_type.level < 3 {
                //getting next level mine stats
                let next_level = building_type.level + 1;
                let next_level_stats = block_type::table
                    .inner_join(mine_type::table)
                    .inner_join(building_type::table)
                    .filter(building_type::level.eq(next_level))
                    .filter(mine_type::id.eq(mine_type.id))
                    .select((mine_type::all_columns, building_type::all_columns))
                    .first::<(MineType, BuildingType)>(conn)
                    .map_err(|err| DieselError {
                        table: "building_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or((
                        MineType {
                            id: -1,
                            radius: -1,
                            damage: -1,
                        },
                        BuildingType {
                            id: -1,
                            name: "".to_string(),
                            width: -1,
                            height: -1,
                            level: -1,
                            cost: -1,
                            capacity: -1,
                            hp: -1,
                        },
                    ));

                let next_level_mine_stats = Some(NextLevelMineTypeResponse {
                    id: next_level_stats.0.id,
                    radius: next_level_stats.0.radius,
                    damage: next_level_stats.0.damage,
                    level: next_level_stats.1.level,
                    name: next_level_stats.1.name,
                    cost: next_level_stats.1.cost,
                });

                MineTypeResponse {
                    id: mine_type.id,
                    radius: mine_type.radius,
                    damage: mine_type.damage,
                    level: building_type.level,
                    name: building_type.name,
                    cost: building_type.cost,
                    next_level_stats: next_level_mine_stats,
                }
            } else {
                //mine is at max level
                MineTypeResponse {
                    id: mine_type.id,
                    radius: mine_type.radius,
                    damage: mine_type.damage,
                    level: building_type.level,
                    name: building_type.name,
                    cost: building_type.cost,
                    next_level_stats: None,
                }
            }
        })
        .collect();

    //fetching available attackers of the user
    let joined_table = available_blocks::table
        .inner_join(attacker_type::table)
        .filter(available_blocks::user_id.eq(user_id))
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
            if attacker_type.level < 3 {
                let next_level = attacker_type.level + 1;
                let next_level_stats = attacker_type::table
                    .filter(attacker_type::level.eq(next_level))
                    .filter(attacker_type::id.eq(attacker_type.id))
                    .first::<AttackerType>(conn)
                    .map_err(|err| DieselError {
                        table: "attacker_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or(AttackerType {
                        id: -1,
                        max_health: -1,
                        speed: -1,
                        amt_of_emps: -1,
                        level: -1,
                        cost: -1,
                    });
                AttackerTypeResponse {
                    id: attacker_type.id,
                    max_health: attacker_type.max_health,
                    speed: attacker_type.speed,
                    amt_of_emps: attacker_type.amt_of_emps,
                    level: attacker_type.level,
                    cost: attacker_type.cost,
                    next_level_stats: Some(NextLevelAttackerTypeResponse {
                        id: next_level_stats.id,
                        max_health: next_level_stats.max_health,
                        speed: next_level_stats.speed,
                        amt_of_emps: next_level_stats.amt_of_emps,
                        level: next_level_stats.level,
                        cost: next_level_stats.cost,
                    }),
                }
            } else {
                //attacker is at max level
                AttackerTypeResponse {
                    id: attacker_type.id,
                    max_health: attacker_type.max_health,
                    speed: attacker_type.speed,
                    amt_of_emps: attacker_type.amt_of_emps,
                    level: attacker_type.level,
                    cost: attacker_type.cost,
                    next_level_stats: None,
                }
            }
        })
        .collect();

    //fetching available defender types of the user
    let joined_table = available_blocks::table
        .inner_join(
            block_type::table
                .inner_join(defender_type::table)
                .inner_join(building_type::table),
        )
        .filter(available_blocks::user_id.eq(user_id))
        .filter(available_blocks::category.eq(ItemCategory::Block))
        .filter(block_type::category.eq(BlockCategory::Defender))
        .select((defender_type::all_columns, building_type::all_columns));
    let defenders: Vec<DefenderTypeResponse> = joined_table
        .load::<(DefenderType, BuildingType)>(conn)
        .map_err(|err| DieselError {
            table: "defender_type",
            function: function!(),
            error: err,
        })?
        .into_iter()
        .map(|(defender_type, building_type)| {
            if building_type.level < 3 {
                let next_level = building_type.level + 1;
                let next_level_stats = block_type::table
                    .inner_join(defender_type::table)
                    .inner_join(building_type::table)
                    .filter(building_type::level.eq(next_level))
                    .filter(defender_type::id.eq(defender_type.id))
                    .select((defender_type::all_columns, building_type::all_columns))
                    .first::<(DefenderType, BuildingType)>(conn)
                    .map_err(|err| DieselError {
                        table: "building_type",
                        function: function!(),
                        error: err,
                    })
                    .unwrap_or((
                        DefenderType {
                            id: -1,
                            speed: -1,
                            damage: -1,
                            radius: -1,
                        },
                        BuildingType {
                            id: -1,
                            name: "".to_string(),
                            width: -1,
                            height: -1,
                            capacity: -1,
                            level: -1,
                            cost: -1,
                            hp: -1,
                        },
                    ));

                let next_level_defender_stats = Some(NextLevelDefenderTypeResponse {
                    id: next_level_stats.0.id,
                    speed: next_level_stats.0.speed,
                    damage: next_level_stats.0.damage,
                    radius: next_level_stats.0.radius,
                    name: next_level_stats.1.name,
                    level: next_level_stats.1.level,
                    cost: next_level_stats.1.cost,
                    hp: next_level_stats.1.hp,
                });

                DefenderTypeResponse {
                    id: defender_type.id,
                    speed: defender_type.speed,
                    damage: defender_type.damage,
                    radius: defender_type.radius,
                    name: building_type.name,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: next_level_defender_stats,
                }
            } else {
                //defender is at max level
                DefenderTypeResponse {
                    id: defender_type.id,
                    speed: defender_type.speed,
                    damage: defender_type.damage,
                    radius: defender_type.radius,
                    name: building_type.name,
                    level: building_type.level,
                    cost: building_type.cost,
                    hp: building_type.hp,
                    next_level_stats: None,
                }
            }
        })
        .collect();

    Ok(GetInventoryResponse {
        buildings,
        mines,
        attackers,
        defenders,
    })
}

#[derive(Serialize, Deserialize)]
pub struct ItemUpgradeResponse {
    pub success: bool,
    pub message: String,
}

pub fn upgrade_item(_user_id: i32, _conn: &mut PgConnection) -> Result<ItemUpgradeResponse> {
    todo!()
}
