use std::collections::HashSet;
// use std::cmp;

use crate::{
    // schema::defender_type::damage,
    // simulation::defense::defender,
    validator::util::{Attacker, Bomb, BuildingDetails, Coordinates, DefenderDetails, MineDetails},
};

use serde::Serialize;

#[derive(Serialize)]
pub struct State {
    pub frame_no: i32,
    pub attacker_user_id: i32,
    pub defender_user_id: i32,
    pub attacker: Option<Attacker>,
    pub attacker_death_count: i32,
    pub bombs: i32,
    pub damage_percentage: f32,
    pub artifacts: i32,
    pub defenders: Vec<DefenderDetails>,
    pub mines: Vec<MineDetails>, //added this
    pub buildings: Vec<BuildingDetails>,
}

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
            bombs: 0,
            damage_percentage: 0.0,
            artifacts: 0,
            defenders,
            mines,
            buildings,
        }
    }

    // Setters for the state

    fn attacker_movement_update(&mut self, attacker_pos: &Coordinates) {
        self.attacker.as_mut().unwrap().attacker_pos.x = attacker_pos.x;
        self.attacker.as_mut().unwrap().attacker_pos.y = attacker_pos.y;
    }

    fn defender_movement_update(&mut self, defender_id: i32, defender_pos: Coordinates) {
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

    // fn attacker_death_update(&mut self) {
    //     self.attacker.as_mut().unwrap().attacker_pos = Coords { x: -1, y: -1 };
    //     self.attacker_death_count += 1;
    // }

    fn defender_death_update(&mut self, defender_id: i32) {
        let attacker = self.attacker.as_mut().unwrap();

        for i in 0..self.defenders.len() {
            if self.defenders[i].id == defender_id {
                attacker.attacker_health -= self.defenders[i].damage;
                self.defenders[i].is_alive = false;
                // break;
            }
        }
    }

    fn mine_blast_update(&mut self, _id: i32, damage_to_attacker: i32) {
        let attacker = self.attacker.as_mut().unwrap();
        if attacker.attacker_health > 0 {
            attacker.attacker_health -= self.defenders[0].damage;
        }
        attacker.attacker_health = std::cmp::max(0, attacker.attacker_health - damage_to_attacker);
        if attacker.attacker_health == 0 {
            self.attacker_death_count += 1;
            attacker.attacker_pos = Coordinates { x: -1, y: -1 };
        }
    }

    // fn bomb_blast_update(&mut self, final_damage_percentage: i32, increase_artifacts: i32) {
    //     self.damage_percentage = final_damage_percentage;
    //     self.artifacts += increase_artifacts;
    // }

    //logic

    pub fn attacker_movement(
        &mut self,
        frame_no: i32,
        attacker_delta: Vec<Coordinates>,
        attacker_current: Attacker,
        defenders_current: Vec<DefenderDetails>,
    ) -> Option<&Self> {
        if (frame_no - self.frame_no) != 1 {
            Some(self) // invalid frame error
        } else {
            self.frame_no += 1;

            let mut attacker = self.attacker.clone().unwrap();

            if attacker.attacker_speed != attacker_current.attacker_speed {
                // invalid event error
                return Some(self);
            }

            let new_pos = attacker.attacker_pos;
            let mut coord_temp: Coordinates = Coordinates {
                x: attacker_delta[0].x,
                y: attacker_delta[0].y,
            };

            for coord in attacker_delta {
                if (coord_temp.x - coord.x > 1)
                    || (coord_temp.y - coord.y > 1)
                    || ((coord_temp.x - coord.x).abs() == 1 && coord_temp.y != coord.y)
                    || ((coord_temp.y - coord.y).abs() == 1 && coord_temp.x != coord.x)
                {
                    // invalid movement error
                    return Some(self);
                }
                coord_temp = coord;
                // if !util::is_road(&coord) {
                //     // tile not road error
                // }
            }

            // let attacker = self.attacker.as_mut().unwrap();
            for defender in defenders_current {
                if defender.is_alive
                    && defender.id != -1
                    && ((defender.defender_pos.x - new_pos.x).pow(2) as f32
                        + (defender.defender_pos.y - new_pos.y).pow(2) as f32)
                        .sqrt()
                        <= defender.radius as f32
                {
                    // defender triggered
                    self.defender_death_update(defender.id);
                }
            }

            if attacker_current.bombs.len() - attacker.bombs.len() > 1 {
                return Some(self);
            }

            if attacker_current.bombs[attacker_current.bombs.len() - 1].is_dropped {
                //dropped a bomb
                //damage buildings

                if attacker.bombs[attacker.bombs.len() - 1].damage
                    != attacker_current.bombs[attacker_current.bombs.len() - 1].damage
                {
                    return Some(self);
                }
                if attacker.bombs[attacker.bombs.len() - 1].blast_radius
                    != attacker_current.bombs[attacker_current.bombs.len() - 1].blast_radius
                {
                    return Some(self);
                }
                self.bomb_blast(attacker_current.bombs[attacker_current.bombs.len() - 1]);
                attacker.bombs.pop();
            }

            self.attacker_movement_update(&new_pos);
            None
        }
    }

    pub fn defender_movement(
        &mut self,
        frame_no: i32,
        defenders: Vec<DefenderDetails>,
    ) -> Option<&Self> {
        if (frame_no - self.frame_no) != 1 {
            Some(self) // invalid frame error
        } else {
            self.frame_no += 1;

            if !defenders.is_empty() {
                for defender in defenders {
                    if defender.speed != self.defenders[0].speed {
                        // invalid speed error
                        return Some(self);
                    }
                    self.defender_movement_update(defender.id, defender.defender_pos);
                }
            }

            // for defender in defenders {
            //     // if !util::is_road(&defender.defender_pos) {
            //     //     // tile not road error
            //     // }
            // }

            None
        }
    }

    pub fn mine_blast(
        &mut self,
        frame_no: i32,
        mut mines: Vec<MineDetails>,
        attacker_pos: Coordinates,
    ) -> Option<&Self> {
        if (frame_no - self.frame_no) != 1 {
            Some(self) // invalid frame error
        } else {
            self.frame_no += 1;
            let damage_to_attacker;
            for (i, mine) in mines.iter_mut().enumerate() {
                if attacker_pos.x == mine.pos.x && attacker_pos.y == mine.pos.y {
                    // triggered
                    if mine.pos.x != self.mines[i].pos.x && mine.pos.y != self.mines[i].pos.y {
                        return Some(self); // return previous state, detected a removal of mine or change in position
                    }
                    damage_to_attacker = mine.damage;
                    self.mine_blast_update(mine.id, damage_to_attacker);

                    break;
                }
            }

            None
        }
    }

    pub fn bomb_blast(&mut self, bomb: Bomb) -> Option<&Self> {
        if bomb.blast_radius != self.attacker.as_ref().unwrap().bombs[0].blast_radius {
            return Some(self);
        }
        if bomb.damage != self.attacker.as_ref().unwrap().bombs[0].damage {
            return Some(self);
        }
        let total_hit_points = 20000;

        // for (_i, building) in self.buildings.iter_mut().enumerate() {
        for building in self.buildings.iter_mut() {
            if building.current_hp != 0 {
                // let damage_buildings = self.calculate_damage_area(building, bomb);
                let building_matrix: HashSet<Coordinates> = (building.tile.y
                    ..building.tile.y + building.width)
                    .flat_map(|y| {
                        (building.tile.x..building.tile.x + building.width)
                            .map(move |x| Coordinates { x, y })
                    })
                    .collect();

                let bomb_matrix: HashSet<Coordinates> = (bomb.pos.y - bomb.blast_radius
                    ..bomb.pos.y + bomb.blast_radius + 1)
                    .flat_map(|y| {
                        (bomb.pos.x - bomb.blast_radius..bomb.pos.x + bomb.blast_radius + 1)
                            .map(move |x| Coordinates { x, y })
                    })
                    .collect();

                let coinciding_coords_damage = building_matrix.intersection(&bomb_matrix).count();

                let damage_buildings: f32 =
                    (coinciding_coords_damage as f32 / building_matrix.len() as f32) * 100.0;

                if damage_buildings != 0.0 {
                    let old_hp = building.current_hp;
                    let mut current_damage =
                        (damage_buildings * building.total_hp as f32 / 100.0) as i32;
                    building.current_hp -=
                        (damage_buildings * building.total_hp as f32 / 100.0) as i32;
                    if building.current_hp <= 0 {
                        building.current_hp = 0;
                        current_damage = old_hp;
                        self.artifacts += building.artifacts_obtained;
                        self.damage_percentage +=
                            (current_damage as f32 / total_hit_points as f32) * 100.0_f32;
                    } else {
                        self.damage_percentage +=
                            (current_damage as f32 / total_hit_points as f32) * 100.0_f32;
                    }
                }
            } else {
                continue;
            }
        }

        // if util::is_road(&bomb.pos) {
        //     // tile not road error
        // }

        None
    }

    pub fn calculate_damage_area(&mut self, building: &mut BuildingDetails, bomb: Bomb) -> i32 {
        // let mut building_matrix: Vec<Coords> = Vec::new();
        // let mut bomb_matrix: Vec<Coords> = Vec::new();

        // building will have top left coordinate and the x and y dimensions
        // for y in building.tile.y..building.tile.y+building.dimensions.y {
        //     for x in building.tile.x..building.tile.x+building.dimensions.x {
        //         building_matrix.push(Coords{x, y});
        //     }
        // }

        // for y in bomb.pos.y-bomb.blast_radius..bomb.pos.y+bomb.blast_radius {
        //     for x in bomb.pos.x-bomb.blast_radius..bomb.pos.x+bomb.blast_radius {
        //         bomb_matrix.push(Coords{x, y});
        //     }
        // }
        // let mut same_coordinates: Vec<Coords> = Vec::new();

        // for building_coord in building_matrix.iter() {
        //     for bomb_coord in bomb_matrix.iter() {
        //         if building_coord == bomb_coord {
        //             same_coordinates.push(*building_coord);
        //         }
        //     }
        // }

        // the below code is a more efficient way to do the same thing as above

        let building_matrix: HashSet<Coordinates> = (building.tile.y
            ..building.tile.y + building.width)
            .flat_map(|y| {
                (building.tile.x..building.tile.x + building.width)
                    .map(move |x| Coordinates { x, y })
            })
            .collect();

        let bomb_matrix: HashSet<Coordinates> = (bomb.pos.y - bomb.blast_radius
            ..bomb.pos.y + bomb.blast_radius + 1)
            .flat_map(|y| {
                (bomb.pos.x - bomb.blast_radius..bomb.pos.x + bomb.blast_radius + 1)
                    .map(move |x| Coordinates { x, y })
            })
            .collect();

        let coinciding_coords_damage = building_matrix.intersection(&bomb_matrix).count();

        let blast_damage_percent =
            (coinciding_coords_damage as f32 / building_matrix.len() as f32) * 100.0;

        blast_damage_percent as i32
    }

    // bomb placement
    // mines
}
