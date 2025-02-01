use cipher_core::repository::RepositoryError;
use cipher_core::repository::RepositoryProvider;
use secrecy::ExposeSecret;
use serenity::all::GatewayIntents;
use serenity::Client;

use crate::cli::AppInfo;
use crate::cli::DiscordCredentials;

mod event_handler;
mod framework;
mod on_error;

#[derive(Debug, thiserror::Error)]
pub enum AppStartError {
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
}

pub struct AppData<R> {
    repository_provider: R,
    qualified_command_names: Vec<String>,
    info: AppInfo,
}

#[derive(Debug, thiserror::Error)]
pub enum AppError<E> {
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError<E>),
    #[error("staff-only command used by non-staff user")]
    StaffOnly { command_name: String },
}

pub type AppContext<'a, R, E> = poise::ApplicationContext<'a, AppData<R>, AppError<E>>;
pub type AppCommand<R, E> = poise::Command<AppData<R>, AppError<E>>;

pub async fn start<R>(credentials: DiscordCredentials, info: AppInfo, repository_provider: R) -> Result<(), AppStartError>
where
    R: RepositoryProvider + Send + Sync + 'static,
    R::BackendError: Send + Sync,
    for<'a> R::Repository<'a>: Send + Sync,
{
    let mut client = Client::builder(credentials.bot_token.expose_secret(), GatewayIntents::all())
        .framework(framework::framework(repository_provider, info))
        .await?;

    let shard_manager = client.shard_manager.clone();
    tokio::spawn(async move {
        if let Err(err) = tokio::signal::ctrl_c().await {
            log::error!("Failed to register ctrl+c handler: {}", err);
        }
        log::info!("Stopping.");
        shard_manager.shutdown_all().await;
    });

    client.start().await.map_err(AppStartError::from)
}

impl<R> AppData<R>
where
    R: RepositoryProvider,
{
    pub async fn repository(&self) -> Result<R::Repository<'_>, RepositoryError<R::BackendError>> {
        self.repository_provider.get().await
    }
}

impl<R> AppData<R> {
    pub fn qualified_command_names(&self) -> &[String] {
        &self.qualified_command_names
    }

    pub fn info(&self) -> &AppInfo {
        &self.info
    }
}
