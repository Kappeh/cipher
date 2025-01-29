use poise::CreateReply;
use cipher_core::repository::RepositoryProvider;
use serenity::all::CreateEmbed;

use crate::app::AppContext;
use crate::app::AppError;
use crate::utils;

/// Is the bot alive or dead? :thinking:
#[poise::command(slash_command)]
pub async fn ping<R: RepositoryProvider + Send + Sync>(ctx: AppContext<'_, R, R::BackendError>) -> Result<(), AppError<R::BackendError>> {
    let embed = CreateEmbed::new()
        .title("Pong :ping_pong:")
        .color(utils::bot_color(&ctx).await);

    let reply = CreateReply::default()
        .embed(embed);

    ctx.send(reply).await?;

    Ok(())
}
