use cipher_core::repository::RepositoryProvider;

use crate::app::AppCommand;

mod ping;
mod profile;

pub fn commands<R>() -> Vec<AppCommand<R, R::BackendError>>
where
    R: RepositoryProvider + Send + Sync + 'static,
{
    vec![
        ping::ping(),
        profile::profile(),
    ]
}
