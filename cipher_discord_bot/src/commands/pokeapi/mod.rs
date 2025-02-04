use cipher_core::repository::RepositoryProvider;

use crate::app::AppContext;
use crate::app::AppError;

mod pokemon;

/// Query PokéAPI for Pokémon related information.
#[poise::command(
    slash_command,
    guild_only,
    subcommands(
        "pokemon::pokemon",
    ),
)]
pub async fn pokeapi<R: RepositoryProvider + Send + Sync>(
    _ctx: AppContext<'_, R, R::BackendError>,
) -> Result<(), AppError<R::BackendError>> {
    Ok(())
}
