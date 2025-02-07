ALTER TABLE users ADD COLUMN pokemon_go_code VARCHAR(32);
ALTER TABLE users ADD COLUMN pokemon_pocket_code VARCHAR(32);
ALTER TABLE users ADD COLUMN switch_code VARCHAR(32);

UPDATE users
INNER JOIN (
    SELECT user_id, pokemon_go_code, pokemon_pocket_code, switch_code
    FROM profiles
    WHERE is_active = true
) AS subquery
ON users.id = subquery.user_id
SET
	users.pokemon_go_code = subquery.pokemon_go_code,
    users.pokemon_pocket_code = subquery.pokemon_pocket_code,
    users.switch_code = subquery.switch_code;

DROP INDEX profiles_created_at ON profiles;
DROP INDEX profiles_is_active ON profiles;

DROP TABLE profiles;
