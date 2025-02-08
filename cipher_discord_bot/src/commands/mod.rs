use cipher_core::repository::RepositoryProvider;

use crate::app::AppCommand;

mod about;
mod help;
mod pokeapi;
mod profile;

pub fn commands<R>() -> Vec<AppCommand<R, R::BackendError>>
where
    R: RepositoryProvider + Send + Sync + 'static,
{
    vec![
        about::about(),
        help::help(),
        pokeapi::pokeapi(),
        profile::profile(),
        profile::cmu_profile_show(),
    ]
}

pub fn qualified_command_names<R>(commands: &[AppCommand<R, R::BackendError>]) -> Vec<String>
where
    R: RepositoryProvider,
{
    let mut prefix = String::new();
    let mut names = Vec::new();
    qualified_command_names_inner(&mut prefix, commands, &mut names);
    names
}

fn qualified_command_names_inner<R>(prefix: &mut String, commands: &[AppCommand<R, R::BackendError>], names: &mut Vec<String>)
where
    R: RepositoryProvider,
{
    for command in commands {
        if command.slash_action.is_none() {
            // Skip context-menu-only commands
            continue;
        }

        names.push(format!("{}{}", prefix, command.qualified_name.clone()));

        if !command.subcommands.is_empty() {
            let old_len = prefix.len();
            prefix.push_str(&command.qualified_name);
            prefix.push(' ');
            qualified_command_names_inner(prefix, &command.subcommands, names);
            prefix.truncate(old_len);
        }
    }
}
