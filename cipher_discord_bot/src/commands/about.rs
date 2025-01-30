use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use serenity::all::CreateEmbed;

use crate::app::AppContext;
use crate::app::AppError;
use crate::utils;

/// Show information about the bot.
#[poise::command(slash_command)]
pub async fn about<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[description = "Hide reply from other users. Defaults to True."] ephemeral: Option<bool>,
) -> Result<(), AppError<R::BackendError>> {
    let info = ctx.data().info();

    let avatar_url = utils::bot_avatar_url(&ctx).await?;

    let embed = CreateEmbed::new()
        .title(&info.about_title)
        .description(&info.about_description)
        .thumbnail(avatar_url)
        .field("Source Code", info.source_code_url.as_str(), false)
        .color(utils::bot_color(&ctx).await);

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(ephemeral.unwrap_or(true));

    ctx.send(reply).await?;

    Ok(())
}
