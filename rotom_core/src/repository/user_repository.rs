#[async_trait::async_trait]
pub trait UserRepository {
    type BackendError: std::error::Error;

    async fn user(&mut self, id: i32) -> Result<Option<User>, Self::BackendError>;

    async fn insert_user(&mut self, new_user: NewUser) -> Result<User, Self::BackendError>;

    async fn update_user(&mut self, user: User) -> Result<Option<User>, Self::BackendError>;

    async fn remove_user(&mut self, id: i32) -> Result<Option<User>, Self::BackendError>;
}

pub struct User {
    pub id: i32,
    pub discord_user_id: u64,
    pub pokemon_go_code: Option<String>,
    pub pokemon_pocket_code: Option<String>,
    pub switch_code: Option<String>,
}

pub struct NewUser {
    pub discord_user_id: u64,
    pub pokemon_go_code: Option<String>,
    pub pokemon_pocket_code: Option<String>,
    pub switch_code: Option<String>,
}
