pub struct Diffuser {
    pub id: i32,
    pub diffuser_type: i32,
    pub radius: i32,
    pub speed: i32,
    pub damage: i32,
    pub x_position: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub target_emp_id: Option<i32>,
}

impl Diffuser {
    // #[allow(dead_code)]
    // pub fn new() -> Self {
    //     println!("Diffuser Created");
    //     todo!()
    // }

    #[allow(dead_code)]
    pub fn simulate() {
        todo!()
    }
}
