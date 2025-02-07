use super::RepositoryError;

#[async_trait::async_trait]
pub trait UserRepository {
    type BackendError: std::error::Error;

    async fn user(&mut self, id: i32) -> Result<Option<User>, RepositoryError<Self::BackendError>>;

    async fn user_by_discord_user_id(&mut self, id: u64) -> Result<Option<User>, RepositoryError<Self::BackendError>>;

    async fn insert_user(&mut self, new_user: NewUser) -> Result<User, RepositoryError<Self::BackendError>>;

    async fn update_user(&mut self, user: User) -> Result<Option<User>, RepositoryError<Self::BackendError>>;
}

pub struct User {
    pub id: i32,
    pub discord_user_id: u64,
}

pub struct NewUser {
    pub discord_user_id: u64,
}
