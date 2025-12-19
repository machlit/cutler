// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result};
use std::{path::PathBuf, sync::OnceLock};

use crate::config::get_config_path;

/// The static snapshot path to use throughout each command run.
/// This is to make sure that accidental variable changes don't alter the snapshot being written.
static SNAP_PATH: OnceLock<PathBuf> = OnceLock::new();

/// Returns the path to the snapshot file.
pub fn get_snapshot_path() -> Result<PathBuf> {
    if let Some(cached) = SNAP_PATH.get().cloned() {
        return Ok(cached);
    }

    let config_parent = get_config_path()
        .parent()
        .with_context(|| "Could not determine config parent directory".to_string())?
        .to_path_buf();

    let new_path = config_parent.join("snapshot.json");

    SNAP_PATH.set(new_path.clone()).ok();
    Ok(new_path)
}
