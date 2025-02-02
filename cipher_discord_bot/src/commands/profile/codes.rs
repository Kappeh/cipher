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

/// Manage friend codes.
#[poise::command(
    slash_command,
    guild_only,
    subcommands(
        "edit",
        "overwrite",
    ),
)]
pub async fn codes<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}

#[derive(Debug, poise::Modal)]
#[name = "Edit Friend Codes"]
struct EditCodesModal {
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

impl EditCodesModal {
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

#[derive(Debug, thiserror::Error)]
enum EditError<E> {
    #[error(transparent)]
    SerenityError(#[from] serenity::Error),
    #[error(transparent)]
    RepositoryError(#[from] RepositoryError<E>),
    #[error("Validation error")]
    ValidationError(Vec<String>),
}

async fn execute_edit_codes_modal<R>(
    ctx: AppContext<'_, R, R::BackendError>,
    discord_user_id: u64,
) -> Result<(), EditError<R::BackendError>>
where
    R: RepositoryProvider + Send + Sync,
{
    let mut repo = ctx.data.repository().await?;

    if let Some(mut user) = repo.user_by_discord_user_id(discord_user_id).await? {
        let defaults = EditCodesModal {
            pokemon_go_code: user.pokemon_go_code.clone(),
            pokemon_pocket_code: user.pokemon_pocket_code.clone(),
            switch_code: user.switch_code.clone(),
        };

        let mut data = match EditCodesModal::execute_with_defaults(ctx, defaults).await? {
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
        let mut data = match EditCodesModal::execute(ctx).await? {
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

/// Edit your friend codes.
#[poise::command(slash_command, guild_only)]
async fn edit<R: RepositoryProvider + Send + Sync>(ctx: AppContext<'_, R, R::BackendError>) -> Result<(), AppError<R::BackendError>> {
    let author_id = ctx.author().id.get();

    let embed = match execute_edit_codes_modal(ctx, author_id).await {
        Ok(()) => CreateEmbed::new()
            .title("Changes Saved")
            .description("Your changes have been saved successfully.")
            .color(crate::utils::bot_color(&ctx).await),
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

/// Edit any user's friend codes.
#[poise::command(
    slash_command,
    guild_only,
    hide_in_help,
    check = "crate::checks::is_staff",
)]
async fn overwrite<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[description = "The profile to edit."]
    member: Member,
) -> Result<(), AppError<R::BackendError>> {
    let member_id = member.user.id.get();

    let embed = match execute_edit_codes_modal(ctx, member_id).await {
        Ok(()) => CreateEmbed::new()
            .title("Changes Saved")
            .description("Your changes have been saved successfully.")
            .color(crate::utils::bot_color(&ctx).await),
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
