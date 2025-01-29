use user_repository::UserRepository;

pub mod user_repository;

#[async_trait::async_trait]
pub trait RepositoryProvider {
    type BackendError: std::error::Error;

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
