use cipher_core::repository::user_repository::NewUser;
use cipher_core::repository::user_repository::User;
use cipher_core::repository::user_repository::UserRepository;
use cipher_core::repository::RepositoryError;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use poise::Modal;
use serenity::all::Color;
use serenity::all::CreateEmbed;
use serenity::all::Member;

use crate::app::AppContext;
use crate::app::AppError;
use crate::utils;

/// Edit and show profiles.
#[poise::command(
    slash_command,
    subcommands(
        "edit",
        "overwrite",
        "show",
    ),
)]
pub async fn profile<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}

#[derive(Debug, poise::Modal)]
#[name = "Edit Profile Profile"]
struct EditProfileModal {
    #[name = "Pokémon Go Friend Code"]
    #[placeholder = "0000 0000 0000"]
    pokemon_go_code: Option<String>,
    #[name = "Pokémon TCG Pocket Friend Code"]
    #[placeholder = "0000 0000 0000 0000"]
    pokemon_pocket_code: Option<String>,
    #[name = "Nintendo Switch Friend Code"]
    #[placeholder = "SW-0000-0000-0000"]
    switch_code: Option<String>,
}

/// Edit your profile.
#[poise::command(slash_command)]
pub async fn edit<R: RepositoryProvider + Send + Sync>(ctx: AppContext<'_, R, R::BackendError>) -> Result<(), AppError<R::BackendError>> {
    let author_id = ctx.author().id.get();

    let embed = match edit_user(ctx, author_id).await {
        Ok(()) => CreateEmbed::new()
            .title("Changes Saved")
            .description("Your changes have been saved successfully.")
            .color(utils::bot_color(&ctx).await),
        Err(EditError::ValidationError(errors)) => CreateEmbed::new()
            .title("Validation Error")
            .description(errors.join("\n"))
            .color(Color::RED),
        Err(EditError::SerenityError(err)) => return Err(AppError::from(err)),
        Err(EditError::RepositoryError(err)) => return Err(AppError::from(err)),
    };

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}

/// Edit any user's profile.
#[poise::command(
    slash_command,
    hide_in_help,
    owners_only,
)]
pub async fn overwrite<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[description = "The profile to edit."]
    member: Member,
) -> Result<(), AppError<R::BackendError>> {
    let member_id = member.user.id.get();

    let embed = match edit_user(ctx, member_id).await {
        Ok(()) => CreateEmbed::new()
            .title("Changes Saved")
            .description("Your changes have been saved successfully.")
            .color(utils::bot_color(&ctx).await),
        Err(EditError::ValidationError(errors)) => CreateEmbed::new()
            .title("Validation Error")
            .description(errors.join("\n"))
            .color(Color::RED),
        Err(EditError::SerenityError(err)) => return Err(AppError::from(err)),
        Err(EditError::RepositoryError(err)) => return Err(AppError::from(err)),
    };

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
enum EditError<E> {
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError<E>),
    #[error("Validation error")]
    ValidationError(Vec<String>),
}

async fn edit_user<R>(
    ctx: AppContext<'_, R, R::BackendError>,
    discord_user_id: u64,
) -> Result<(), EditError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync,
{
    let mut repo = ctx.data.repository().await?;

    if let Some(mut user) = repo.user_by_discord_user_id(discord_user_id).await? {
        let defaults = EditProfileModal {
            pokemon_go_code: user.pokemon_go_code.clone(),
            pokemon_pocket_code: user.pokemon_pocket_code.clone(),
            switch_code: user.switch_code.clone(),
        };

        let mut data = match EditProfileModal::execute_with_defaults(ctx, defaults).await? {
            Some(data) => data,
            None => return Ok(()),
        };

        data.validate().map_err(EditError::ValidationError)?;

        user = User {
            id: user.id,
            discord_user_id: user.discord_user_id,
            pokemon_go_code: data.pokemon_go_code,
            pokemon_pocket_code: data.pokemon_pocket_code,
            switch_code: data.switch_code,
        };

        repo.update_user(user).await?;
    } else {
        let mut data = match EditProfileModal::execute(ctx).await? {
            Some(data) => data,
            None => return Ok(()),
        };

        data.validate().map_err(EditError::ValidationError)?;

        let new_user = NewUser {
            discord_user_id,
            pokemon_go_code: data.pokemon_go_code,
            pokemon_pocket_code: data.pokemon_pocket_code,
            switch_code: data.switch_code,
        };

        repo.insert_user(new_user).await?;
    }

    Ok(())
}

impl EditProfileModal {
    fn validate(&mut self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if let Some(code) = &self.pokemon_go_code {
            match parse_pokemon_go_code(code) {
                Some(parsed) => self.pokemon_go_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid Pokémon Go friend code.", code)),
            };
        }

        if let Some(code) = &self.pokemon_pocket_code {
            match parse_pokemon_pocket_code(code) {
                Some(parsed) => self.pokemon_pocket_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid Pokémon TCG Pocket friend code.", code)),
            };
        }

        if let Some(code) = &self.switch_code {
            match parse_switch_code(code) {
                Some(parsed) => self.switch_code = Some(parsed),
                None => errors.push(format!("`{}` is not a valid switch friend code.", code)),
            };
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn parse_pokemon_go_code(code: &str) -> Option<String> {
    let mut chars = code.chars().peekable();

    let mut parsed = String::new();

    for i in 0..3 {
        if i > 0 {
            let c = *chars.peek()?;
            if c == '-' || c == ' ' {
                chars.next();
            }
            parsed.push(' ');
        }

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

fn parse_pokemon_pocket_code(code: &str) -> Option<String> {
    let mut chars = code.chars().peekable();

    let mut parsed = String::new();

    for i in 0..4 {
        if i > 0 {
            let c = *chars.peek()?;
            if c == '-' || c == ' ' {
                chars.next();
            }
            parsed.push(' ');
        }

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

fn parse_switch_code(code: &str) -> Option<String> {
    let mut chars = code.chars().map(|c| c.to_ascii_uppercase()).peekable();

    let mut parsed = String::from("SW");

    if *chars.peek()? == 'S' {
        chars.next();
        if chars.next()? != 'W' {
            return None;
        }
    }

    for _ in 0..3 {
        let c = *chars.peek()?;
        if c == '-' || c == ' ' {
            chars.next();
        }
        parsed.push('-');

        for _ in 0..4 {
            let c = chars.next()?;
            if !c.is_numeric() {
                return None;
            }
            parsed.push(c);
        }
    }

    chars.next().is_none().then_some(parsed)
}

/// Show your profile or someone else's.
#[poise::command(slash_command)]
pub async fn show<R: RepositoryProvider + Send + Sync>(
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
