use crate::{
    config::{Config, get_config_path},
    snapshot::{Snapshot, get_snapshot_path},
};
use anyhow::{Context, Result};
use tokio::fs;

pub struct AppContext {
    pub config: Config,
    pub snapshot: Snapshot,
}

pub struct AppContextManager;

impl AppContextManager {
    async fn old_snapshot_sync() -> Result<()> {
        let old_home =
            dirs::home_dir().with_context(|| "Could not determine home directory".to_string())?;

        let old_path = old_home.join(".cutler_snapshot");
        let new_path = get_snapshot_path()?;

        if old_path.exists() {
            if new_path.exists() {
                fs::remove_file(&old_path)
                    .await
                    .with_context(|| format!("Failed to delete old snapshot at: {old_path:?}"))?
            } else {
                fs::rename(&old_path, &new_path).await.with_context(|| {
                    format!("Failed to move snapshot from {old_path:?} to {new_path:?}")
                })?;
            }
        }
        Ok(())
    }

    pub async fn sync() -> Result<AppContext> {
        Self::old_snapshot_sync().await?;

        let config_path = get_config_path();
        let config = Config::new(config_path);

        let snapshot_path = get_snapshot_path()?;
        let snapshot = Snapshot::new(snapshot_path);

        Ok(AppContext { config, snapshot })
    }
}
