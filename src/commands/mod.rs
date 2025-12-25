// SPDX-License-Identifier: MIT OR Apache-2.0

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

use crate::context::AppContext;

/// A common trait for cutler commands.
///
/// This trait must be implemented for all commands inside cutler since in
/// src/cli/args.rs, the trait is used for passing down the same callable.
#[async_trait]
pub trait Runnable {
    /// Run the command. The result is implemented using `anyhow::Result` since cutler's internal functions
    /// often propagate an error upto the root error handler.
    async fn run(&self, ctx: &AppContext) -> Result<()>;

    /// Returns a set of governing rules for a Runnable to be run properly.
    fn get_invoke_rules(&self) -> RunnableInvokeRules;
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
