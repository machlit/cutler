// SPDX-License-Identifier: MIT OR Apache-2.0

use async_trait::async_trait;
use clap::Args;

use anyhow::{Result, bail};

use crate::{
    cli::atomic::should_dry_run,
    commands::{Runnable, RunnableInvokeRules},
    config::ConfigCoreMethods,
    context::AppContext,
    log_dry,
};

#[derive(Debug, Args)]
pub struct UnlockCmd;

#[async_trait]
impl Runnable for UnlockCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: false,
            require_sudo: true,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        if !ctx.config.is_loadable() {
            bail!("Cannot find a configuration to unlock in the first place.")
        }

        let mut document = ctx.config.load_as_mut(false).await?;
        let dry_run = should_dry_run();

        if !document
            .get("lock")
            .and_then(toml_edit::Item::as_bool)
            .unwrap_or(false)
        {
            bail!("Already unlocked.")
        } else if dry_run {
            log_dry!("Would unlock config file.");
            return Ok(());
        }

        document.remove("lock");
        document.save(ctx.config.path()).await?;

        Ok(())
    }
}
