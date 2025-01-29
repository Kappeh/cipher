use clap::Parser;
use secrecy::ExposeSecret;

use super::DatabaseCredentials;
use super::DiscordCredentials;

/// Start the main discord bot application.
#[derive(Debug, Clone, Parser)]
pub struct Start {
    /// Credentials required to establish a database connection.
    #[command(flatten)]
    pub database: DatabaseCredentials,

    /// Credentials required to authenticate a bot with Discord.
    #[command(flatten)]
    pub discord: DiscordCredentials,
}

#[derive(Debug, thiserror::Error)]
pub enum StartError {
    #[error(transparent)]
    RepositoryBackendError(#[from] rotom_database::BackendError),
    #[error(transparent)]
    AppError(#[from] crate::app::AppError),
}

impl Start {
    pub async fn execute(self) -> Result<(), StartError> {
        log::debug!("{:#?}", self);

        let database_url = self.database.url.expose_secret();

        match self.database.dialect {
            #[cfg(feature = "mysql")]
            crate::cli::DatabaseDialect::Mysql => {
                log::info!("Running any pending database migrations.");
                rotom_database::mysql::run_pending_migrations(database_url)?;
                let repository_provider = rotom_database::mysql::repository_provider(database_url).await?;
                log::info!("Starting discord application.");
                crate::app::start(self.discord, repository_provider).await?;
            },
            #[cfg(feature = "postgres")]
            crate::cli::DatabaseDialect::Postgres => {
                log::info!("Running any pending database migrations.");
                rotom_database::postgres::run_pending_migrations(database_url)?;
                let repository_provider = rotom_database::postgres::repository_provider(database_url).await?;
                log::info!("Starting discord application.");
                crate::app::start(self.discord, repository_provider).await?;
            },
            #[cfg(feature = "sqlite")]
            crate::cli::DatabaseDialect::Sqlite => {
                log::info!("Running any pending database migrations.");
                rotom_database::sqlite::run_pending_migrations(database_url)?;
                let repository_provider = rotom_database::sqlite::repository_provider(database_url).await?;
                log::info!("Starting discord application.");
                crate::app::start(self.discord, repository_provider).await?;
            },
        }

        Ok(())
    }
}
