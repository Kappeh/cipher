use diesel::SqliteConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::bb8::PooledConnection;
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use rotom_core::repository::Repository;
use rotom_core::repository::RepositoryProvider;

use crate::BackendError;

mod user_repository;

pub struct SqliteRepository<'a> {
    conn: PooledConnection<'a, SyncConnectionWrapper<SqliteConnection>>,
}

impl<'a> SqliteRepository<'a> {
    pub fn new(conn: PooledConnection<'a, SyncConnectionWrapper<SqliteConnection>>) -> Self {
        Self { conn }
    }
}

impl Repository for SqliteRepository<'_> {
    type BackendError = BackendError;
}

pub struct SqliteRepositoryProvider {
    pool: Pool<SyncConnectionWrapper<SqliteConnection>>,
}

impl SqliteRepositoryProvider {
    pub fn new(pool: Pool<SyncConnectionWrapper<SqliteConnection>>) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl RepositoryProvider for SqliteRepositoryProvider {
    type BackendError = BackendError;

    type Repository<'a> = SqliteRepository<'a>;

    async fn get(&self) -> Result<Self::Repository<'_>, Self::BackendError> {
        self.pool.get().await.map(SqliteRepository::new).map_err(BackendError::from)
    }
}
