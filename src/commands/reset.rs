// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, Preferences};
use tokio::fs;

use crate::{
    cli::atomic::should_dry_run,
    commands::{Runnable, RunnableInvokeRules},
    context::AppContext,
    domains::{collect, core::get_effective_sys_domain_key, read_current},
    log_cute, log_dry, log_err, log_info, log_warn,
    util::io::{confirm, restart_services},
};

#[derive(Args, Debug)]
pub struct ResetCmd;

#[async_trait]
impl Runnable for ResetCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: false,
            require_sudo: false,
            respect_lock: true,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        let dry_run = should_dry_run();

        log_warn!("This will DELETE all settings defined in your config file.",);
        log_warn!("Settings will be reset to macOS defaults, not to their previous values.",);

        if !confirm("Are you sure you want to continue?") {
            return Ok(());
        }

        let doc = ctx.config.load_as_mut().await?;
        let config_system_domains = collect(&doc).await?;

        for (dom, table) in config_system_domains {
            for (key, _) in table {
                let (eff_dom, eff_key) = get_effective_sys_domain_key(&dom, &key);

                // only delete it if currently set
                if read_current(&eff_dom, &eff_key).await.is_some() {
                    let domain_obj = if eff_dom == "NSGlobalDomain" {
                        Domain::Global
                    } else {
                        Domain::User(eff_dom.clone())
                    };

                    if dry_run {
                        log_dry!("Would reset {eff_dom}.{eff_key} to system default",);
                    } else {
                        match Preferences::delete(domain_obj, &eff_key) {
                            Ok(()) => {
                                log_info!("Reset {eff_dom}.{eff_key} to system default");
                            }
                            Err(e) => {
                                log_err!("Failed to reset {eff_dom}.{eff_key}: {e}");
                            }
                        }
                    }
                } else {
                    log_info!("Skipping {eff_dom}.{eff_key} (not set)",);
                }
            }
        }

        // remove snapshot if present
        let snap_path = ctx.snapshot.path();

        if ctx.snapshot.is_loadable() {
            if dry_run {
                log_dry!("Would remove snapshot at {snap_path:?}",);
            } else if let Err(e) = fs::remove_file(&snap_path).await {
                log_warn!("Failed to remove snapshot: {e}",);
            } else {
                log_info!("Removed snapshot at {snap_path:?}",);
            }
        }

        log_cute!("Reset complete. All configured settings have been removed.",);

        // restart system services if requested
        restart_services().await;

        log_cute!("Reset complete.");

        Ok(())
    }
}
