use cipher_core::repository::staff_role_repository::StaffRoleRepository;
use cipher_core::repository::RepositoryProvider;

use crate::app::AppData;
use crate::app::AppError;

pub async fn is_staff<R>(ctx: poise::Context<'_, AppData<R>, AppError<R::BackendError>>) -> Result<bool, AppError<R::BackendError>>
where
    R: RepositoryProvider,
{
    let roles: Vec<_> = match ctx.author_member().await {
        Some(member) => member.roles.iter().map(|r| r.get()).collect(),
        None => return Ok(false),
    };

    match ctx.data().repository().await?.staff_roles_contains(&roles).await {
        Ok(true) => Ok(true),
        Ok(false) => Err(AppError::StaffOnly { command_name: ctx.command().qualified_name.clone() }),
        Err(err) => Err(AppError::from(err)),
    }
}
