use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use rotom_core::repository::user_repository::NewUser;
use rotom_core::repository::user_repository::User;
use rotom_core::repository::user_repository::UserRepository;

use crate::mysql::schema::users;
use crate::BackendError;

use super::MysqlRepository;

#[async_trait::async_trait]
#[rustfmt::skip]
impl UserRepository for MysqlRepository<'_> {
    type BackendError = BackendError;

    async fn user(&mut self, id: i32) -> Result<Option<User>, Self::BackendError> {
        users::dsl::users.find(id)
            .first::<ModelUser>(&mut self.conn)
            .await
            .optional()
            .map(|option| option.map(User::from))
            .map_err(BackendError::from)
    }

    async fn insert_user(&mut self, new_user: NewUser) -> Result<User, Self::BackendError> {
        let model_new_user = ModelNewUser::from(new_user);

        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| async move {
                diesel::insert_into(users::table)
                    .values(&model_new_user)
                    .execute(conn)
                    .await?;

                users::table
                    .order(users::id.desc())
                    .select(ModelUser::as_select())
                    .first(conn)
                    .await
            }.scope_boxed())
            .await
            .map(User::from)
            .map_err(BackendError::from)
    }

    async fn update_user(&mut self, user: User) -> Result<Option<User>, Self::BackendError> {
        let model_user = ModelUser::from(user);

        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| async move {
                let option_previous = users::dsl::users.find(model_user.id)
                    .select(ModelUser::as_select())
                    .first(conn)
                    .await
                    .optional()?;

                let previous = match option_previous {
                    Some(previous) => previous,
                    None => return Ok(None),
                };

                diesel::update(users::dsl::users.find(model_user.id))
                    .set(&model_user)
                    .execute(conn)
                    .await?;

                Ok(Some(previous))
            }.scope_boxed())
            .await
            .map(|option| option.map(User::from))
            .map_err(BackendError::from)
    }

    async fn remove_user(&mut self, id: i32) -> Result<Option<User>, Self::BackendError> {
        self.conn
            .transaction::<_, diesel::result::Error, _>(move |conn| async move {
                let option_removed = users::dsl::users.find(id)
                    .select(ModelUser::as_select())
                    .first(conn)
                    .await
                    .optional()?;

                let removed = match option_removed {
                    Some(previous) => previous,
                    None => return Ok(None),
                };

                diesel::delete(users::dsl::users.find(id))
                    .execute(conn)
                    .await?;

                Ok(Some(removed))

            }.scope_boxed())
            .await
            .map(|option| option.map(User::from))
            .map_err(BackendError::from)
    }
}

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
struct ModelUser {
    id: i32,
    discord_user_id: i64,
    pokemon_go_code: Option<String>,
    pokemon_pocket_code: Option<String>,
    switch_code: Option<String>,
}

impl From<ModelUser> for User {
    fn from(value: ModelUser) -> Self {
        Self {
            id: value.id,
            discord_user_id: value.discord_user_id as u64,
            pokemon_go_code: value.pokemon_go_code,
            pokemon_pocket_code: value.pokemon_pocket_code,
            switch_code: value.switch_code,
        }
    }
}

impl From<User> for ModelUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            discord_user_id: value.discord_user_id as i64,
            pokemon_go_code: value.pokemon_go_code,
            pokemon_pocket_code: value.pokemon_pocket_code,
            switch_code: value.switch_code,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::mysql::Mysql))]
struct ModelNewUser {
    discord_user_id: i64,
    pokemon_go_code: Option<String>,
    pokemon_pocket_code: Option<String>,
    switch_code: Option<String>,
}

impl From<NewUser> for ModelNewUser {
    fn from(value: NewUser) -> Self {
        Self {
            discord_user_id: value.discord_user_id as i64,
            pokemon_go_code: value.pokemon_go_code,
            pokemon_pocket_code: value.pokemon_pocket_code,
            switch_code: value.switch_code,
        }
    }
}
