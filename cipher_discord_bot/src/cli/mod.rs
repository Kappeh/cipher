use std::fmt::Debug;
use std::path::PathBuf;

use clap::Parser;
use clap::ValueEnum;
use command::Command;
use secrecy::SecretString;
use url::Url;

pub mod command;
pub mod start;

#[derive(Debug, thiserror::Error)]
pub enum CliError {
    #[error(transparent)]
    Dotenvy(#[from] dotenvy::Error),
}

/// Parses command line arguments.
pub fn parse() -> Result<Cli, CliError> {
    // First phase: If a dotenv file is specified, load values from it.
    // This allows values in the dotenv file to be used when parsing other arguments.
    let dotenv = Dotenv::parse();
    if let Some(dotenv_path) = &dotenv.path {
        dotenvy::from_path_override(dotenv_path)?;
    }

    // Second phase: Parse CLI options using the `Cli` parser.
    let mut cli = Cli::parse();

    // The `Cli` parser contains the `Dotenv` parser and it may parse different results in different phases.
    // It is replaced here to ensure the `Cli` instance reflects the original dotenv configuration.
    cli.dotenv = dotenv;

    return Ok(cli);
}

/// Main command line interface for the librarian application.
///
/// This struct combines the dotenv configuration with the subcommands
/// that the CLI application supports.
///
/// The help and version flags and subcommands are explicitly enabled
/// because `Dotenv` disables them and `Cli` requires them to be enabled.
#[derive(Debug, Parser)]
#[command(
    name = "cipher",
    about,
    version,
    long_about = None,
    ignore_errors = false,
    disable_help_flag = false,
    disable_help_subcommand = false,
    disable_version_flag = false,
)]
pub struct Cli {
    /// Configuration for loading environment variables from a dotenv file.
    #[command(flatten)]
    pub dotenv: Dotenv,

    /// The command to be executed as part of the CLI application.
    #[command(subcommand)]
    pub command: Command,
}

/// Configuration for loading environment variables from a dotenv file.
///
/// This struct is used to specify the path to a dotenv file from which
/// environment variables can be loaded. The default value is `.env`.
///
/// The help and version flags and subcommands are disabled because
/// they are handled by `Cli` which is parsed after `Dotenv`.
#[derive(Debug, Clone, Parser)]
#[command(
    ignore_errors = true,
    disable_help_flag = true,
    disable_help_subcommand = true,
    disable_version_flag = true
)]
pub struct Dotenv {
    /// The path to the dotenv file.
    #[arg(
        name = "path",
        short = None,
        long = "dotenv",
        env = "DOTENV",
    )]
    pub path: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DatabaseDialect {
    #[cfg(feature = "mysql")]
    Mysql,
    #[cfg(feature = "postgres")]
    Postgres,
    #[cfg(feature = "sqlite")]
    Sqlite,
}

impl From<DatabaseDialect> for cipher_database::DatabaseDialect {
    fn from(value: DatabaseDialect) -> Self {
        use cipher_database::DatabaseDialect as Dialect;
        match value {
            #[cfg(feature = "mysql")]
            DatabaseDialect::Mysql => Dialect::Mysql,
            #[cfg(feature = "postgres")]
            DatabaseDialect::Postgres => Dialect::Postgres,
            #[cfg(feature = "sqlite")]
            DatabaseDialect::Sqlite => Dialect::Sqlite,
        }
    }
}

/// Credentials required to establish a database connection.
#[derive(Clone, Debug, Parser)]
pub struct DatabaseCredentials {
    /// The dialect of the database to connect to.
    #[arg(
        short = None,
        long = "database-dialect",
        env = "DATABASE_DIALECT",
    )]
    pub dialect: DatabaseDialect,

    /// The URL of the database to connect to. This should include the
    /// necessary credentials (username and password) and the database
    /// name, following the format: `dialect://username:password@host:port/database`.
    #[arg(
        short = None,
        long = "database-url",
        env = "DATABASE_URL",
        hide_env_values(true),
    )]
    pub url: SecretString,
}

/// Credentials required to authenticate a bot with Discord.
#[derive(Clone, Debug, Parser)]
pub struct DiscordCredentials {
    /// The token used to authenticate the bot with Discord.
    #[arg(
        short = None,
        long = "bot-token",
        env = "BOT_TOKEN",
        hide_env_values(true),
    )]
    pub bot_token: SecretString,
}

/// Information about the application
#[derive(Clone, Debug, Parser)]
pub struct AppInfo {
    /// The displayed name of the application
    #[arg(
        short = None,
        long = "app-name",
        env = "APP_NAME",
        default_value = env!("CARGO_PKG_NAME"),
    )]
    pub name: String,

    // The displayed version of the application
    #[arg(
        short = None,
        long = "app-version",
        env = "APP_VERSION",
        default_value = env!("CARGO_PKG_VERSION"),
    )]
    pub version: String,

    /// The displayed description of the application
    #[arg(
        short = None,
        long = "app-description",
        env = "APP_DESCRIPTION",
        default_value = env!("CARGO_PKG_DESCRIPTION"),
    )]
    pub description: String,

    /// The displayed about title of the application
    #[arg(
        short = None,
        long = "about-title",
        env = "ABOUT_TITLE",
    )]
    pub about_title: String,

    /// The displayed about description of the application
    #[arg(
        short = None,
        long = "about-description",
        env = "ABOUT_DESCRIPTION",
    )]
    pub about_description: String,

    /// The url used to direct users to the source code
    #[arg(
        short = None,
        long = "source-code-url",
        env = "SOURCE_CODE_URL",
    )]
    pub source_code_url: Url,
}
