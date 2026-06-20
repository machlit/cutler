use anyhow::Result;
use async_trait::async_trait;

pub mod apply;
pub mod brew;
pub mod check_update;
pub mod completion;
pub mod config;
pub mod cookbook;
pub mod exec;
pub mod fetch;
pub mod init;
pub mod lock;
pub mod reset;
pub mod self_update;
pub mod status;
pub mod unapply;
pub mod unlock;

pub use apply::ApplyCmd;
pub use brew::{backup::BrewBackupCmd, install::BrewInstallCmd};
pub use check_update::CheckUpdateCmd;
pub use completion::CompletionCmd;
pub use config::ConfigCmd;
pub use cookbook::CookbookCmd;
pub use exec::ExecCmd;
pub use fetch::FetchCmd;
pub use init::InitCmd;
pub use lock::LockCmd;
pub use reset::ResetCmd;
pub use self_update::SelfUpdateCmd;
pub use status::StatusCmd;
pub use unapply::UnapplyCmd;
pub use unlock::UnlockCmd;

use crate::{cli::Command, context::AppContext};

/// A common trait for cutler commands.
///
/// This trait must be implemented for all commands inside cutler since in
/// src/cli/args.rs, the trait is used for passing down the same callable.
#[async_trait]
pub trait Runnable {
    /// The primary run function for the command.
    async fn run(&self, ctx: &AppContext) -> Result<()>;

    /// Set governing rules for the runnable command.
    fn set_invoke_rules(&self) -> RunnableInvokeRules;
}

impl Command {
    pub async fn run(&self, ctx: &AppContext) -> Result<()> {
        match self {
            Command::Apply(apply_cmd) => apply_cmd.run(ctx).await,
            Command::Cookbook(cookbook_cmd) => cookbook_cmd.run(ctx).await,
            Command::Exec(exec_cmd) => exec_cmd.run(ctx).await,
            Command::Init(init_cmd) => init_cmd.run(ctx).await,
            Command::Lock(lock_cmd) => lock_cmd.run(ctx).await,
            Command::Unlock(unlock_cmd) => unlock_cmd.run(ctx).await,
            Command::Unapply(unapply_cmd) => unapply_cmd.run(ctx).await,
            Command::Reset(reset_cmd) => reset_cmd.run(ctx).await,
            Command::Status(status_cmd) => status_cmd.run(ctx).await,
            Command::Brew { command } => match command {
                crate::cli::args::BrewSubcmd::Backup(brew_backup_cmd) => {
                    brew_backup_cmd.run(ctx).await
                }
                crate::cli::args::BrewSubcmd::Install(brew_install_cmd) => {
                    brew_install_cmd.run(ctx).await
                }
            },
            Command::Config(config_cmd) => config_cmd.run(ctx).await,
            Command::CheckUpdate(check_update_cmd) => check_update_cmd.run(ctx).await,
            Command::SelfUpdate(self_update_cmd) => self_update_cmd.run(ctx).await,
            Command::Completion(completion_cmd) => completion_cmd.run(ctx).await,
            Command::Fetch(fetch_cmd) => fetch_cmd.run(ctx).await,
        }
    }

    #[must_use] 
    pub fn get_invoke_rules(&self) -> RunnableInvokeRules {
        match self {
            Command::Apply(apply_cmd) => apply_cmd.set_invoke_rules(),
            Command::Cookbook(cookbook_cmd) => cookbook_cmd.set_invoke_rules(),
            Command::Exec(exec_cmd) => exec_cmd.set_invoke_rules(),
            Command::Init(init_cmd) => init_cmd.set_invoke_rules(),
            Command::Lock(lock_cmd) => lock_cmd.set_invoke_rules(),
            Command::Unlock(unlock_cmd) => unlock_cmd.set_invoke_rules(),
            Command::Unapply(unapply_cmd) => unapply_cmd.set_invoke_rules(),
            Command::Reset(reset_cmd) => reset_cmd.set_invoke_rules(),
            Command::Status(status_cmd) => status_cmd.set_invoke_rules(),
            Command::Brew { command } => match command {
                crate::cli::args::BrewSubcmd::Backup(brew_backup_cmd) => {
                    brew_backup_cmd.set_invoke_rules()
                }
                crate::cli::args::BrewSubcmd::Install(brew_install_cmd) => {
                    brew_install_cmd.set_invoke_rules()
                }
            },
            Command::Config(config_cmd) => config_cmd.set_invoke_rules(),
            Command::CheckUpdate(check_update_cmd) => check_update_cmd.set_invoke_rules(),
            Command::SelfUpdate(self_update_cmd) => self_update_cmd.set_invoke_rules(),
            Command::Completion(completion_cmd) => completion_cmd.set_invoke_rules(),
            Command::Fetch(fetch_cmd) => fetch_cmd.set_invoke_rules(),
        }
    }
}

/// Struct to declare execution rules for the Runnable trait.
pub struct RunnableInvokeRules {
    /// Whether to autosync configuration with cloud before command invocation.
    pub do_config_autosync: bool,
    /// Whether the command requires sudo privileges for execution.
    pub require_sudo: bool,
    /// Whether to respect a locked configuration file.
    pub respect_lock: bool,
}
