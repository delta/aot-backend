use std::{
    cmp::max,
    collections::{HashMap, HashSet},
};
// use std::cmp;

use crate::{
    api::attack::socket::{BuildingResponse, DefenderResponse},
    validator::util::{
        Attacker, BuildingDetails, DefenderDetails, DefenderReturnType, MineDetails,
    },
};
use crate::{
    constants::{BOMB_DAMAGE_MULTIPLIER, PERCENTANGE_ARTIFACTS_OBTAINABLE},
    simulation::blocks::{Coords, SourceDest},
};

use serde::{Deserialize, Serialize};

use super::util::BombType;

#[derive(Serialize, Deserialize, Clone)]
pub struct State {
    pub frame_no: i32,
    pub attacker_user_id: i32,
    pub defender_user_id: i32,
    pub attacker: Option<Attacker>,
    pub attacker_death_count: i32,
    pub bombs: BombType,
    pub damage_percentage: f32,
    pub artifacts: i32,
    pub defenders: Vec<DefenderDetails>,
    pub mines: Vec<MineDetails>,
    pub buildings: Vec<BuildingDetails>,
    pub total_hp_buildings: i32,
}

#[allow(dead_code)]
impl State {
    pub fn new(
        attacker_user_id: i32,
        defender_user_id: i32,
        defenders: Vec<DefenderDetails>,
        mines: Vec<MineDetails>,
        buildings: Vec<BuildingDetails>,
    ) -> State {
        State {
            frame_no: 0,
            attacker_user_id,
            defender_user_id,
            attacker: None,
            attacker_death_count: 0,
            bombs: BombType {
                id: -1,
                radius: 0,
                damage: 0,
                total_count: 0,
            },
            damage_percentage: 0.0,
            artifacts: 0,
            defenders,
            mines,
            buildings,
            total_hp_buildings: 0,
        }
    }

    pub fn set_total_hp_buildings(&mut self) {
        let mut total_hp = 0;
        for building in self.buildings.iter() {
            total_hp += building.total_hp;
        }
        self.total_hp_buildings = total_hp;
    }

    pub fn set_bombs(&mut self, bomb_type: BombType, bombs: i32) {
        let bomb_count: i32;
        if self.bombs.total_count != 0 {
            bomb_count = self.bombs.total_count;
        } else {
            bomb_count = bombs;
        }

        self.bombs = BombType {
            id: bomb_type.id,
            radius: bomb_type.radius,
            damage: bomb_type.damage,
            total_count: bomb_count,
        };
    }

    pub fn place_attacker(&mut self, attacker: Attacker) {
        self.attacker = Some(attacker);
    }

    pub fn attacker_movement_update(&mut self, attacker_pos: &Coords) {
        self.attacker.as_mut().unwrap().attacker_pos.x = attacker_pos.x;
        self.attacker.as_mut().unwrap().attacker_pos.y = attacker_pos.y;
    }

    pub fn defender_movement_update(&mut self, defender_id: i32, defender_pos: Coords) {
        let attacker = self.attacker.as_mut().unwrap();

        if attacker.attacker_health > 0 {
            attacker.attacker_health -= self.defenders[0].damage;
        }

        attacker.attacker_health -= self.defenders[0].damage;
        for i in 0..self.defenders.len() {
            if self.defenders[i].id == defender_id {
                if defender_pos.x == attacker.attacker_pos.x
                    && defender_pos.y == attacker.attacker_pos.y
                {
                    attacker.attacker_health -= self.defenders[i].damage;
                }
                self.defenders[i].defender_pos = defender_pos;
                break;
            }
        }
    }

    pub fn mine_blast_update(&mut self, _id: i32, damage_to_attacker: i32) {
        let attacker = self.attacker.as_mut().unwrap();

        if attacker.attacker_health > 0 {
            attacker.attacker_health =
                std::cmp::max(0, attacker.attacker_health - damage_to_attacker);
            if attacker.attacker_health == 0 {
                println!(
                    "attacker died due to mine blast and mine damage : {}",
                    damage_to_attacker
                );
                self.attacker_death_count += 1;
                for defender in self.defenders.iter_mut() {
                    defender.target_id = None;
                }
                attacker.attacker_pos = Coords { x: -1, y: -1 };
            }
        }

        self.mines.retain(|mine| mine.id != _id);
    }

    pub fn update_frame_number(&mut self, frame_no: i32) {
        self.frame_no = frame_no;
    }

    pub fn attacker_movement(
        &mut self,
        frame_no: i32,
        roads: &HashSet<(i32, i32)>,
        attacker_current: Attacker,
    ) -> Option<Attacker> {
        // if (frame_no - self.frame_no) != 1 {
        //     // Some(self) // invalid frame error
        //     None
        //     // GAME_OVER
        // } else {
        // self.frame_no += 1;

        println!("state frame: {} current frame: {}", self.frame_no, frame_no);

        for coord in attacker_current.path_in_current_frame.clone().into_iter() {
            if !roads.contains(&(coord.x, coord.y)) {
                // GAME_OVER

                println!("attacker out of road at {} frame", frame_no);
            }
        }

        let mut attacker = attacker_current.clone();

        if attacker.attacker_speed + 1 != attacker.path_in_current_frame.len() as i32 {
            println!(
                "attacker speed abuse at {} frame --- speed  :{}, length: {}",
                frame_no,
                attacker.attacker_speed,
                attacker.path_in_current_frame.len()
            );
        }

        let mut coord_temp: Coords = Coords {
            x: attacker_current.path_in_current_frame[0].x,
            y: attacker_current.path_in_current_frame[0].y,
        };

        for (i, coord) in attacker_current
            .path_in_current_frame
            .into_iter()
            .enumerate()
        {
            if (coord_temp.x - coord.x > 1)
                || (coord_temp.y - coord.y > 1)
                || ((coord_temp.x - coord.x).abs() == 1 && coord_temp.y != coord.y)
                || ((coord_temp.y - coord.y).abs() == 1 && coord_temp.x != coord.x)
            {
                // GAME_OVER
                println!("attacker skipped a tile at {} frame", frame_no);
            }

            let new_pos = coord;

            for defender in self.defenders.iter_mut() {
                if defender.target_id.is_none()
                    && defender.is_alive
                    && (((defender.defender_pos.x - new_pos.x).abs()
                        + (defender.defender_pos.y - new_pos.y).abs())
                        <= defender.radius)
                {
                    println!(
                        "defender triggered when attacker was at ---- x:{}, y:{} and defender id: {}",
                        new_pos.x, new_pos.y, defender.id
                    );
                    defender.target_id = Some((i) as f32 / attacker.attacker_speed as f32);
                    attacker.trigger_defender = true;
                }
            }

            coord_temp = coord;
        }

        self.frame_no += 1;

        let attacker_result = Attacker {
            id: attacker.id,
            attacker_pos: attacker.path_in_current_frame.last().unwrap().clone(),
            attacker_health: attacker.attacker_health,
            attacker_speed: attacker.attacker_speed,
            path_in_current_frame: attacker.path_in_current_frame.clone(),
            bombs: attacker.bombs.clone(),
            trigger_defender: attacker.trigger_defender,
        };
        Some(attacker_result)
    }

    pub fn place_bombs(
        &mut self,
        attacker_delta: Vec<Coords>,
        bomb_position: Coords,
    ) -> Vec<BuildingResponse> {
        // if attacker_current.bombs.len() - attacker.bombs.len() > 1 {

        // }

        if self.bombs.total_count <= 0 {
            //Nothing
            println!("Bomb over");
        }

        if !attacker_delta.contains(&bomb_position) {
            //GAME_OVER
            println!("Bomb placed out of path");
        }

        let buildings_damaged = self.bomb_blast(bomb_position);

        buildings_damaged
    }

    pub fn is_coord_crosssed(
        defender_pos_prev: Coords,
        defender_pos: Coords,
        attacker_pos_prev: Coords,
        attacker_pos: Coords,
    ) -> bool {
        if defender_pos_prev.x == attacker_pos.x && defender_pos_prev.y == attacker_pos.y
            || defender_pos.x == attacker_pos_prev.x && defender_pos.y == attacker_pos_prev.y
        {
            return true;
        }
        return false;
    }

    pub fn defender_movement(
        &mut self,
        attacker_delta: Vec<Coords>,
        shortest_path: &HashMap<SourceDest, Coords>,
    ) -> DefenderReturnType {
        // self.frame_no += 1;

        let attacker = self.attacker.as_mut().unwrap();
        let mut defenders_triggered: Vec<DefenderResponse> = Vec::new();

        // if attacker is dead, no need to move the defenders
        if attacker.attacker_health == 0 {
            return DefenderReturnType {
                attacker_health: attacker.attacker_health,
                defender_response: defenders_triggered,
                state: self.clone(),
            };
        }

        let mut collision_array: Vec<(i32, f32)> = Vec::new();
        println!("attacker delta: {:?}", attacker_delta);

        for defender in self.defenders.iter_mut() {
            println!("checking defender id: {}", defender.id);
            if !defender.is_alive || defender.target_id.is_none() {
                continue;
            }

            println!(
                "defender id is triggered: {}, defender position: {:?}",
                defender.id, defender.defender_pos
            );

            let attacker_ratio = attacker.attacker_speed as f32 / defender.speed as f32;
            println!(
                "attacker_ratio: {}; attack speed: {}; defender speed: {}",
                attacker_ratio, attacker.attacker_speed, defender.speed
            );
            let mut attacker_float_coords = (
                attacker.attacker_pos.x as f32,
                attacker.attacker_pos.y as f32,
            );
            let mut attacker_delta_index = 1;

            defender.path_in_current_frame.clear();
            defender.path_in_current_frame.push(defender.defender_pos);

            // for every tile of defender's movement
            for i in 1..=defender.speed {
                // move dwon if needed

                let next_hop = shortest_path
                    .get(&SourceDest {
                        source_x: defender.defender_pos.x,
                        source_y: defender.defender_pos.y,
                        dest_x: attacker.attacker_pos.x,
                        dest_y: attacker.attacker_pos.y,
                    })
                    .unwrap_or(&defender.defender_pos);

                let mut attacker_tiles_covered_fract = (((i - 1) as f32) * attacker_ratio).fract();

                let mut attacker_mov_x = 0.0;
                let mut attacker_mov_y = 0.0;

                let mut attacker_tiles_left = attacker_ratio;
                print!("i: {i}; attacker_tiles_left: {}; ", attacker_tiles_left);
                while attacker_tiles_left > 1e-6 {
                    let attacker_tiles_fract_left = attacker_tiles_left
                        .min(1.0)
                        .min(1.0 - attacker_tiles_covered_fract);
                    // print!("attacker_tiles_fract_left: {}; ", attacker_tiles_fract_left);

                    attacker_mov_x += attacker_tiles_fract_left
                        * ((attacker_delta[attacker_delta_index].x
                            - attacker_delta[attacker_delta_index - 1].x)
                            as f32);
                    attacker_mov_y += attacker_tiles_fract_left
                        * ((attacker_delta[attacker_delta_index].y
                            - attacker_delta[attacker_delta_index - 1].y)
                            as f32);

                    // print!(
                    //     "attacker_mov_x: {}; attacker_mov_y: {}; ",
                    //     attacker_mov_x, attacker_mov_y
                    // );

                    attacker_tiles_left -= attacker_tiles_fract_left;
                    attacker_tiles_covered_fract =
                        (attacker_tiles_covered_fract + attacker_tiles_fract_left).fract();
                    if attacker_tiles_covered_fract == 0.0 {
                        attacker_delta_index += 1;
                    }
                    // println!("attacker_tiles_left: {}; attacker_tiles_covered_fract: {}; attacker_delta_index: {}", attacker_tiles_left, attacker_tiles_covered_fract, attacker_delta_index);
                }
                println!(
                    "attacker_mov_x: {}; attacker_mov_y: {}",
                    attacker_mov_x, attacker_mov_y
                );

                attacker_float_coords.0 += attacker_mov_x;
                attacker_float_coords.1 += attacker_mov_y;
                println!("attacker_fract_pos: {:?}", attacker_float_coords);

                attacker.attacker_pos = Coords {
                    x: attacker_float_coords.0.round() as i32,
                    y: attacker_float_coords.1.round() as i32,
                };

                // if defender lags
                if defender.target_id.unwrap() >= ((i as f32) / (defender.speed as f32)) {
                    defender.path_in_current_frame.push(defender.defender_pos);
                    continue;
                }
                defender.defender_pos = *next_hop;
                defender.path_in_current_frame.push(defender.defender_pos);

                println!(
                    "attacker pos: {:?}; defender_position: {:?}",
                    attacker.attacker_pos, defender.defender_pos
                );

                // if defender and attacker are on the same tile, add the defender to the collision_array
                if (defender.defender_pos == attacker.attacker_pos)
                    || (defender.path_in_current_frame[(i - 1) as usize] == attacker.attacker_pos)
                {
                    collision_array.push((defender.id, (i as f32) / (defender.speed as f32)));
                    defender.damage_dealt = true;
                    break;
                }
                println!(
                    "defender id: {}; path: {}",
                    defender.id,
                    defender.path_in_current_frame.len()
                );
            }
            defender.target_id = Some(0.0);
            if !defender.damage_dealt {
                collision_array.push((defender.id, 2.0));
            }
            attacker.attacker_pos = *attacker_delta.first().unwrap();
            println!("done checking defender id: {}", defender.id);
        }

        attacker.attacker_pos = *attacker_delta.last().unwrap();
        // sort the collision_array by the time of collision
        collision_array.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        if !collision_array.is_empty() {
            println!("collision array {:?}", collision_array);
        }
        let mut attacker_death_time = 0.0; // frame fraction at which attacker dies
        for (id, time) in collision_array {
            let defender = self
                .defenders
                .iter_mut()
                .find(|defender| defender.id == id)
                .unwrap();
            defender.target_id = None;
            if time > 1.0 {
                break;
            }

            println!("defender id: {}, time: {}", id, time);
            println!(
                "defender path length: {}",
                defender.path_in_current_frame.len()
            );
            if attacker.attacker_health == 0 {
                println!("attacker died");
                self.attacker_death_count += 1;
                defender.defender_pos = defender.path_in_current_frame
                    [1 + (attacker_death_time * (defender.speed as f32)) as usize];
                continue;
            }
            println!(
                "defender {} hit attacker when defender was at ---- x:{}, y:{}",
                defender.id, defender.defender_pos.x, defender.defender_pos.y
            );
            defenders_triggered.push(DefenderResponse {
                id: defender.id,
                position: defender.defender_pos,
                damage: defender.damage,
            });
            defender.damage_dealt = true;
            attacker.trigger_defender = true;
            attacker.attacker_health = max(0, attacker.attacker_health - defender.damage);
            defender.is_alive = false;

            if attacker.attacker_health == 0 {
                attacker_death_time = time;
                self.attacker_death_count += 1;
                println!("attacker died");
            }
        }

        DefenderReturnType {
            attacker_health: attacker.attacker_health,
            defender_response: defenders_triggered,
            state: self.clone(),
        }
    }

    pub fn mine_blast(&mut self, start_pos: Option<Coords>) -> Vec<MineDetails> {
        // if (frame_no - self.frame_no) != 1 {
        //     //GAME_OVER
        //     None
        // } else {
        let mut damage_to_attacker;
        let attack_current_pos = start_pos.unwrap();

        let mut triggered_mines: Vec<MineDetails> = Vec::new();

        for (_i, mine) in self.mines.clone().iter_mut().enumerate() {
            if attack_current_pos.x == mine.position.x && attack_current_pos.y == mine.position.y {
                damage_to_attacker = mine.damage;
                triggered_mines.push(MineDetails {
                    id: mine.id,
                    position: mine.position,
                    radius: mine.radius,
                    damage: mine.damage,
                });
                self.mine_blast_update(mine.id, damage_to_attacker);
            }
        }

        triggered_mines
        // }
    }

    pub fn bomb_blast(&mut self, bomb_position: Coords) -> Vec<BuildingResponse> {
        // if bomb.blast_radius != self.attacker.as_ref().unwrap().bombs[0].blast_radius {
        //     return Some(self);
        // }
        // if bomb.damage != self.attacker.as_ref().unwrap().bombs[0].damage {
        //     return Some(self);
        // }

        let bomb = &mut self.bombs;
        // println!("bomb blast damage: {}", bomb.damage);
        let mut buildings_damaged: Vec<BuildingResponse> = Vec::new();
        for building in self.buildings.iter_mut() {
            if building.current_hp != 0 {
                let mut artifacts_taken_by_destroying_building: i32 = 0;

                // let damage_buildings = self.calculate_damage_area(building, bomb);
                let building_matrix: HashSet<Coords> = (building.tile.y
                    ..building.tile.y + building.width)
                    .flat_map(|y| {
                        (building.tile.x..building.tile.x + building.width)
                            .map(move |x| Coords { x, y })
                    })
                    .collect();

                let bomb_matrix: HashSet<Coords> = (bomb_position.y - bomb.radius
                    ..bomb_position.y + bomb.radius + 1)
                    .flat_map(|y| {
                        (bomb_position.x - bomb.radius..bomb_position.x + bomb.radius + 1)
                            .map(move |x| Coords { x, y })
                    })
                    .collect();

                let coinciding_coords_damage = building_matrix.intersection(&bomb_matrix).count();

                let damage_buildings: f32 =
                    coinciding_coords_damage as f32 / building_matrix.len() as f32;

                if damage_buildings != 0.0 {
                    // println!("damage building: {}, bomb damage: {}, total_hp:{}",damage_buildings,bomb.damage,building.total_hp);

                    let old_hp = building.current_hp;
                    let mut current_damage = (damage_buildings
                        * (bomb.damage as f32 * BOMB_DAMAGE_MULTIPLIER))
                        .round() as i32;
                    // println!("current damage: {}, current_hp: {}",current_damage,building.current_hp - current_damage);

                    building.current_hp -= current_damage;

                    if building.current_hp <= 0 {
                        building.current_hp = 0;
                        current_damage = old_hp;
                        artifacts_taken_by_destroying_building =
                            (building.artifacts_obtained as f32 * PERCENTANGE_ARTIFACTS_OBTAINABLE)
                                .floor() as i32;
                        self.artifacts += artifacts_taken_by_destroying_building;
                        self.damage_percentage +=
                            (current_damage as f32 / self.total_hp_buildings as f32) * 100.0_f32;
                    } else {
                        self.damage_percentage +=
                            (current_damage as f32 / self.total_hp_buildings as f32) * 100.0_f32;
                    }

                    buildings_damaged.push(BuildingResponse {
                        id: building.id,
                        position: building.tile,
                        hp: building.current_hp,
                        artifacts_if_damaged: artifacts_taken_by_destroying_building,
                    });
                }
            } else {
                continue;
            }
        }

        self.bombs.total_count -= 1;

        buildings_damaged
    }
}
