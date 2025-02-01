// @generated automatically by Diesel CLI.

diesel::table! {
    staff_roles (id) {
        id -> Integer,
        discord_role_id -> BigInt,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        discord_user_id -> BigInt,
        pokemon_go_code -> Nullable<Text>,
        pokemon_pocket_code -> Nullable<Text>,
        switch_code -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    staff_roles,
    users,
);
