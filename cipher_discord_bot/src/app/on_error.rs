use cipher_core::repository::RepositoryError;
use cipher_core::repository::RepositoryProvider;
use poise::CreateReply;
use poise::FrameworkError;
use serenity::all::Color;
use serenity::all::CreateEmbed;
use serenity::all::Permissions;

use super::AppData;
use super::AppError;

struct ErrorMessage {
    embed: Option<ErrorEmbed>,
    log: Option<ErrorLog>,
}

struct ErrorEmbed {
    title: String,
    description: String,
}

struct ErrorLog {
    message: String,
    log_level: log::Level,
}

impl ErrorMessage {
    fn new<T, D, M>(title: T, description: D, message: M, log_level: log::Level) -> Self
    where
        T: ToString,
        D: ToString,
        M: ToString,
    {
        Self {
            embed: Some(ErrorEmbed { title: title.to_string(), description: description.to_string() }),
            log: Some(ErrorLog { message: message.to_string(), log_level }),
        }
    }
}

#[rustfmt::skip]
pub async fn on_error<R>(framework_error: FrameworkError<'_, AppData<R>, AppError<R::BackendError>>)
where
    R: RepositoryProvider,
{
    let ctx = framework_error.ctx();

    let error_data = ErrorMessage::from(framework_error);

    match error_data.log {
        Some(ErrorLog { message, log_level: log::Level::Trace }) => log::trace!("{}", message),
        Some(ErrorLog { message, log_level: log::Level::Debug }) => log::debug!("{}", message),
        Some(ErrorLog { message, log_level: log::Level::Info }) => log::info!("{}", message),
        Some(ErrorLog { message, log_level: log::Level::Warn }) => log::warn!("{}", message),
        Some(ErrorLog { message, log_level: log::Level::Error }) => log::error!("{}", message),
        None => {},
    }

    if let Some((ctx, ErrorEmbed { title, description })) = ctx.zip(error_data.embed) {
        let embed = CreateEmbed::new()
            .title(title)
            .description(description)
            .color(Color::RED);

        let reply = CreateReply::default()
            .embed(embed)
            .ephemeral(true);

        ctx.send(reply).await.ok();
    }
}

impl<'a, R> From<FrameworkError<'a, AppData<R>, AppError<R::BackendError>>> for ErrorMessage
where
    R: RepositoryProvider,
{
    fn from(value: FrameworkError<'a, AppData<R>, AppError<R::BackendError>>) -> ErrorMessage {
        use FrameworkError as F;

        fn format_permissions(permissions: Permissions) -> String {
            permissions
                .iter_names()
                .map(|(name, _)| format!("`{}`", name))
                .collect::<Vec<_>>()
                .join(", ")
        }

        #[allow(unused)]
        match value {
            F::Setup { error, framework, data_about_bot, ctx, .. } => error.into(),
            F::EventHandler { error, ctx, event, framework, .. } => error.into(),
            F::Command { error, ctx, .. } => error.into(),
            F::SubcommandRequired { ctx } => ErrorMessage::new(
                "Expected Subcommand",
                format!("Expected subcommand for `/{}`. Please contact a bot administrator to review the logs for further details.", ctx.command().qualified_name),
                format!("expected subcommand for `/{}`. this error has likely occurred due to the application commands not being synced with discord.", ctx.command().qualified_name),
                log::Level::Error,
            ),
            F::CommandPanic { ctx, payload, .. } => ErrorMessage::new(
                "A Panic Has Occurred",
                "A panic has occurred during command execution. Please contact a bot administrator to review the logs for further details.",
                format!("panic in command `{}`: {}", ctx.command().qualified_name, payload.unwrap_or_else(|| "Unknown panic".to_string())),
                log::Level::Error,
            ),
            F::ArgumentParse { error, input, ctx, .. } => ErrorMessage::new(
                "Argument Parse Error",
                "Failed to parse argument in command. Please contact a bot administrator to review the logs for further details.",
                format!("failed to parse argument in command `{}` on input {:?}", ctx.command().qualified_name, input),
                log::Level::Error,
            ),
            F::CommandStructureMismatch { description, ctx, .. } => ErrorMessage::new(
                "Command Structure Mismatch",
                "Unexpected application command structure. Please contact a bot administrator to review the logs for further details.",
                format!("unexpected application command structure in command `{}`: {}", ctx.command.qualified_name, description),
                log::Level::Error,
            ),
            F::CooldownHit { remaining_cooldown, ctx, .. } => ErrorMessage::new(
                "Cooldown Hit",
                format!("You can't use that command right now. Try again in {}.", humantime::format_duration(remaining_cooldown)),
                format!("cooldown hit in command `{}` ({:?} remaining)", ctx.command().qualified_name, remaining_cooldown),
                log::Level::Info,
            ),
            F::MissingBotPermissions { missing_permissions, ctx, .. } => ErrorMessage::new(
                "Insufficient Bot Permissions",
                format!("The bot is missing the following permissions: {}.", format_permissions(missing_permissions)),
                format!("bot is missing permissions ({}) to execute command `{}`", missing_permissions, ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::MissingUserPermissions { missing_permissions, ctx, .. } => ErrorMessage::new(
                "Insufficient User Permissions",
                missing_permissions
                    .map(|p| format!("You are missing the following permissions: {}.", format_permissions(p)))
                    .unwrap_or_else(|| "Failed to get user permissions.".to_string()),
                format!("user is or may be missing permissions ({:?}) to execute command `{}`", missing_permissions, ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::NotAnOwner { ctx, .. } => ErrorMessage::new(
                "Owner Only Command",
                format!("`/{}` can only be used by bot owners.", ctx.command().qualified_name),
                format!("owner-only command `{}` cannot be run by non-owners", ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::GuildOnly { ctx, .. } => ErrorMessage::new(
                "Guild Only Command",
                format!("`/{}` can only be used in a server.", ctx.command().qualified_name),
                format!("guild-only command `{}` cannot be run in DMs", ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::DmOnly { ctx, .. } => ErrorMessage::new(
                "Direct Message Only Command",
                format!("`/{}` can only be used in direct messages.", ctx.command().qualified_name),
                format!("DM-only command `{}` cannot be run in DMs", ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::NsfwOnly { ctx, .. } => ErrorMessage::new(
                "NSFW Only Command",
                format!("`/{}` can only be used in channels marked as NSFW.", ctx.command().qualified_name),
                format!("nsfw-only command `{}` cannot be run in non-nsfw channels", ctx.command().qualified_name),
                log::Level::Info,
            ),
            F::CommandCheckFailed { error, ctx, .. } => error.map(|err| err.into()).unwrap_or_else(|| ErrorMessage::new(
                "Command Check Failed",
                "A pre-command check failed without a reason. Please contact a bot administrator to review the logs for further details.",
                format!("pre-command check for command `{}` either denied access or errored without a reason", ctx.command().qualified_name),
                log::Level::Warn,
            )),
            F::DynamicPrefix { error, ctx, msg, .. } => ErrorMessage::new(
                "Dynamic Prefix Error",
                format!("Dynamic prefix callback error on message {:?}", msg.content),
                format!("dynamic prefix callback error on message {:?}", msg.content),
                log::Level::Error,
            ),
            F::UnknownCommand { ctx, msg, prefix, msg_content, framework, invocation_data, trigger, .. } => ErrorMessage::new(
                "Unknown Command",
                format!("Unknown command `{}`", msg_content),
                format!("unknown command `{}`", msg_content),
                log::Level::Error,
            ),
            F::UnknownInteraction { ctx, framework, interaction, .. } => ErrorMessage::new(
                "Unknown Interaction",
                format!("Unknown interaction `{}`", interaction.data.name),
                format!("unknown interaction `{}`", interaction.data.name),
                log::Level::Error,
            ),
            unknown_error => ErrorMessage::new(
                "Unexpected Error",
                "An unexpected error has occurred. Please contact a bot administrator to review the logs for further details.",
                format!("unknown error: {}", unknown_error),
                log::Level::Error,
            ),
        }
    }
}

impl<E> From<AppError<E>> for ErrorMessage
where
    E: std::error::Error,
{
    fn from(value: AppError<E>) -> ErrorMessage {
        use AppError as A;
        use serenity::Error as S;

        #[allow(unused)]
        match value {
            A::SerenityError(S::Decode(msg, value)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                msg,
                log::Level::Error,
            ),
            A::SerenityError(S::Format(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::Io(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::Json(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::Model(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::ExceededLimit(_, _)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                "input exceeded a limit",
                log::Level::Error,
            ),
            A::SerenityError(S::NotInRange(_, _, _, _)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                "input is not in the specified range",
                log::Level::Error,
            ),
            A::SerenityError(S::Other(msg)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                msg,
                log::Level::Error,
            ),
            A::SerenityError(S::Url(msg)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                msg,
                log::Level::Error,
            ),
            A::SerenityError(S::Client(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::Gateway(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(S::Http(http_error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                http_error,
                log::Level::Error,
            ),
            A::SerenityError(S::Tungstenite(error)) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                error,
                log::Level::Error,
            ),
            A::SerenityError(unknown_error) => ErrorMessage::new(
                "Internal Error",
                "Please contact a bot administrator to review the logs for further details.",
                format!("unknown error: {}", unknown_error),
                log::Level::Error,
            ),

            A::RepositoryError(RepositoryError(error)) => ErrorMessage::new(
                "Repository Backend Error",
                "Please contact a bot administrator to review the logs for further details.",
                format!("repository backend error: {}", error),
                log::Level::Error,
            ),

            A::RustemonError(error) => ErrorMessage::new(
                "PokéAPI Error",
                "Failed to get resource from Pokémon.",
                format!("failed to get resource from PokéAPI: {}", error),
                log::Level::Warn,
            ),

            A::StaffOnly { command_name } => ErrorMessage::new(
                "Staff Only Command",
                format!("`/{}` can only be used by staff.", command_name),
                format!("staff-only command `{}` cannot be run by non-staff users", command_name),
                log::Level::Info,
            ),
            A::UnknownCacheOrHttpError => ErrorMessage::new(
                "Unknown Cache or Http Error",
                "Failed to get resource.",
                "cache lookup or http request failed",
                log::Level::Warn,
            )
        }
    }
}
