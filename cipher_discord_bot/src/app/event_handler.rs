use cipher_core::repository::RepositoryProvider;
use serenity::all::FullEvent;

use crate::utils;

use super::AppData;
use super::AppError;

pub async fn event_handler<R: RepositoryProvider>(
    serenity_ctx: &serenity::client::Context,
    event: &FullEvent,
    framework_ctx: poise::FrameworkContext<'_, AppData<R>, AppError<R::BackendError>>,
    _data: &AppData<R>,
) -> Result<(), AppError<R::BackendError>> {
    match event {
        FullEvent::Ready { data_about_bot } => {
            log::info!(
                "Connected as {} in {} guild(s).",
                data_about_bot.user.name,
                data_about_bot.guilds.len()
            );
        }
        FullEvent::CacheReady { guilds } => {
            utils::register_in_guilds(serenity_ctx, &framework_ctx.options.commands, guilds).await;
        }
        _ => {}
    }

    Ok(())
}
