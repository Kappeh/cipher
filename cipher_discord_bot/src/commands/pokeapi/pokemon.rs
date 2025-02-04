use std::time::Duration;

use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use rustemon::Follow;
use serenity::all::ComponentInteractionCollector;
use serenity::all::CreateActionRow;
use serenity::all::CreateButton;
use serenity::all::CreateEmbed;
use serenity::all::CreateInteractionResponse;
use uuid::Uuid;

use crate::app::AppContext;
use crate::app::AppError;

/// Get information about Pokémon.
#[poise::command(
    slash_command,
    guild_only,
    subcommands(
        "list",
        "search",
    ),
)]
pub async fn pokemon<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}

/// List all of the Pokémon.
#[poise::command(slash_command, guild_only)]
async fn list<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[rename = "page"]
    #[description = "The page to show. Default is 1."]
    #[min = 1]
    option_page_number: Option<usize>,
    #[rename = "amount"]
    #[description = "The number of results to show per page. Default is 10."]
    #[min = 1]
    #[max = 20]
    option_amount: Option<usize>,
) -> Result<(), AppError<R::BackendError>> {
    let colour = crate::utils::bot_color(&ctx).await;
    let working_embed = CreateEmbed::new()
        .title("Consulting the Pokédex")
        .description("Just a moment...")
        .color(colour);

    let working_reply = CreateReply::default()
        .embed(working_embed)
        .components(vec![])
        .ephemeral(true);

    let reply_handle = ctx.send(working_reply.clone()).await?;

    let rustemon_client = rustemon::client::RustemonClient::default();
    let all = rustemon::pokemon::pokemon::get_all_entries(&rustemon_client).await?;

    let amount = option_amount.unwrap_or(10);
    let max_page_number = (all.len() + amount - 1) / amount;
    let mut page_number = option_page_number.unwrap_or(1).min(max_page_number);

    loop {
        let page_index = page_number - 1;
        let lower = page_index * amount;
        let upper = (lower + amount).min(all.len());

        let mut embed_description = String::new();
        for pokemon in &all[lower..upper] {
            let pokemon = pokemon.follow(&rustemon_client).await?;
            embed_description.push_str(&format!("{} #{}\n", pokemon.name, pokemon.id));
        }
        embed_description.pop();

        let embed = CreateEmbed::new()
            .title(format!("Pokémon Page {}/{}", page_number, max_page_number))
            .description(embed_description)
            .color(colour);

        let previous_button_id = Uuid::new_v4().to_string();
        let previous_button = CreateButton::new(&previous_button_id)
            .label("Previous")
            .disabled(page_index <= 0);

        let next_button_id = Uuid::new_v4().to_string();
        let next_button = CreateButton::new(&next_button_id)
            .label("Next")
            .disabled(page_number >= max_page_number);

        let action_row = CreateActionRow::Buttons(vec![previous_button, next_button]);

        let reply = CreateReply::default()
            .embed(embed)
            .components(vec![action_row])
            .ephemeral(true);

        reply_handle.edit(ctx.into(), reply).await?;

        let collector = ComponentInteractionCollector::new(ctx)
            .author_id(ctx.author().id)
            .channel_id(ctx.channel_id())
            .timeout(Duration::from_secs(60));

        let mci = match collector.await {
            Some(mci) => mci,
            None => break,
        };

        if mci.data.custom_id == previous_button_id {
            page_number -= 1;
        }

        if mci.data.custom_id == next_button_id {
            page_number += 1;
        }

        mci.create_response(ctx, CreateInteractionResponse::Acknowledge).await?;

        reply_handle.edit(ctx.into(), working_reply.clone()).await?;
    }

    reply_handle.delete(ctx.into()).await?;

    Ok(())
}

/// Search for a Pokémon by name.
#[poise::command(slash_command, guild_only)]
async fn search<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[description = "The name of the Pokémon"] name: String,
) -> Result<(), AppError<R::BackendError>> {
    let rustemon_client = rustemon::client::RustemonClient::default();

    let colour = crate::utils::bot_color(&ctx).await;

    let found = match rustemon::pokemon::pokemon::get_by_name(&name, &rustemon_client).await {
        Ok(found) => found,
        Err(err) => {
            log::error!("{}", err);

            let embed = CreateEmbed::new()
                .title("Could not find requested Pokémon")
                .description(
                    "Either no Pokémon exists with that name or a network error has occurred.",
                )
                .color(colour);

            let reply = CreateReply::default().embed(embed).ephemeral(true);

            ctx.send(reply).await?;

            return Ok(());
        }
    };

    let forms = found
        .forms
        .iter()
        .map(|f| f.name.clone())
        .collect::<Vec<_>>()
        .join(", ");

    let mut embed = CreateEmbed::new()
        .title(format!("{} #{}", found.name, found.id))
        .field("Forms", forms, false)
        .color(colour);

    if let Some(sprite) = found.sprites.front_default {
        embed = embed.thumbnail(sprite);
    }

    let reply = CreateReply::default().embed(embed).ephemeral(true);

    ctx.send(reply).await?;

    Ok(())
}
