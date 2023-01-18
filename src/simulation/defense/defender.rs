#[allow(dead_code)]
pub struct Defender {
    pub id: i32,
    pub defender_type: i32,
    pub y_position: i32,
    pub is_alive: bool,
    pub target_id: Option<i32>,
    pub map_id: i32,
    pub defender_path: Vec<(i32, i32)>,
}
