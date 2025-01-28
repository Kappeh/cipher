use diesel::Connection;
use diesel::PgConnection;
use diesel_async::pooled_connection::bb8::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_migrations::embed_migrations;
use diesel_migrations::EmbeddedMigrations;
use diesel_migrations::MigrationHarness;

use crate::BackendError;

mod schema;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations/postgres");

pub fn run_pending_migrations(database_url: &str) -> Result<(), BackendError> {
    let mut connection = PgConnection::establish(database_url)?;
    connection.run_pending_migrations(MIGRATIONS)
        .map_err(BackendError::DieselMigrationError)?;
    Ok(())
}

pub async fn establish_async_pool(
    database_url: &str,
) -> Result<Pool<AsyncPgConnection>, BackendError> {
    let config = AsyncDieselConnectionManager::new(database_url);
    let pool = Pool::builder().build(config).await?;
    Ok(pool)
}
