// SPDX-License-Identifier: MIT OR Apache-2.0

use std::process::exit;

use clap::Parser;
use cutler::autosync::try_auto_sync;

use cutler::cli::Args;
use cutler::cli::atomic::{
    set_accept_all, set_dry_run, set_no_restart_services, set_quiet, set_verbose,
};
use cutler::commands::Runnable;
use cutler::context::AppContextManager;
use cutler::util::sudo::{run_with_noroot, run_with_root};
use cutler::{log_err, log_info};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args = Args::parse();

    // set some of them atomically
    // (described why in util/globals.rs)
    set_accept_all(args.accept_all);
    set_quiet(args.quiet);
    set_verbose(args.verbose);
    set_dry_run(args.dry_run);
    set_no_restart_services(args.no_restart_services);

    // create app context
    let ctx = match AppContextManager::sync().await {
        Ok(ctx) => ctx,
        Err(_) => {
            log_err!("App context failed to initialize for cutler.");
            exit(1);
        }
    };

    // retrieve Runnable from command instance
    let runnable: &dyn Runnable = args.command.as_runnable();
    let rules = runnable.get_invoke_rules();

    // do lock-check and terminate if true
    if rules.respect_lock && ctx.config.is_locked().await {
        log_err!("Config is locked. Run `cutler config unlock` to unlock.");
        exit(1);
    }

    // run remote-sync if command respects
    if args.no_sync {
        log_info!("Skipping remote config autosync.");
    } else if rules.do_config_autosync {
        try_auto_sync(&ctx.config).await;
    }

    // sudo protection
    if let Err(e) = if rules.require_sudo {
        run_with_root().await
    } else {
        run_with_noroot()
    } {
        log_err!("{e}");
        exit(1);
    }

    let result = runnable.run(&ctx).await;

    if let Err(err) = result {
        log_err!("{err}");
        exit(1);
    }
}
