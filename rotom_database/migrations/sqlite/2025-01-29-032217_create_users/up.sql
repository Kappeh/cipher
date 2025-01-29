CREATE TABLE users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    discord_user_id BIGINT NOT NULL,
    pokemon_go_code VARCHAR(32),
    pokemon_pocket_code VARCHAR(32),
    switch_code VARCHAR(32)
);

CREATE UNIQUE INDEX users_discord_user_id ON users(discord_user_id);
