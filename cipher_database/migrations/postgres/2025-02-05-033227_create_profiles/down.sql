ALTER TABLE users ADD COLUMN pokemon_go_code VARCHAR(32);
ALTER TABLE users ADD COLUMN pokemon_pocket_code VARCHAR(32);
ALTER TABLE users ADD COLUMN switch_code VARCHAR(32);

UPDATE users
SET
	pokemon_go_code = subquery.pokemon_go_code,
    pokemon_pocket_code = subquery.pokemon_pocket_code,
    switch_code = subquery.switch_code
FROM (
    SELECT user_id, pokemon_go_code, pokemon_pocket_code, switch_code
    FROM profiles
    WHERE is_active = true
) AS subquery
WHERE id = subquery.user_id;

DROP INDEX profiles_user_id;
DROP INDEX profiles_created_at;
DROP INDEX profiles_is_active;

DROP TABLE profiles;
