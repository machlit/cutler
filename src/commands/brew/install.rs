// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use tokio::process::Command;

use crate::{
    brew::{
        types::BrewDiff,
        utils::{diff_brew, ensure_brew},
    },
    cli::atomic::should_dry_run,
    commands::{Runnable, RunnableInvokeRules},
    context::AppContext,
    log_cute, log_dry, log_err, log_info, log_warn,
};

#[derive(Debug, Args)]
pub struct BrewInstallCmd {
    /// Use the `--force` flag for Homebrew installs.
    #[arg(short, long)]
    pub force: bool,

    /// Skip cask installs.
    #[arg(long)]
    pub skip_cask: bool,

    /// Skip formula installs.
    #[arg(long)]
    pub skip_formula: bool,
}

#[async_trait]
impl Runnable for BrewInstallCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: true,
            require_sudo: false,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        let dry_run = should_dry_run();
        let loaded_config = ctx.config.load(true).await?;

        let brew_cfg = loaded_config
            .brew
            .ok_or_else(|| anyhow::anyhow!("No [brew] section found in config"))?;

        // ensure homebrew installation
        ensure_brew().await?;

        // check the current brew state, including taps, formulae, and casks
        let brew_diff = match diff_brew(brew_cfg).await {
            Ok(diff) => {
                if !diff.extra_formulae.is_empty() {
                    log_warn!(
                        "Extra installed formulae not in config: {:?}",
                        diff.extra_formulae
                    );
                }
                if !diff.extra_casks.is_empty() {
                    log_warn!(
                        "Extra installed casks not in config: {:?}",
                        diff.extra_casks
                    );
                }
                if !diff.extra_taps.is_empty() {
                    log_warn!("Extra taps not in config: {:?}", diff.extra_taps);
                }
                if !diff.extra_formulae.is_empty() || !diff.extra_casks.is_empty() {
                    log_warn!(
                        "Run `cutler brew backup` to synchronize your config with the system.\n",
                    );
                }
                diff
            }
            Err(e) => {
                log_err!("Could not check Homebrew status: {e}",);

                // If we cannot compare the state, treat as if nothing is missing.
                BrewDiff::default()
            }
        };

        // tap only the missing taps reported by BrewDiff
        if !brew_diff.missing_taps.is_empty() {
            if dry_run {
                for tap in &brew_diff.missing_taps {
                    log_dry!("Would tap {tap}");
                }
            } else {
                for tap in &brew_diff.missing_taps {
                    log_info!("Tapping: {tap}");
                    let status = Command::new("brew").arg("tap").arg(tap).status().await?;

                    if !status.success() {
                        log_err!("Failed to tap: {tap}");
                    }
                }
            }
        } else {
            log_info!("No taps to initialize.")
        }

        if !brew_diff.missing_formulae.is_empty() && !self.skip_formula {
            if dry_run {
                brew_diff.missing_formulae.iter().for_each(|formula| {
                    log_dry!("Would install formula: {formula}");
                });
            } else {
                install_all(brew_diff.missing_formulae, self.force, false).await?;
            }
        } else {
            log_info!("Skipping formulae install.")
        }

        if !brew_diff.missing_casks.is_empty() && !self.skip_cask {
            if dry_run {
                brew_diff.missing_casks.iter().for_each(|formula| {
                    log_dry!("Would install cask: {formula}");
                });
            } else {
                install_all(brew_diff.missing_casks, self.force, true).await?;
            }
        } else {
            log_info!("Skipping casks install.")
        }

        log_cute!("Homebrew sync complete.");

        Ok(())
    }
}

/// Install formulae/casks sequentially.
/// The argument is a vector of argslices, representing the arguments to the `brew install` subcommand.
async fn install_all(install_tasks: Vec<String>, force: bool, cask: bool) -> anyhow::Result<()> {
    if install_tasks.is_empty() {
        return Ok(());
    }

    let task = if cask { "casks" } else { "formulae" };
    log_info!("Installing {task}...");

    let status = if force {
        Command::new("brew")
            .arg("install")
            .arg(format!("--{task}"))
            .arg("--force")
            .args(install_tasks)
            .status()
            .await?
    } else {
        Command::new("brew")
            .arg("install")
            .arg(format!("--{task}"))
            .args(install_tasks)
            .status()
            .await?
    };

    if !status.success() {
        log_err!("Failed to install: {task}");
    }

    Ok(())
}
