use std::fmt::Display;

use user_repository::UserRepository;

pub mod user_repository;

#[async_trait::async_trait]
pub trait RepositoryProvider {
    type BackendError: std::error::Error + Send + Sync;

    type Repository<'a>: Repository<BackendError = <Self as RepositoryProvider>::BackendError> + Send + Sync
    where
        Self: 'a;

    async fn get(&self) -> Result<Self::Repository<'_>, RepositoryError<Self::BackendError>>;
}

pub trait Repository
where
    Self: UserRepository<BackendError = <Self as Repository>::BackendError>,
{
    type BackendError: std::error::Error;
}

#[derive(Debug)]
pub struct RepositoryError<E>(pub E);

impl<E> Display for RepositoryError<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
