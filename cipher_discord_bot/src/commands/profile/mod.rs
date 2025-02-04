use cipher_core::repository::user_repository::UserRepository;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use serenity::all::Member;
use serenity::all::User;

use crate::app::AppContext;
use crate::app::AppError;

mod codes;

pub use codes::cmu_profile_edit;

/// Edit and show profiles.
#[poise::command(
    slash_command,
    subcommands(
        "codes::codes",
        "show",
    ),
)]
pub async fn profile<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}

#[poise::command(context_menu_command = "Show User Profile", guild_only)]
pub async fn cmu_profile_show<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    user: User,
) -> Result<(), AppError<R::BackendError>> {
    let guild = match ctx.guild_id() {
        Some(guild) => guild,
        None => return Ok(()),
    };

    let member = guild.member(ctx, user.id).await?;

    show_inner(ctx, member, true).await
}

/// Show your profile or someone else's.
#[poise::command(slash_command, guild_only)]
async fn show<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[rename = "member"]
    #[description = "The profile to show."]
    option_member: Option<serenity::all::Member>,
    #[description = "Hide reply from other users. Defaults to True."]
    ephemeral: Option<bool>,
) -> Result<(), AppError<R::BackendError>> {
    let member = match option_member {
        Some(member) => member,
        None => ctx.author_member().await.ok_or(AppError::UnknownCacheOrHttpError)?.into_owned(),
    };

    show_inner(ctx, member, ephemeral.unwrap_or(true)).await
}

async fn show_inner<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    member: Member,
    ephemeral: bool,
) -> Result<(), AppError<R::BackendError>> {
    let mut repo = ctx.data.repository().await?;

    let avatar_url = crate::utils::member_avatar_url(&member);

    let embed_color = match member.colour(ctx) {
        Some(color) => color,
        None => crate::utils::bot_color(&ctx).await,
    };

    let mut embed = CreateEmbed::new()
        .title(member.display_name())
        .thumbnail(avatar_url)
        .color(embed_color);

    let mut is_profile_empty = true;

    if let Some(user_info) = repo.user_by_discord_user_id(member.user.id.get()).await? {
        if let Some(code) = user_info.pokemon_go_code {
            embed = embed.field("Pokémon Go Friend Code", code, false);
            is_profile_empty = false;
        }

        if let Some(code) = user_info.pokemon_pocket_code {
            embed = embed.field("Pokémon TCG Pocket Friend Code", code, false);
            is_profile_empty = false;
        }

        if let Some(code) = user_info.switch_code {
            embed = embed.field("Nintendo Switch Friend Code", code, false);
            is_profile_empty = false;
        }
    };

    if is_profile_empty {
        embed = embed.description("No information to show.");
    }

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(ephemeral);

    ctx.send(reply).await?;

    Ok(())
}
