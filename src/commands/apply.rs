// SPDX-License-Identifier: MIT OR Apache-2.0

use std::collections::{HashMap, HashSet};

use crate::{
    cli::atomic::should_dry_run,
    commands::{BrewInstallCmd, Runnable, RunnableInvokeRules},
    config::remote::RemoteConfigManager,
    context::AppContext,
    domains::{
        collect,
        convert::{prefvalue_to_serializable, toml_to_prefvalue},
        core::{self, get_sys_domain_strings},
    },
    exec::{ExecMode, run_all},
    log_cute, log_dry, log_err, log_info, log_warn,
    snapshot::core::SettingState,
    util::{
        io::{confirm, restart_services},
        sha::get_digest,
    },
};
use anyhow::{Context, Result, bail};
use async_trait::async_trait;
use clap::Args;
use defaults_rs::{Domain, PrefValue, Preferences};

use crate::domains::convert::SerializablePrefValue;

#[derive(Args, Debug)]
pub struct ApplyCmd {
    /// The URL to the remote config file.
    #[arg(short, long)]
    url: Option<String>,

    /// Skip executing external commands.
    #[arg(short, long, conflicts_with_all = &["all_cmd", "flagged_cmd"])]
    no_cmd: bool,

    /// Execute all external commands (even flagged ones).
    #[arg(short, long, conflicts_with_all = &["no_cmd", "flagged_cmd"])]
    all_cmd: bool,

    /// Execute flagged external commands only.
    #[arg(short, long, conflicts_with_all = &["all_cmd", "no_cmd"])]
    flagged_cmd: bool,

    /// WARN: Disables domain existence check.
    #[arg(long)]
    no_dom_check: bool,

    /// Invoke `brew install` after applying preferences.
    #[arg(short, long)]
    brew: bool,

    /// When invoking `brew install`, pass the `--force` flag for formula/cask installs.
    #[arg(long)]
    brew_force: bool,

    /// When invoking `brew install`, skip cask installs.
    #[arg(long)]
    brew_skip_cask: bool,

    /// When invoking `brew install`, skip formula installs.
    #[arg(long)]
    brew_skip_formula: bool,
}

/// Represents a preference modification job.
#[derive(Debug)]
struct PreferenceJob {
    domain: String,
    key: String,
    original: Option<SerializablePrefValue>,
    new_value: PrefValue,
}

#[async_trait]
impl Runnable for ApplyCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: true,
            require_sudo: false,
            respect_lock: true,
        }
    }

    async fn run(&self, ctx: &AppContext) -> Result<()> {
        let dry_run = should_dry_run();

        // remote download logic
        if let Some(url) = &self.url {
            if ctx.config.is_loadable()
                && !confirm("Local config exists but a URL was still passed. Proceed?")
            {
                bail!("Aborted apply: --url is passed despite local config.")
            }

            let remote_mgr = RemoteConfigManager::new(url.to_owned());
            remote_mgr.fetch().await?;
            remote_mgr.save().await?;

            log_info!("Remote config downloaded at path: {:?}", ctx.config.path());
        }

        // parse + flatten domains
        let digest = get_digest(ctx.config.path())?;
        let doc = ctx.config.load_as_mut().await?;
        let config_system_domains = collect(&doc).await?;

        // load the old snapshot (if any), otherwise create a new instance
        let mut is_bad_snap: bool = false;

        let snap = if ctx.snapshot.is_loadable() {
            match ctx.snapshot.load().await {
                Ok(snap) => snap,
                Err(e) => {
                    log_warn!("Bad snapshot: {e}; starting new.");
                    log_warn!("When unapplying, all your settings will reset to factory defaults.");

                    is_bad_snap = true;
                    ctx.snapshot.new_empty()
                }
            }
        } else {
            ctx.snapshot.new_empty()
        };

        // ---

        // prepare for applying jobs
        // this is going to be used for applying preferences and also saving them to snapshot
        let mut jobs: Vec<PreferenceJob> = Vec::new();
        let system_domains: HashSet<String> = get_sys_domain_strings()?;

        // turn the old snapshot into a hashmap for a quick lookup
        let mut existing: HashMap<_, _> = snap
            .settings
            .iter()
            .map(|s| ((s.domain.clone(), s.key.clone()), s))
            .collect();

        // system-specific domains
        for (dom, table) in config_system_domains {
            for (key, toml_value) in table {
                let (eff_dom, eff_key) = core::get_effective_sys_domain_key(&dom, &key);

                if !self.no_dom_check
                    && eff_dom != "NSGlobalDomain"
                    && !system_domains.contains(&eff_dom)
                {
                    bail!(
                        "Domain \"{eff_dom}\" was not found; cannot write to it. Disable this behavior by passing: --no-dom-check"
                    )
                }

                let current_pref = core::read_current(&eff_dom, &eff_key).await;
                let new_pref = toml_to_prefvalue(&toml_value)?;

                // Compare PrefValues directly instead of strings
                let changed = match &current_pref {
                    Some(current) => current != &new_pref,
                    None => true, // No current value means it's a new setting
                };

                // grab the old snapshot entry if it exists
                let old_entry = existing.get(&(eff_dom.clone(), eff_key.clone())).cloned();

                if changed {
                    existing.remove(&(eff_dom.clone(), eff_key.clone()));

                    // Preserve existing non-null original
                    // otherwise, for brand new keys, capture original from system
                    let original = if let Some(e) = &old_entry {
                        e.original_value.clone()
                    } else if let Some(pref) = current_pref {
                        Some(prefvalue_to_serializable(&pref).with_context(|| {
                            format!(
                                "Failed to serialize current preference value for key '{eff_key}'."
                            )
                        })?)
                    } else {
                        None
                    };

                    jobs.push(PreferenceJob {
                        domain: eff_dom,
                        key: eff_key,
                        new_value: new_pref,
                        original: if is_bad_snap { None } else { original },
                    });
                } else {
                    log_info!("Skipping unchanged {eff_dom} | {eff_key}",);
                }
            }
        }

        if dry_run {
            for job in &jobs {
                log_dry!(
                    "Would apply: {} {} -> {}",
                    job.domain,
                    job.key,
                    job.new_value
                );
            }
        } else {
            let mut applyable_settings_count = 0;

            for job in &jobs {
                let domain_obj = if job.domain == "NSGlobalDomain" {
                    Domain::Global
                } else {
                    Domain::User(job.domain.clone())
                };

                log_info!(
                    "Applying {} | {} -> {} {}",
                    job.domain,
                    job.key,
                    job.new_value.to_string(),
                    if let Some(orig) = &job.original {
                        format!(
                            "[Restorable to {}]",
                            serde_json::to_string(orig).unwrap_or_else(|_| "?".to_string())
                        )
                    } else {
                        String::new()
                    }
                );

                if let Err(e) = Preferences::write(domain_obj, &job.key, job.new_value.clone()) {
                    log_err!(
                        "Failed to apply preference ({} | {}). Error: {}",
                        job.domain,
                        job.key,
                        e
                    );
                } else {
                    applyable_settings_count += 1;
                }
            }

            if applyable_settings_count > 0 {
                log_info!(
                    "Applied {} settings, will restart services.",
                    applyable_settings_count
                );
                restart_services().await;
            }
        }

        // prepare snapshot (old + new)
        let mut new_snap = ctx.snapshot.new_empty();

        for ((_, _), old_entry) in existing {
            new_snap.settings.push(old_entry.clone());
        }

        for job in jobs {
            new_snap.settings.push(SettingState {
                domain: job.domain,
                key: job.key,
                original_value: job.original.clone(),
            });
        }

        // save config digest to snapshot
        new_snap.digest = digest;

        if dry_run {
            log_dry!("Would save snapshot with system preferences.");
        } else {
            new_snap.save().await?;
            log_info!("Logged system preferences change in snapshot.");
        }

        // run brew
        if self.brew {
            BrewInstallCmd {
                force: self.brew_force,
                skip_cask: self.brew_skip_cask,
                skip_formula: self.brew_skip_formula,
            }
            .run(ctx)
            .await?;
        }

        // exec external commands
        if !self.no_cmd {
            let mode = if self.all_cmd {
                ExecMode::All
            } else if self.flagged_cmd {
                ExecMode::Flagged
            } else {
                ExecMode::Regular
            };

            let loaded_config = ctx.config.load().await?;
            let exec_run_count = run_all(loaded_config, mode).await?;

            if dry_run {
                log_dry!("Would save snapshot with external command execution.");
            } else if exec_run_count > 0 {
                new_snap.exec_run_count = exec_run_count;
                new_snap.save().await?;

                log_info!("Logged command execution in snapshot.");
            }
        }

        log_cute!("Applying complete!");

        Ok(())
    }
}
