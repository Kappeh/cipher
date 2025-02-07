use std::fmt::Display;

use profile_repository::ProfileRepository;
use staff_role_repository::StaffRoleRepository;
use user_repository::UserRepository;

pub mod profile_repository;
pub mod staff_role_repository;
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
    Self: ProfileRepository<BackendError = <Self as Repository>::BackendError>,
    Self: StaffRoleRepository<BackendError = <Self as Repository>::BackendError>,
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
