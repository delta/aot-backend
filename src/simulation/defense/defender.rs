use std::collections::HashMap;

use crate::models::*;
use crate::simulation::attack::AttackManager;
use crate::simulation::blocks::*;
use anyhow::{Ok, Result};
use diesel::dsl::not;
use diesel::prelude::*;
use diesel::PgConnection;

#[derive(Clone)]
pub struct Defender {
    pub id: i32,
    pub defender_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub target_id: Option<i32>,
    pub map_id: i32,
    pub defender_path: Vec<(i32, i32)>,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct Defenders(HashMap<i32, Defender>);

impl Defenders {
    #[allow(dead_code)]
    pub fn new(conn: &mut PgConnection) -> Result<Self> {
        use crate::schema::{building_type, defender_type, map_spaces};

        let joined_table = map_spaces::table.inner_join(
            building_type::table
                .inner_join(defender_type::table.on(not(building_type::defender_type.is_null()))),
        );

        let result: Vec<Defender> = joined_table
            .load::<(MapSpaces, (BuildingType, DefenderType))>(conn)?
            .into_iter()
            .map(|(map_space, (_, defender_type))| Defender {
                id: map_space.id,
                defender_type: defender_type.id,
                radius: defender_type.radius,
                x_position: map_space.x_coordinate,
                y_position: map_space.y_coordinate,
                is_alive: true,
                target_id: None,
                speed: defender_type.speed,
                damage: defender_type.damage,
                map_id: map_space.map_id,
                defender_path: Vec::new(),
            })
            .collect();

        let mut defenders: HashMap<i32, Defender> = HashMap::new();

        for defender in result.into_iter() {
            defenders.insert(defender.id, defender);
        }

        Ok(Defenders(defenders))
    }

    pub fn get_defenders_entrance(roads: &[MapSpaces], defender: &Defender) -> Result<(i32, i32)> {
        let mut x = i32::MAX;
        let mut y = i32::MAX;
        for road in roads.iter() {
            let r_x = road.x_coordinate;
            let r_y = road.y_coordinate;

            if (r_x == defender.x_position + 1 || r_x == defender.x_position - 1)
                && (r_y == defender.y_position + 1 || r_y == defender.y_position - 1)
            {
                x = r_x;
                y = r_y;
                break;
            }
        }

        Ok((x, y))
    }

    #[allow(dead_code)]
    pub fn simulate(
        &mut self,
        attack_manager: &mut AttackManager,
        conn: &mut PgConnection,
        map_id: i32,
    ) -> Result<()> {
        let Defenders(defenders) = self;
        let attackers = &mut attack_manager.attackers;
        let roads = BuildingsManager::get_road_map_spaces(conn, map_id)?;
        let shortest_paths = BuildingsManager::get_shortest_paths(conn, map_id)?;

        let mut attacker_damage: HashMap<usize, i32> = HashMap::new();

        for (_, (attacker_id, attacker)) in attackers.iter().enumerate() {
            let (attacker_x, attacker_y) = attacker.get_current_position()?;

            let mut newly_assigned = false;
            let mut change_in_pos = false;
            let mut target_defender = None;
            let mut opt_dist = f32::MAX;
            let mut opt_damage = i32::MIN;

            for (_, defender) in defenders.iter_mut() {
                if change_in_pos {
                    break;
                }

                match defender.target_id {
                    Some(target_id) => {
                        if target_id == *attacker_id {
                            change_in_pos = true;
                            target_defender = Some(defender.id);
                        }
                    }
                    None => {
                        let radius = (((attacker_x - defender.x_position).pow(2)
                            + (attacker_y - defender.y_position).pow(2))
                            as f32)
                            .sqrt();

                        if radius <= defender.radius as f32 {
                            newly_assigned = true;
                            if radius < opt_dist {
                                opt_dist = radius;
                                opt_damage = defender.damage;
                                target_defender = Some(defender.id);
                            } else if (opt_dist == radius) && (opt_damage < defender.damage) {
                                opt_damage = defender.damage;
                                target_defender = Some(defender.id);
                            }
                        }
                    }
                }
            }

            let updated_defender = defenders.get_mut(&target_defender.unwrap()).unwrap();

            let target_attacker = attackers.get(&updated_defender.target_id.unwrap()).unwrap();
            if change_in_pos {
                if target_attacker.is_alive {
                    let (prev_att_x, prev_att_y) = updated_defender.defender_path[0];

                    let mut extra_path = shortest_paths
                        .get(&SourceDest {
                            source_x: prev_att_x,
                            source_y: prev_att_y,
                            dest_x: attacker_x,
                            dest_y: attacker_y,
                        })
                        .unwrap()
                        .clone();

                    let mut i = 0;

                    while i < extra_path.len() && i < (updated_defender.defender_path.len() - 1) {
                        let (r_x, r_y) = extra_path[i];
                        let (e_x, e_y) = updated_defender.defender_path[i];

                        if r_x != e_x || r_y != e_y {
                            break;
                        }

                        i += 1;
                    }

                    if i == 1 {
                        extra_path.reverse();
                        extra_path.append(&mut updated_defender.defender_path);
                        updated_defender.defender_path = extra_path;
                    } else {
                        updated_defender.defender_path =
                            updated_defender.defender_path[i..].to_vec();

                        let (d_x, d_y) = updated_defender.defender_path[0];
                        extra_path = shortest_paths
                            .get(&SourceDest {
                                source_x: d_x,
                                source_y: d_y,
                                dest_x: attacker_x,
                                dest_y: attacker_y,
                            })
                            .unwrap()
                            .clone();

                        extra_path.reverse();
                        extra_path.pop();
                        extra_path.append(&mut updated_defender.defender_path);
                        updated_defender.defender_path = extra_path;
                    }

                    (updated_defender.x_position, updated_defender.y_position) =
                        updated_defender.defender_path[updated_defender.defender_path.len() - 1];
                    updated_defender.defender_path.pop();

                    if (updated_defender.x_position == attacker_x)
                        && (updated_defender.y_position == attacker_y)
                    {
                        attacker_damage.insert(*attacker_id as usize, updated_defender.damage);
                    }
                } else {
                    //special case
                    (updated_defender.x_position, updated_defender.y_position) =
                        updated_defender.defender_path[0];
                    updated_defender.defender_path = updated_defender.defender_path[1..].to_vec();
                }
            } else if newly_assigned && target_attacker.is_alive {
                updated_defender.target_id = Some(*attacker_id);
                let (init_x, init_y) = Defenders::get_defenders_entrance(&roads, updated_defender)?;

                updated_defender.x_position = init_x;
                updated_defender.y_position = init_y;

                let mut extra_path = shortest_paths
                    .get(
                        &(SourceDest {
                            source_x: updated_defender.x_position,
                            source_y: updated_defender.y_position,
                            dest_x: attacker_x,
                            dest_y: attacker_y,
                        }),
                    )
                    .unwrap()
                    .clone();

                extra_path.reverse();

                updated_defender.defender_path = extra_path;
            }
        }

        for (att_id, (_, damage)) in attacker_damage.iter().enumerate() {
            let att = attackers.get_mut(&(att_id as i32)).unwrap();
            att.get_damage(*damage);
        }

        Ok(())
    }
}
