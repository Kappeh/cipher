use clap::Parser;

/// Subcommand of the CLI application.
#[derive(Debug, Clone, Parser)]
pub enum Command {
    /// Start the main discord bot application.
    #[command(
        name = "start",
        about,
        long_about = None,
    )]
    Start(super::start::Start),
}

#[derive(Debug, thiserror::Error)]
pub enum CommandError {
    #[error(transparent)]
    StartError(#[from] super::start::StartError),
}

impl Command {
    pub async fn execute(self) -> Result<(), CommandError> {
        match self {
            Command::Start(start) => start.execute().await?,
        }

        Ok(())
    }
}
