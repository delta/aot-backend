use serde::Serialize;
use crate::validator::util::{Attacker, Defender, Coords};

#[derive(Serialize)]
pub struct State {
    pub frame_no: i32,
    pub attacker_user_id: i32,
    pub defender_user_id: i32,
    pub attacker: Option<Attacker>,
    pub attacker_death_count: i32,
    pub bombs_left: i32,
    pub damage_percentage: f32,
    pub artifacts: i32,
    pub defenders: Vec<Defender>
}

// impl constructor, and other necessary functions skeleton

impl State {
    pub fn new(
        attacker_user_id: i32,
        defender_user_id: i32,
    ) -> State {
        State {
            frame_no: 0,
            attacker_user_id: attacker_user_id,
            defender_user_id: defender_user_id,
            attacker: None,
            attacker_death_count: 0,
            bombs_left: 0,
            damage_percentage: 0.0,
            artifacts: 0,
            defenders: Vec::new(),
        }
    }

    // Setters for the state

    fn attacker_movement_update(&mut self, attacker_pos: Coords) {
        self.attacker_pos = attacker_pos;
    }

    fn defender_movement_update(&mut self, defender_id: i32, defender_pos: Coords) {
        for i in 0..self.defenders.len() {
            if self.defenders[i].0 == defender_id {
                self.defenders[i].2 = defender_pos;
                break;
            }
        }
    }

    fn attacker_death_update(&mut self) {
        self.attacker_pos = Coords { x: -1, y: -1 };
        self.attacker_death_count += 1;
    }

    fn defender_death_update(&mut self, defender_id: i32) {
        for i in 0..self.defenders.len() {
            if self.defenders[i].0 == defender_id {
                self.defenders[i].1 = false;
                self.
                break;
            }
        }
    }

    fn mine_blast_update(&mut self, damage_to_attacker: f32) {
        self.attacker_health = std::cmp::max(0, self.attacker_health - damage_to_attacker);
        if self.attacker_health == 0 {
            self.attacker_death_count += 1;
            self.attacker_pos = Coords { x: -1, y: -1 };
        }
    }

    fn bomb_blast_update(&mut self, final_damage_percentage: f32, increase_artifacts: i32) {
        self.damage_percentage = final_damage_percentage;
        self.artifacts += increase_artifacts;
    }

    //logic
    // movement of attacker

    pub fn attacker_movement(&mut self, frame_no: i32, attacker_delta: Coords) {
        if self.attacker.is_none() {
            // invalid event error
        }

        if (frame_no - self.frame_no) != 1 {
            // invalid frame error
        }
        self.frame_no += 1;

        if attacker_delta.x > 
    }

    // bomb placement
    // mines
    // defender
}
