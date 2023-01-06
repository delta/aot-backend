use crate::models::*;
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
}

impl Defender {
    #[allow(dead_code)]
    pub fn new(defender_type: &DefenderType, id: i32, x_position: i32, y_position: i32) -> Self {
        Self {
            id,
            defender_type: defender_type.id,
            radius: defender_type.radius,
            speed: defender_type.speed,
            damage: defender_type.damage,
            x_position,
            y_position,
            is_alive: true,
            target_id: None,
        }
    }

    #[allow(dead_code)]
    pub fn simulate() {
        //checking any attcker within his range

        //assign target

        //change the position

        //after reaching target defender will be out
    }
}
