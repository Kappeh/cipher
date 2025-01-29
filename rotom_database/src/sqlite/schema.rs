// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        discord_user_id -> BigInt,
        pokemon_go_code -> Nullable<Text>,
        pokemon_pocket_code -> Nullable<Text>,
        switch_code -> Nullable<Text>,
    }
}
