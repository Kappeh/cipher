use cipher_core::repository::user_repository::UserRepository;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use serenity::all::Color;
use serenity::all::CreateEmbed;

use crate::app::AppContext;
use crate::app::AppError;
use crate::utils;

mod codes;

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

/// Show your profile or someone else's.
#[poise::command(slash_command)]
async fn show<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[rename = "member"]
    #[description = "The profile to show."]
    option_member: Option<serenity::all::Member>,
) -> Result<(), AppError<R::BackendError>> {
    let mut repo = ctx.data.repository().await?;

    let member = match option_member {
        Some(member) => member,
        None => match ctx.author_member().await {
            Some(member) => member.into_owned(),
            None => {
                let embed = CreateEmbed::new()
                    .title("Error")
                    .description("This command can only be used in server.")
                    .color(Color::RED);

                let reply = CreateReply::default()
                    .embed(embed)
                    .ephemeral(true);

                ctx.send(reply).await?;

                return Ok(())
            },
        },
    };

    let avatar_url = member.avatar_url()
        .or_else(|| member.user.avatar_url())
        .or_else(|| member.user.static_avatar_url())
        .unwrap_or_else(|| member.user.default_avatar_url());

    let embed_color = match member.colour(ctx) {
        Some(color) => color,
        None => utils::bot_color(&ctx).await,
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
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}
