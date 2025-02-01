CREATE TABLE staff_roles (
    id SERIAL PRIMARY KEY,
    discord_role_id BIGINT NOT NULL
);

CREATE UNIQUE INDEX staff_roles_discord_role_id ON staff_roles(discord_role_id);
