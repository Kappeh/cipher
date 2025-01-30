use poise::Framework;
use poise::FrameworkOptions;
use cipher_core::repository::RepositoryProvider;

use crate::commands;

use super::event_handler;
use super::on_error;
use super::AppData;
use super::AppError;

pub fn framework<R>(repository_provider: R) -> Framework<AppData<R>, AppError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync + 'static,
    R::BackendError: Send + Sync,
    for<'a> R::Repository<'a>: Send + Sync,
{
    let commands = commands::commands();

    let app_data = AppData {
        repository_provider,
        qualified_command_names: commands::qualified_command_names(&commands),
    };

    let options = FrameworkOptions::<AppData<R>, AppError<R::BackendError>> {
        commands,
        on_error: |framework_error| {
            Box::pin(async move { on_error::on_error(framework_error).await })
        },
        event_handler: |serenity_ctx, event, framework_ctx, data| {
            Box::pin(async move {
                event_handler::event_handler(serenity_ctx, event, framework_ctx, data).await
            })
        },
        ..Default::default()
    };

    Framework::builder()
        .options(options)
        .setup(|_ctx, _ready, _framework| Box::pin(async move { Ok(app_data) }))
        .build()
}
