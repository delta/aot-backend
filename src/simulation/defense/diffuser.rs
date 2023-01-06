use crate::models::*;
pub struct Diffuser {
    pub id: i32,
    pub diffuser_type: i32,
    pub radius: i32,
    pub speed: i32,
    // pub damage: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub target_emp_id: Option<i32>,
}

#[allow(dead_code)]
pub struct Diffusers(Vec<Diffuser>);

impl Diffuser {
    #[allow(dead_code)]
    pub fn new(diffuser_type: &DiffuserType, id: i32, x_position: i32, y_position: i32) -> Self {
        Self {
            id,
            diffuser_type: diffuser_type.id,
            radius: diffuser_type.radius,
            speed: diffuser_type.speed,
            // damage:diffuser_type.damage,
            x_position,
            y_position,
            is_alive: true,
            target_emp_id: None,
        }
    }

    #[allow(dead_code)]
    pub fn simulate() {
        //get list of emps within radius

        //choose one emp optimally

        //deacticate  it once used
    }
}
