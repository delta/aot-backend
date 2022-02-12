table! {
    attack_type (id) {
        id -> Int4,
        att_type -> Varchar,
        attack_radius -> Int4,
        attack_damage -> Int4,
    }
}

table! {
    attacker_path (id, game_id) {
        id -> Int4,
        y_coord -> Int4,
        x_coord -> Int4,
        is_emp -> Bool,
        game_id -> Int4,
        emp_type -> Nullable<Int4>,
        emp_time -> Nullable<Int4>,
    }
}

table! {
    block_type (id) {
        id -> Int4,
        name -> Varchar,
        width -> Int4,
        height -> Int4,
        entrance_x -> Int4,
        entrance_y -> Int4,
    }
}

table! {
    building_weights (time, building_id) {
        time -> Int4,
        building_id -> Int4,
        weight -> Int4,
    }
}

table! {
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

table! {
    level_constraints (level_id, block_id) {
        level_id -> Int4,
        block_id -> Int4,
        no_of_buildings -> Int4,
    }
}

table! {
    levels_fixture (id) {
        id -> Int4,
        start_date -> Date,
        end_date -> Date,
        no_of_bombs -> Int4,
    }
}

table! {
    map_layout (id) {
        id -> Int4,
        player -> Int4,
        level_id -> Int4,
        is_valid -> Bool,
    }
}

table! {
    map_spaces (id) {
        id -> Int4,
        map_id -> Int4,
        blk_type -> Int4,
        x_coordinate -> Int4,
        y_coordinate -> Int4,
        rotation -> Int4,
    }
}

table! {
    shortest_path (base_id, source_x, source_y, dest_x, dest_y) {
        base_id -> Int4,
        source_x -> Int4,
        source_y -> Int4,
        dest_x -> Int4,
        dest_y -> Int4,
        pathlist -> Varchar,
    }
}

table! {
    simulation_log (game_id) {
        game_id -> Int4,
        log_text -> Text,
    }
}

table! {
    user (id) {
        id -> Int4,
        name -> Varchar,
        email -> Varchar,
        phone -> Varchar,
        username -> Varchar,
        overall_rating -> Int4,
        is_pragyan -> Bool,
        password -> Varchar,
        is_verified -> Bool,
        highest_rating -> Int4,
    }
}

joinable!(attacker_path -> attack_type (emp_type));
joinable!(attacker_path -> game (game_id));
joinable!(building_weights -> block_type (building_id));
joinable!(game -> map_layout (map_layout_id));
joinable!(level_constraints -> block_type (block_id));
joinable!(level_constraints -> levels_fixture (level_id));
joinable!(map_layout -> levels_fixture (level_id));
joinable!(map_layout -> user (player));
joinable!(map_spaces -> block_type (blk_type));
joinable!(map_spaces -> map_layout (map_id));
joinable!(shortest_path -> map_layout (base_id));
joinable!(simulation_log -> game (game_id));

allow_tables_to_appear_in_same_query!(
    attack_type,
    attacker_path,
    block_type,
    building_weights,
    game,
    level_constraints,
    levels_fixture,
    map_layout,
    map_spaces,
    shortest_path,
    simulation_log,
    user,
);
