// SPDX-License-Identifier: MIT OR Apache-2.0

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::path::PathBuf;
use tokio::fs;

use crate::domains::convert::SerializablePrefValue;

/// A single defaults‑setting change.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub struct SettingState {
    pub domain: String,
    pub key: String,
    pub original_value: Option<SerializablePrefValue>,
}

/// Represents a snapshot.
///
/// This struct has also implemented I/O operations and functions for using across cutler's codebase,
/// in order to properly interact with the snapshot file without much hassle.
#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct LoadedSnapshot {
    pub settings: Vec<SettingState>,
    pub exec_run_count: i32,
    pub version: String,
    pub digest: String,
    #[serde(skip)]
    path: PathBuf,
}

impl LoadedSnapshot {
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Deletes the snapshot.
    pub async fn delete(&self) -> Result<()> {
        fs::remove_file(&self.path)
            .await
            .with_context(|| format!("Could not delete snapshot file {:?}.", &self.path))
    }

    /// Saves the snapshot into the designated path for the instance.
    pub async fn save(&self) -> Result<()> {
        if let Some(dir) = self.path.parent() {
            fs::create_dir_all(dir).await?;
        }

        let json = serde_json::to_string_pretty(self)?;
        fs::write(&self.path, json).await?;
        Ok(())
    }
}

pub struct Snapshot {
    path: PathBuf,
}

impl Snapshot {
    #[must_use]
    pub const fn new(path: PathBuf) -> Self {
        Self { path }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    #[must_use]
    pub fn is_loadable(&self) -> bool {
        !self.path.as_os_str().is_empty() && self.path.try_exists().unwrap_or(false)
    }

    #[must_use]
    pub fn new_empty(&self) -> LoadedSnapshot {
        LoadedSnapshot {
            settings: vec![],
            exec_run_count: 0,
            version: env!("CARGO_PKG_VERSION").to_string(),
            digest: String::new(),
            path: self.path.clone(),
        }
    }

    /// Loads the snapshot from the given path.
    /// If deserialization of the full Snapshot fails, try to deserialize only the `settings` field.
    pub async fn load(&self) -> Result<LoadedSnapshot> {
        if self.is_loadable() {
            let txt = fs::read_to_string(&self.path).await?;
            let snap_result: Result<LoadedSnapshot, _> = serde_json::from_str(&txt);

            match snap_result {
                Ok(mut snap) => {
                    snap.path = self.path.to_owned();
                    Ok(snap)
                }
                Err(e) => {
                    // fallback settings-only deserialization
                    #[derive(Deserialize)]
                    struct SettingsOnly {
                        settings: Vec<SettingState>,
                    }

                    let settings_only_result: Result<SettingsOnly, _> = serde_json::from_str(&txt);

                    match settings_only_result {
                        Ok(settings_only) => {
                            let mut snap = self.new_empty();
                            snap.settings = settings_only.settings;
                            snap.path = self.path.to_owned();
                            Ok(snap)
                        }
                        Err(_) => {
                            bail!("Failed to deserialize snapshot: {e}")
                        }
                    }
                }
            }
        } else {
            bail!("Invalid path, cannot load.")
        }
    }
}
