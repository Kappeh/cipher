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
        #[max_length = 32]
        pokemon_go_code -> Nullable<Varchar>,
        #[max_length = 32]
        pokemon_pocket_code -> Nullable<Varchar>,
        #[max_length = 32]
        switch_code -> Nullable<Varchar>,
        created_at -> Timestamp,
        is_active -> Bool,
    }
}

diesel::table! {
    staff_roles (id) {
        id -> Integer,
        discord_role_id -> Bigint,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        discord_user_id -> Bigint,
    }
}

diesel::joinable!(profiles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    profiles,
    staff_roles,
    users,
);
