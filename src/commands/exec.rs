// SPDX-License-Identifier: MIT OR Apache-2.0

use crate::commands::{Runnable, RunnableInvokeRules};

use crate::context::AppContext;
use crate::exec::{ExecMode, run_all, run_one};
use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

#[derive(Args, Debug)]
pub struct ExecCmd {
    /// The command to execute. Defaults to 'all' if not passed.
    #[arg(value_name = "NAME")]
    name: Option<String>,

    /// Executes all declared commands.
    #[arg(short, long, conflicts_with = "flagged")]
    all: bool,

    /// Execute flagged commands only.
    #[arg(short, long, conflicts_with = "all")]
    flagged: bool,
}

#[async_trait]
impl Runnable for ExecCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: true,
            require_sudo: false,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        let loaded_config = ctx.config.load(true).await?;

        let mode = if self.all {
            ExecMode::All
        } else if self.flagged {
            ExecMode::Flagged
        } else {
            ExecMode::Regular
        };

        if let Some(cmd_name) = &self.name {
            run_one(loaded_config, cmd_name).await?;
        } else {
            run_all(loaded_config, mode).await?;
        }

        Ok(())
    }
}
