// @generated automatically by Diesel CLI.

diesel::table! {
    staff_roles (id) {
        id -> Int4,
        discord_role_id -> Int8,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        discord_user_id -> Int8,
        #[max_length = 32]
        pokemon_go_code -> Nullable<Varchar>,
        #[max_length = 32]
        pokemon_pocket_code -> Nullable<Varchar>,
        #[max_length = 32]
        switch_code -> Nullable<Varchar>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    staff_roles,
    users,
);
