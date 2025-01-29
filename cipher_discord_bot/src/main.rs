mod app;
mod cli;
mod commands;
mod utils;

#[derive(Debug, thiserror::Error)]
enum MainError {
    #[error(transparent)]
    CliError(#[from] cli::CliError),
    #[error(transparent)]
    CommandError(#[from] cli::command::CommandError)
}

#[tokio::main]
async fn main() -> Result<(), MainError> {
    let c = cli::parse()?;
    env_logger::init();

    c.command.execute().await?;

    Ok(())
}
