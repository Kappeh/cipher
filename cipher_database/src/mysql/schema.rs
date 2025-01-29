// @generated automatically by Diesel CLI.

diesel::table! {
    users (id) {
        id -> Integer,
        discord_user_id -> Bigint,
        #[max_length = 32]
        pokemon_go_code -> Nullable<Varchar>,
        #[max_length = 32]
        pokemon_pocket_code -> Nullable<Varchar>,
        #[max_length = 32]
        switch_code -> Nullable<Varchar>,
    }
}
