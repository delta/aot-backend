use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct Coords {
    pub x: i32,
    pub y: i32,
}

#[derive(Serialize)]
pub struct Attacker {
    pub attacker_type: i32,
    pub attacker_pos: Coords,
    pub attacker_health: i32,
    pub attacker_speed: i32,
}

#[derive(Serialize)]
pub struct Defender {
    pub id: i32,
    pub defender_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub hut_x: i32,
    pub hut_y: i32,
    pub is_alive: bool,
    pub damage_dealt: bool,
    pub target_id: Option<i32>,
    pub path: Vec<(i32, i32)>,
    pub path_in_current_frame: Vec<DefenderPathStats>,
}

#[derive(Serialize)]
pub struct State {
    pub frame_no: i32,
    pub attacker_id: i32,
    pub defender_id: i32,
    pub attacker: Attacker,
    pub bomb_type: i32,
    pub attacker_death_count: i32,
    pub damage_percentage: f32,
    pub artifacts: i32,
    pub defenders: Vec<Defender> // (defender_id, is_alive, { defender_pos_x, defender_pos_y })
}

// impl constructor, and other necessary functions skeleton

impl State {
    pub fn new(
        attacker_id: i32,
        defender_id: i32,
        defenders: Vec<(i32, bool, Coords)>,
    ) -> State {
        State {
            frame_no: 0,
            attacker_id: attacker_id,
            defender_id: defender_id,
            attacker_type: -1,
            bomb_type: -1,
            attacker_pos: Coords { x: -1, y: -1 },
            attacker_health: 0,
            attacker_death_count: -1,
            damage_percentage: 0.0,
            artifacts: 0,
            defenders: defenders,
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
