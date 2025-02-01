use super::RepositoryError;

#[async_trait::async_trait]
pub trait StaffRoleRepository {
    type BackendError: std::error::Error;

    async fn is_staff_role(&mut self, id: u64) -> Result<bool, RepositoryError<Self::BackendError>>;

    async fn staff_roles(&mut self) -> Result<Vec<u64>, RepositoryError<Self::BackendError>>;

    async fn staff_roles_contains(&mut self, ids: &[u64]) -> Result<bool, RepositoryError<Self::BackendError>>;

    async fn set_staff_role(&mut self, id: u64) -> Result<(), RepositoryError<Self::BackendError>>;

    async fn unset_staff_role(&mut self, id: u64) -> Result<(), RepositoryError<Self::BackendError>>;
}
