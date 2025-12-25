// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::Result;
use async_trait::async_trait;
use clap::Args;

use crate::{
    commands::{Runnable, RunnableInvokeRules},
    context::AppContext,
    util::io::open,
};

#[derive(Args, Debug)]
pub struct CookbookCmd;

#[async_trait]
impl Runnable for CookbookCmd {
    fn get_invoke_rules(&self) -> RunnableInvokeRules {
        RunnableInvokeRules {
            do_config_autosync: false,
            require_sudo: false,
            respect_lock: false,
        }
    }

    async fn run(&self, _: &AppContext) -> Result<()> {
        open("https://machlit.github.io/cutler").await
    }
}
