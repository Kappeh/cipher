use futures::Stream;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use serenity::all::CreateEmbed;
use serenity::futures::StreamExt;

use crate::app::AppCommand;
use crate::app::AppContext;
use crate::app::AppError;
use crate::utils;

async fn autocomplete_command<'a, R> (
    ctx: AppContext<'a, R, R::BackendError>,
    partial: &'a str,
) -> impl Stream<Item = String> + 'a
where
    R: RepositoryProvider,
{
    futures::stream::iter(ctx.data().qualified_command_names())
        .filter(move |name| futures::future::ready(name.contains(partial)))
        .map(|name| name.to_string())
}

/// Show help message.
#[poise::command(slash_command, guild_only)]
pub async fn help<R: RepositoryProvider + Send + Sync>(
    ctx: AppContext<'_, R, R::BackendError>,
    #[rename = "command"]
    #[description = "Command to get help for."]
    #[autocomplete = "autocomplete_command"]
    option_query: Option<String>,
    #[description = "Show hidden commands. Defaults to False."] all: Option<bool>,
    #[description = "Hide reply from other users. Defaults to True."] ephemeral: Option<bool>,
) -> Result<(), AppError<R::BackendError>> {
    let embed = if let Some(query) = &option_query {
        let option_command = poise::find_command(
            &ctx.framework().options.commands,
            query,
            true,
            &mut Vec::new(),
        );

        if let Some((command, _, _)) = option_command {
            command_help_embed(&ctx, command, all.unwrap_or(false)).await
        } else {
            CreateEmbed::new()
                .title("Help")
                .description(format!("Could not find command `/{}`", query))
                .color(utils::bot_color(&ctx).await)
        }
    } else {
        root_help_embed(&ctx, all.unwrap_or(false)).await
    };

    let reply = CreateReply::default()
        .embed(embed)
        .ephemeral(ephemeral.unwrap_or(true));

    ctx.send(reply).await?;

    Ok(())
}

async fn root_help_embed<R>(ctx: &AppContext<'_, R, R::BackendError>, all: bool) -> CreateEmbed
where
    R: RepositoryProvider + Send + Sync,
{
    let mut commands_field_value = String::new();
    for command in &ctx.framework().options.commands {
        if !all && command.hide_in_help || command.slash_action.is_none() {
            continue;
        }

        commands_field_value.push('`');
        commands_field_value.push('/');
        commands_field_value.push_str(&command.name);
        commands_field_value.push('`');

        if let Some(command_description) = &command.description {
            commands_field_value.push_str(" - ");
            commands_field_value.push_str(command_description);
        }

        commands_field_value.push('\n');
    }
    commands_field_value.pop();

    let mut embed = CreateEmbed::new()
        .title("Help")
        .color(utils::bot_color(&ctx).await);

    if !commands_field_value.is_empty() {
        embed = embed.field("Commands", commands_field_value, false);
    }

    embed = embed.field("More", "For information on specific commands `/help <command>`.", false);

    embed
}

async fn command_help_embed<R>(
    ctx: &AppContext<'_, R, R::BackendError>,
    command: &AppCommand<R, R::BackendError>,
    all: bool,
) -> CreateEmbed
where
    R: RepositoryProvider + Send + Sync,
{
    let mut required_parameters_field_value = String::new();
    let mut optional_parameters_field_value = String::new();
    for parameter in &command.parameters {
        let parameters_field_value = match parameter.required {
            true => &mut required_parameters_field_value,
            false => &mut optional_parameters_field_value,
        };

        parameters_field_value.push('`');
        parameters_field_value.push_str(&parameter.name);
        parameters_field_value.push('`');

        if let Some(parameter_description) = &parameter.description {
            parameters_field_value.push_str(" - ");
            parameters_field_value.push_str(parameter_description);
        }

        parameters_field_value.push('\n');
    }
    required_parameters_field_value.pop(); // Removes final newline
    optional_parameters_field_value.pop(); // Removes final newline

    let mut subcommands_field_value = String::new();
    for subcommand in &command.subcommands {
        if !all && subcommand.hide_in_help || command.slash_action.is_none() {
            continue;
        }

        subcommands_field_value.push('`');
        subcommands_field_value.push_str(&subcommand.name);
        subcommands_field_value.push('`');

        if let Some(command_description) = &subcommand.description {
            subcommands_field_value.push_str(" - ");
            subcommands_field_value.push_str(command_description);
        }

        subcommands_field_value.push('\n');
    }
    subcommands_field_value.pop(); // Removes final newline

    let mut embed = CreateEmbed::new()
        .title(format!("Help `/{}`", command.qualified_name))
        .color(utils::bot_color(ctx).await);

    if let Some(category) = &command.category {
        embed = embed.field("Category", category, false);
    }

    if !required_parameters_field_value.is_empty() {
        embed = embed.field("Required Parameters", required_parameters_field_value, false);
    }

    if !optional_parameters_field_value.is_empty() {
        embed = embed.field("Optional Parameters", optional_parameters_field_value, false);
    }

    if !subcommands_field_value.is_empty() {
        embed = embed.field("Subcommands", subcommands_field_value, false);
    }

    if let Some(description) = &command.description {
        embed = embed.description(description);
    }

    embed = embed.field("More", "For information on specific commands `/help <command>`.", false);

    embed
}
