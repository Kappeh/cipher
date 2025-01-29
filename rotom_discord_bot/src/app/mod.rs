use rotom_core::repository::RepositoryProvider;

use crate::cli::DiscordCredentials;

#[derive(Debug, thiserror::Error)]
pub enum AppError {

}

pub async fn start<R>(_credentials: DiscordCredentials, _repository_provider: R) -> Result<(), AppError>
where
    R: RepositoryProvider + Send + Sync + 'static,
    R::BackendError: Send + Sync,
    for<'a> R::Repository<'a>: Send + Sync,
{
    todo!()
}
