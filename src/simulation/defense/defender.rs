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
    // #[allow(dead_code)]
    // pub fn new() -> Self {
    //     println!("Defender Created");
    //     todo!()
    // }

    #[allow(dead_code)]
    pub fn simulate() {
        todo!()
    }
}
