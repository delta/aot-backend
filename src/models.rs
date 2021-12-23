#[derive(Queryable)]
pub struct AttackType {
    pub id: i32,
    pub att_type: i32,
    pub attack_radius: i32,
    pub attack_damage: i32,
}

#[derive(Queryable)]
pub struct AttackerPath {
    pub id: i32,
    pub y_coord: i32,
    pub x_coord: i32,
    pub is_emp: bool,
    pub game_id: i32,
    pub emp_type: i32,
    pub emp_time: i32,
}

#[derive(Queryable)]
pub struct BlockType {
    pub id: i32,
    pub name: String,
    pub width: i32,
    pub height: i32,
    pub revenue: i32,
}

#[derive(Queryable)]
pub struct Game {
    pub id: i32,
    pub attack_id: i32,
    pub defend_id: i32,
    pub map_layout_id: i32,
    pub attack_score: i32,
    pub defend_score: i32,
}

#[derive(Queryable)]
pub struct LevelsFixture {
    pub id: i32,
    pub start_date: i32,
    pub end_date: i32,
}

#[derive(Queryable)]
pub struct MapLayout {
    pub id: i32,
    pub player: i32,
    pub level_id: i32,
}

#[derive(Queryable)]
pub struct MapSpaces {
    pub id: i32,
    pub map_id: i32,
    pub blk_id: i32,
    pub x_coordinate: i32,
    pub y_coordinate: i32,
}

#[derive(Queryable)]
pub struct ShortestPath {
    pub base_id: i32,
    pub source_x: i32,
    pub source_y: i32,
    pub dest_x: i32,
    pub dest_y: i32,
    pub pathlist: String,
}

#[derive(Queryable)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub email: String,
    pub phone: String,
    pub username: String,
    pub is_pragyan: bool,
    pub password: String,
    pub is_verified: bool,
}