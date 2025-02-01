use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::AsyncMysqlConnection;
use cipher_core::repository::Repository;
use cipher_core::repository::RepositoryError;
use cipher_core::repository::RepositoryProvider;

use crate::BackendError;

mod staff_role_repository;
mod user_repository;

pub struct MysqlRepository<'a> {
    conn: PooledConnection<'a, AsyncMysqlConnection>,
}

impl<'a> MysqlRepository<'a> {
    pub fn new(conn: PooledConnection<'a, AsyncMysqlConnection>) -> Self {
        Self { conn }
    }
}

impl Repository for MysqlRepository<'_> {
    type BackendError = BackendError;
}

pub struct MysqlRepositoryProvider {
    pool: Pool<AsyncMysqlConnection>,
}

impl MysqlRepositoryProvider {
    pub fn new(pool: Pool<AsyncMysqlConnection>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RepositoryProvider for MysqlRepositoryProvider {
    type BackendError = BackendError;

    type Repository<'a> = MysqlRepository<'a>;

    async fn get(&self) -> Result<Self::Repository<'_>, RepositoryError<Self::BackendError>> {
        self.pool
            .get()
            .await
            .map(MysqlRepository::new)
            .map_err(|err| RepositoryError(BackendError::from(err)))
    }
}
