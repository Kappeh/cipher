use diesel::Connection;
use diesel::MysqlConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;
use repository::MysqlRepositoryProvider;

use crate::BackendError;

pub mod repository;
mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/mysql");

pub fn run_pending_migrations(database_url: &str) -> Result<(), BackendError> {
    let mut connection = MysqlConnection::establish(database_url)?;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map_err(BackendError::DieselMigrationError)?;
    Ok(())
}

pub async fn repository_provider(database_url: &str) -> Result<MysqlRepositoryProvider, BackendError> {
    let config = AsyncDieselConnectionManager::new(database_url);
    let pool = Pool::builder().build(config).await?;
    Ok(MysqlRepositoryProvider::new(pool))
}
