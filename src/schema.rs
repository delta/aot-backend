// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "building_category"))]
    pub struct BuildingCategory;
}

diesel::table! {
    attack_type (id) {
        id -> Int4,
        att_type -> Varchar,
        attack_radius -> Int4,
        attack_damage -> Int4,
    }
}

diesel::table! {
    attacker_type (id) {
        id -> Int4,
        max_health -> Int4,
        speed -> Int4,
        amt_of_emps -> Int4,
    }
}

diesel::table! {
    block_type (id) {
        id -> Int4,
        name -> Varchar,
        width -> Int4,
        height -> Int4,
        entrance_x -> Int4,
        entrance_y -> Int4,
        capacity -> Int4,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BuildingCategory;

    building_type (id) {
        id -> Int4,
        defender_type -> Nullable<Int4>,
        diffuser_type -> Nullable<Int4>,
        mine_type -> Nullable<Int4>,
        blk_type -> Int4,
        building_category -> BuildingCategory,
    }
}

diesel::table! {
    building_weights (time, building_id) {
        time -> Int4,
        building_id -> Int4,
        weight -> Int4,
    }
}

diesel::table! {
    defender_type (id) {
        id -> Int4,
        speed -> Int4,
        damage -> Int4,
        radius -> Int4,
    }
}

diesel::table! {
    diffuser_type (id) {
        id -> Int4,
        radius -> Int4,
        speed -> Int4,
    }
}

diesel::table! {
    drone_usage (id) {
        id -> Int4,
        attacker_id -> Int4,
        map_id -> Int4,
        drone_x -> Int4,
        drone_y -> Int4,
    }
}

diesel::table! {
    emp_type (id) {
        id -> Int4,
        att_type -> Varchar,
        attack_radius -> Int4,
        attack_damage -> Int4,
    }
}

diesel::table! {
    game (id) {
        id -> Int4,
        attack_id -> Int4,
        defend_id -> Int4,
        map_layout_id -> Int4,
        attack_score -> Int4,
        defend_score -> Int4,
        robots_destroyed -> Int4,
        emps_used -> Int4,
        damage_done -> Int4,
        is_attacker_alive -> Bool,
    }
}

diesel::table! {
    level_constraints (level_id, building_id) {
        level_id -> Int4,
        no_of_buildings -> Int4,
        building_id -> Int4,
    }
}

diesel::table! {
    levels_fixture (id) {
        id -> Int4,
        start_date -> Timestamp,
        end_date -> Timestamp,
        no_of_bombs -> Int4,
        no_of_robots -> Int4,
        rating_factor -> Float4,
        no_of_attackers -> Int4,
    }
}

diesel::table! {
    map_layout (id) {
        id -> Int4,
        player -> Int4,
        level_id -> Int4,
        is_valid -> Bool,
    }
}

diesel::table! {
    map_spaces (id) {
        id -> Int4,
        map_id -> Int4,
        x_coordinate -> Int4,
        y_coordinate -> Int4,
        rotation -> Int4,
        building_type -> Int4,
    }
}

diesel::table! {
    mine_type (id) {
        id -> Int4,
        radius -> Int4,
        damage -> Int4,
    }
}

diesel::table! {
    shortest_path (base_id, source_x, source_y, dest_x, dest_y) {
        base_id -> Int4,
        source_x -> Int4,
        source_y -> Int4,
        dest_x -> Int4,
        dest_y -> Int4,
        pathlist -> Varchar,
    }
}

diesel::table! {
    simulation_log (game_id) {
        game_id -> Int4,
        log_text -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        phone -> Varchar,
        username -> Varchar,
        overall_rating -> Float4,
        is_pragyan -> Bool,
        password -> Varchar,
        is_verified -> Bool,
        highest_rating -> Float4,
    }
}

diesel::joinable!(building_type -> block_type (blk_type));
diesel::joinable!(building_type -> defender_type (defender_type));
diesel::joinable!(building_type -> diffuser_type (diffuser_type));
diesel::joinable!(building_type -> mine_type (mine_type));
diesel::joinable!(building_weights -> block_type (building_id));
diesel::joinable!(drone_usage -> map_layout (map_id));
diesel::joinable!(drone_usage -> user (attacker_id));
diesel::joinable!(game -> map_layout (map_layout_id));
diesel::joinable!(level_constraints -> building_type (building_id));
diesel::joinable!(level_constraints -> levels_fixture (level_id));
diesel::joinable!(map_layout -> levels_fixture (level_id));
diesel::joinable!(map_layout -> user (player));
diesel::joinable!(map_spaces -> building_type (building_type));
diesel::joinable!(map_spaces -> map_layout (map_id));
diesel::joinable!(shortest_path -> map_layout (base_id));
diesel::joinable!(simulation_log -> game (game_id));

diesel::allow_tables_to_appear_in_same_query!(
    attack_type,
    attacker_type,
    block_type,
    building_type,
    building_weights,
    defender_type,
    diffuser_type,
    drone_usage,
    emp_type,
    game,
    level_constraints,
    levels_fixture,
    map_layout,
    map_spaces,
    mine_type,
    shortest_path,
    simulation_log,
    user,
);
