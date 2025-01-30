use cipher_core::repository::RepositoryProvider;
use serenity::all::{Color, GuildId};

use crate::app::{AppCommand, AppContext, AppError};

pub async fn register_in_guilds<R>(
    serenity_ctx: &serenity::client::Context,
    commands: &[AppCommand<R, R::BackendError>],
    guilds: &[GuildId],
)
where
    R: RepositoryProvider,
{
    for guild in guilds {
        let result = poise::builtins::register_in_guild(serenity_ctx, commands, *guild).await;
        match (guild.name(serenity_ctx), result) {
            (None, Err(err)) => log::warn!("Failed to register command in guild with id {}: {}", guild.get().to_string(), err),
            (None, Ok(())) => log::info!("Successfully registered commands in guild with id {}", guild.get().to_string()),
            (Some(guild_name), Err(err)) => log::warn!("Failed to register command in guild with id {} ({}): {}", guild.get().to_string(), guild_name, err),
            (Some(guild_name), Ok(())) => log::info!("Successfully registered commands in guild with id {} ({})", guild.get().to_string(), guild_name),
        }
    }
}

pub async fn bot_color<R>(ctx: &AppContext<'_, R, R::BackendError>) -> Color
where
    R: RepositoryProvider + Send + Sync,
{
    let member = match ctx.guild_id() {
        Some(guild) => guild.member(ctx, ctx.framework.bot_id).await.ok(),
        None => None,
    };

    member.and_then(|m| m.colour(ctx)).unwrap_or(Color::BLURPLE)
}

pub async fn bot_avatar_url<R>(ctx: &AppContext<'_, R, R::BackendError>) -> Result<String, AppError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync,
{
    let member_avatar_url = match ctx.guild_id() {
        Some(guild) => guild.member(ctx, ctx.framework.bot_id).await?.avatar_url(),
        None => None,
    };

    let avatar_url = match member_avatar_url {
        Some(url) => url,
        None => {
            let user = ctx.framework.bot_id.to_user(ctx).await?;
            user.avatar_url()
                .or_else(|| user.static_avatar_url())
                .unwrap_or_else(|| user.default_avatar_url())
        },
    };

    Ok(avatar_url)
}
