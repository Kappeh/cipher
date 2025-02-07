CREATE TABLE profiles (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,

    thumbnail_url TEXT,
    image_url TEXT,

    trainer_class TEXT,
    nature TEXT,
    partner_pokemon TEXT,
    starting_region TEXT,
    favourite_food TEXT,
    likes TEXT,
    quotes TEXT,

    pokemon_go_code VARCHAR(32),
    pokemon_pocket_code VARCHAR(32),
    switch_code VARCHAR(32),

    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    is_active BOOLEAN NOT NULL DEFAULT true,

    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE RESTRICT ON UPDATE CASCADE
);

CREATE INDEX profiles_user_id ON profiles(user_id);
CREATE INDEX profiles_created_at ON profiles(created_at);
CREATE INDEX profiles_is_active ON profiles(is_active);

INSERT INTO profiles (user_id, pokemon_go_code, pokemon_pocket_code, switch_code)
SELECT id, pokemon_go_code, pokemon_pocket_code, switch_code
FROM users;

ALTER TABLE users DROP COLUMN pokemon_go_code;
ALTER TABLE users DROP COLUMN pokemon_pocket_code;
ALTER TABLE users DROP COLUMN switch_code;
