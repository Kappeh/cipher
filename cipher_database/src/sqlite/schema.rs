// @generated automatically by Diesel CLI.

diesel::table! {
    profiles (id) {
        id -> Integer,
        user_id -> Integer,
        thumbnail_url -> Nullable<Text>,
        image_url -> Nullable<Text>,
        trainer_class -> Nullable<Text>,
        nature -> Nullable<Text>,
        partner_pokemon -> Nullable<Text>,
        starting_region -> Nullable<Text>,
        favourite_food -> Nullable<Text>,
        likes -> Nullable<Text>,
        quotes -> Nullable<Text>,
        pokemon_go_code -> Nullable<Text>,
        pokemon_pocket_code -> Nullable<Text>,
        switch_code -> Nullable<Text>,
        created_at -> Timestamp,
        is_active -> Bool,
    }
}

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
    }
}

diesel::joinable!(profiles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    profiles,
    staff_roles,
    users,
);
