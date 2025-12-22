// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use clap::Args;
use tokio::fs;

use crate::{
    commands::{Runnable, RunnableInvokeRules},
    context::AppContext,
    log_cute, log_warn,
    util::io::confirm,
};

#[derive(Args, Debug)]
pub struct InitCmd;

#[async_trait]
impl Runnable for InitCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: false,
            require_sudo: false,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        if ctx.config.is_loadable() {
            log_warn!(
                "Configuration file already exists at {:?}",
                ctx.config.path()
            );
            if !confirm("Do you want to overwrite it?") {
                bail!("Configuration init aborted.")
            }
        }

        // write TOML template to disk
        // this is not done by create_empty_config
        let default_cfg = include_str!("../../examples/complete.toml");

        fs::create_dir_all(
            ctx.config
                .path()
                .parent()
                .with_context(|| "Failed to initialize new configuration path.".to_string())?,
        )
        .await?;
        fs::write(ctx.config.path(), default_cfg).await?;

        log_cute!(
            "Config created at {:?}, Review and customize it before applying.",
            ctx.config.path()
        );

        Ok(())
    }
}
