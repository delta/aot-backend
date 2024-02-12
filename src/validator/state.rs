use std::collections::{HashMap, HashSet};
// use std::cmp;

use crate::{
    // schema::defender_type::damage,
    // simulation::defense::defender,
    api::attack, schema::shortest_path, simulation::defense::defender, validator::util::{Attacker, Bomb, BuildingDetails, Coordinates, DefenderDetails, MineDetails, SourceDest}
};

use rayon::iter;
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
    pub mines: Vec<MineDetails>, 
    pub buildings: Vec<BuildingDetails>,
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
            bombs: 0,
            damage_percentage: 0.0,
            artifacts: 0,
            defenders,
            mines,
            buildings,
        }
    }

    // Setters for the state

    pub fn place_attacker(&mut self, attacker: Attacker) {
        self.attacker = Some(attacker);
    }

    pub fn attacker_movement_update(&mut self, attacker_pos: &Coordinates) {
        self.attacker.as_mut().unwrap().attacker_pos.x = attacker_pos.x;
        self.attacker.as_mut().unwrap().attacker_pos.y = attacker_pos.y;
    }

    pub fn defender_movement_update(&mut self, defender_id: i32, defender_pos: Coordinates) {
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

    //  pub fn attacker_death_update(&mut self) {
    //     self.attacker.as_mut().unwrap().attacker_pos = Coords { x: -1, y: -1 };
    //     self.attacker_death_count += 1;
    // }

    pub fn defender_death_update(&mut self, defender_id: i32) {
        let attacker = self.attacker.as_mut().unwrap();
        

        for i in 0..self.defenders.len() {
            if self.defenders[i].id == defender_id {
                attacker.attacker_health -= self.defenders[i].damage;
                if(attacker.attacker_health <= 0){
                    attacker.attacker_pos = Coordinates { x: -1, y: -1 };
                    self.attacker_death_count += 1;
                }
                self.defenders[i].is_alive = false;
                // break;
            }
        }
    }

    pub fn mine_blast_update(&mut self, _id: i32, damage_to_attacker: i32) {
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
        // defenders_current: Vec<DefenderDetails>,
    ) -> Option<Attacker> {
        if (frame_no - self.frame_no) != 1 {
            // Some(self) // invalid frame error
            None
            // GAME_OVER
        } else {
            self.frame_no += 1;

            let attacker = self.attacker.clone().unwrap();

            if attacker.attacker_speed != attacker_current.attacker_speed {
                // invalid event error
                            // GAME_OVER

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
                               // GAME_OVER

                }

                for defender in self.defenders.iter_mut() {
                        if defender.is_alive
                            && defender.id != -1
                            && ((defender.defender_pos.x - new_pos.x).pow(2) as f32
                                + (defender.defender_pos.y - new_pos.y).pow(2) as f32)
                                .sqrt()
                                <= defender.radius as f32
                        {
                            defender.target_id = Some(attacker.id);
                        }
                    }

            coord_temp = coord;
            
                // if !util::is_road(&coord) {
                //     // tile not road error
                // }
            }

            // let attacker = self.attacker.as_mut().unwrap();
            // for defender in defenders_current {
            //     if defender.is_alive
            //         && defender.id != -1
            //         && ((defender.defender_pos.x - new_pos.x).pow(2) as f32
            //             + (defender.defender_pos.y - new_pos.y).pow(2) as f32)
            //             .sqrt()
            //             <= defender.radius as f32
            //     {
            //         // defender triggered
            //         // self.defender_death_update(defender.id);
            //     }
            // }

            
            // if attacker_current.bombs.len() - attacker.bombs.len() > 1 {
            //     return Some(self);
            // }

            // if attacker_current.bombs[attacker_current.bombs.len() - 1].is_dropped {
            //     //dropped a bomb
            //     //damage buildings

            //     if attacker.bombs[attacker.bombs.len() - 1].damage
            //         != attacker_current.bombs[attacker_current.bombs.len() - 1].damage
            //     {
            //         return Some(self);
            //     }
            //     if attacker.bombs[attacker.bombs.len() - 1].blast_radius
            //         != attacker_current.bombs[attacker_current.bombs.len() - 1].blast_radius
            //     {
            //         return Some(self);
            //     }
            //     self.bomb_blast(attacker_current.bombs[attacker_current.bombs.len() - 1]);
            //     attacker.bombs.pop();
            // }

            self.attacker_movement_update(&new_pos);
            Some(attacker_current)
        }
    }

    pub fn place_bombs(&mut self, attacker_current: Attacker) -> Option<&Self> {
        let mut attacker = self.attacker.clone().unwrap();


        // if attacker_current.bombs.len() - attacker.bombs.len() > 1 {
        //                 

        // }
        if attacker_current.bombs[attacker_current.bombs.len() - 1].is_dropped {
            //dropped a bomb
            //damage buildings

            if attacker.bombs[attacker.bombs.len() - 1].damage
                != attacker_current.bombs[attacker_current.bombs.len() - 1].damage
            {
                            // GAME_OVER

            }
            if attacker.bombs[attacker.bombs.len() - 1].blast_radius
                != attacker_current.bombs[attacker_current.bombs.len() - 1].blast_radius
            {
                           // GAME_OVER

            }
            self.bomb_blast(attacker_current.bombs[attacker_current.bombs.len() - 1]);
            attacker.bombs.pop();
        }

        return Some(self)
    }


    pub fn is_coord_crosssed(defender_pos_prev: Coordinates,defender_pos: Coordinates, attacker_pos_prev: Coordinates, attacker_pos: Coordinates) -> bool {
        if defender_pos_prev.x == attacker_pos.x && defender_pos_prev.y == attacker_pos.y || defender_pos.x == attacker_pos_prev.x && defender_pos.y == attacker_pos_prev.y {
            return true;
        }
        return false;
    }

    #[warn(unused_assignments)]

    pub fn defender_movement(
        &mut self,
        frame_no: i32,
        attacker_delta: Vec<Coordinates>,
        shortest_path: &HashMap<(Coordinates,Coordinates),Coordinates>,
    ) -> Option<(Attacker, &Vec<DefenderDetails>)> {
        if (frame_no - self.frame_no) != 1 {
            None
            // GAME_OVER
        } else {
            self.frame_no += 1;

            // if !defenders.is_empty() {
            //     for defender in defenders {
            //         if defender.damage != self.defenders[0].damage || defender.speed != self.defenders[0].speed || defender.radius != self.defenders[0].radius{
            //             // invalid speed error
            //             return Some(self);
            //         }
            //        if defender.is_alive {
            //         self.defender_movement_update(defender.id, defender.defender_pos);

            //        }
            //     }
            // }
            let mut attacker = self.attacker.clone().unwrap();

            let ratio: f32 = attacker.attacker_speed as f32 / self.defenders[0].speed as f32;
            attacker.attacker_pos = attacker_delta[0];
            let mut attacker_prev = attacker_delta[0];

            for defender in self.defenders.iter_mut(){

                if defender.target_id != None && defender.is_alive {
                    if defender.path_in_current_frame.len() > 0 {
                        defender.defender_pos = defender.path_in_current_frame[defender.path_in_current_frame.len()-1]; // this is the current position of the defender
                    }
                    let mut defender_prev;
                    defender.path_in_current_frame.clear();
                    let mut attacker_pointer_coord: i32 = -1; // i kept this is as the reference to the attacker's position in the attacker_delta vector. initially it is -1;
                    let mut position_float = 0.5 as f32; 
                    for iterator in 0..defender.speed {

                        if iterator == 0 {
                            defender_prev = defender.defender_pos;
                        }
                        else {
                            defender_prev = defender.path_in_current_frame[iterator as usize - 1];
                        }
                        let next_hop = shortest_path.get(&(&(defender.defender_pos,attacker.attacker_pos))).unwrap();

                        defender.defender_pos = *next_hop;
                        position_float += ratio as f32;

                        if iterator < (attacker_delta.len() as usize).try_into().unwrap() && position_float > 1.0  {
                            if attacker_pointer_coord != -1 {
                                attacker_prev = attacker_delta[attacker_pointer_coord as usize];
                            }
                            attacker_pointer_coord += 1;
                            attacker.attacker_pos = attacker_delta[attacker_pointer_coord as usize];
                            if attacker_delta[attacker_pointer_coord as usize] == defender.defender_pos || Self::is_coord_crosssed(defender_prev,defender.defender_pos,attacker_prev,attacker.attacker_pos) {
                                // defender sucided
                                attacker.attacker_health -= defender.damage;
                                defender.is_alive = false;
                                if attacker.attacker_health <= 0 {
                                    attacker.attacker_pos = Coordinates { x: -1, y: -1 };
                                    self.attacker_death_count += 1;
                                }
                            }
                            position_float -= 1 as f32;
                        }
                     
                        defender.path_in_current_frame.push(*next_hop);
                    }
                }
            }

            // for defender in defenders {
            //     // if !util::is_road(&defender.defender_pos) {
            //     //     // tile not road error
            //     // }
            // }

            Some((attacker, &self.defenders))
        }
    }

    pub fn mine_blast(
        &mut self,
        frame_no: i32,
        mut mines: Vec<MineDetails>,
        attacker_delta:Vec<Coordinates>,
        
    ) -> Option<&Attacker> {
        if (frame_no - self.frame_no) != 1 {
            //GAME_OVER
            None
        } else {
            self.frame_no += 1;
            let mut damage_to_attacker;

            for (_i, mine) in mines.iter_mut().enumerate() {
                for attacker_pos in attacker_delta.iter() { 
                    if attacker_pos.x == mine.pos.x && attacker_pos.y == mine.pos.y {
                        damage_to_attacker = mine.damage;
                        self.mine_blast_update(mine.id, damage_to_attacker);
                    }
                }  
            }

            self.attacker.as_ref()
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
                    (coinciding_coords_damage as f32 / building_matrix.len() as f32);

                if damage_buildings != 0.0 {
                    let old_hp = building.current_hp;
                    let mut current_damage =
                        (damage_buildings * building.total_hp as f32) as i32;
                    building.current_hp -=
                        (damage_buildings * building.total_hp as f32) as i32;
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

    // pub fn calculate_damage_area(&mut self, building: &mut BuildingDetails, bomb: Bomb) -> i32 {
    //     // let mut building_matrix: Vec<Coords> = Vec::new();
    //     // let mut bomb_matrix: Vec<Coords> = Vec::new();

    //     // building will have top left coordinate and the x and y dimensions
    //     // for y in building.tile.y..building.tile.y+building.dimensions.y {
    //     //     for x in building.tile.x..building.tile.x+building.dimensions.x {
    //     //         building_matrix.push(Coords{x, y});
    //     //     }
    //     // }

    //     // for y in bomb.pos.y-bomb.blast_radius..bomb.pos.y+bomb.blast_radius {
    //     //     for x in bomb.pos.x-bomb.blast_radius..bomb.pos.x+bomb.blast_radius {
    //     //         bomb_matrix.push(Coords{x, y});
    //     //     }
    //     // }
    //     // let mut same_coordinates: Vec<Coords> = Vec::new();

    //     // for building_coord in building_matrix.iter() {
    //     //     for bomb_coord in bomb_matrix.iter() {
    //     //         if building_coord == bomb_coord {
    //     //             same_coordinates.push(*building_coord);
    //     //         }
    //     //     }
    //     // }

    //     // the below code is a more efficient way to do the same thing as above

    //     let building_matrix: HashSet<Coordinates> = (building.tile.y
    //         ..building.tile.y + building.width)
    //         .flat_map(|y| {
    //             (building.tile.x..building.tile.x + building.width)
    //                 .map(move |x| Coordinates { x, y })
    //         })
    //         .collect();

    //     let bomb_matrix: HashSet<Coordinates> = (bomb.pos.y - bomb.blast_radius
    //         ..bomb.pos.y + bomb.blast_radius + 1)
    //         .flat_map(|y| {
    //             (bomb.pos.x - bomb.blast_radius..bomb.pos.x + bomb.blast_radius + 1)
    //                 .map(move |x| Coordinates { x, y })
    //         })
    //         .collect();

    //     let coinciding_coords_damage = building_matrix.intersection(&bomb_matrix).count();

    //     let blast_damage_percent =
    //         (coinciding_coords_damage as f32 / building_matrix.len() as f32) * 100.0;

    //     blast_damage_percent as i32
    // }

    // bomb placement
    // mines
}
