use diesel::prelude::*;
use diesel_async::scoped_futures::ScopedFutureExt;
use diesel_async::AsyncConnection;
use diesel_async::RunQueryDsl;
use cipher_core::repository::user_repository::NewUser;
use cipher_core::repository::user_repository::User;
use cipher_core::repository::user_repository::UserRepository;
use cipher_core::repository::RepositoryError;

use crate::sqlite::schema::users;
use crate::BackendError;

use super::SqliteRepository;

#[async_trait::async_trait]
#[rustfmt::skip]
impl UserRepository for SqliteRepository<'_> {
    type BackendError = BackendError;

    async fn user(&mut self, id: i32) -> Result<Option<User>, RepositoryError<Self::BackendError>> {
        users::dsl::users.find(id)
            .first::<ModelUser>(&mut self.conn)
            .await
            .optional()
            .map(|option| option.map(User::from))
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn user_by_discord_user_id(&mut self, id: u64) -> Result<Option<User>, RepositoryError<Self::BackendError>> {
        let model_id = id as i64;

        users::dsl::users
            .filter(users::dsl::discord_user_id.eq(model_id))
            .first::<ModelUser>(&mut self.conn)
            .await
            .optional()
            .map(|option| option.map(User::from))
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn insert_user(&mut self, new_user: NewUser) -> Result<User, RepositoryError<Self::BackendError>> {
        let model_new_user = ModelNewUser::from(new_user);

        self.conn
            .transaction::<_, diesel::result::Error, _>(|conn| async move {
                diesel::insert_into(users::table)
                    .values(&model_new_user)
                    .returning(ModelUser::as_returning())
                    .get_result(conn)
                    .await
            }.scope_boxed())
            .await
            .map(User::from)
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }

    async fn update_user(&mut self, user: User) -> Result<Option<User>, RepositoryError<Self::BackendError>> {
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
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }
}

#[derive(Queryable, Selectable, AsChangeset)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct ModelUser {
    id: i32,
    discord_user_id: i64,
}

impl From<ModelUser> for User {
    fn from(value: ModelUser) -> Self {
        Self {
            id: value.id,
            discord_user_id: value.discord_user_id as u64,
        }
    }
}

impl From<User> for ModelUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            discord_user_id: value.discord_user_id as i64,
        }
    }
}

#[derive(Insertable)]
#[diesel(table_name = users)]
#[diesel(treat_none_as_null = true)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
struct ModelNewUser {
    discord_user_id: i64,
}

impl From<NewUser> for ModelNewUser {
    fn from(value: NewUser) -> Self {
        Self {
            discord_user_id: value.discord_user_id as i64,
        }
    }
}
