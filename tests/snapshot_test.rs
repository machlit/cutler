// SPDX-License-Identifier: MIT OR Apache-2.0

#[cfg(test)]
mod tests {
    use cutler::{
        domains::convert::SerializablePrefValue,
        snapshot::core::{LoadedSnapshot, SettingState, Snapshot},
    };
    use std::{collections::HashMap, env};
    use tempfile::TempDir;
    use tokio::fs;

    #[tokio::test]
    async fn test_snapshot_basic() {
        // Create a temporary directory for the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.json");

        // Test creation with Snapshot wrapper
        let snapshot = Snapshot::new(snapshot_path.clone());
        let loaded_snapshot = snapshot.new_empty();

        assert_eq!(loaded_snapshot.settings.len(), 0);
        assert_eq!(loaded_snapshot.exec_run_count, 0);
        assert_eq!(loaded_snapshot.version, env!("CARGO_PKG_VERSION"));
        assert_eq!(loaded_snapshot.path(), snapshot_path.as_path());

        // Test setting state
        let setting = SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some(SerializablePrefValue::Integer(36)),
        };
        assert_eq!(setting.domain, "com.apple.dock");
        assert_eq!(setting.key, "tilesize");
        assert_eq!(
            setting.original_value,
            Some(SerializablePrefValue::Integer(36))
        );
    }

    #[tokio::test]
    async fn test_snapshot_serialization() {
        // Create a temporary directory for the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_snapshot.json");

        // Create a Snapshot wrapper and get an empty LoadedSnapshot
        let snapshot = Snapshot::new(snapshot_path.clone());
        let mut loaded_snapshot = snapshot.new_empty();

        // Add multiple settings with different patterns
        loaded_snapshot.settings.push(SettingState {
            domain: "com.apple.dock".to_string(),
            key: "tilesize".to_string(),
            original_value: Some(SerializablePrefValue::Integer(36)),
        });

        loaded_snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "ShowPathbar".to_string(),
            original_value: None, // Test null original value
        });

        loaded_snapshot.settings.push(SettingState {
            domain: "NSGlobalDomain".to_string(),
            key: "ApplePressAndHoldEnabled".to_string(),
            original_value: Some(SerializablePrefValue::Boolean(false)),
        });

        // Save the snapshot
        loaded_snapshot.save().await.unwrap();

        // Verify file exists and has content
        assert!(fs::try_exists(&snapshot_path).await.unwrap());
        let content = fs::read_to_string(&snapshot_path).await.unwrap();
        assert!(content.contains("com.apple.dock"));
        assert!(content.contains("tilesize"));

        // Load the snapshot back using the Snapshot wrapper
        let reloaded_snapshot = snapshot.load().await.unwrap();

        // Verify contents match
        assert_eq!(reloaded_snapshot.settings.len(), 3);

        // Convert to HashMap for easier testing
        let settings_map: HashMap<_, _> = reloaded_snapshot
            .settings
            .iter()
            .map(|s| ((s.domain.clone(), s.key.clone()), s))
            .collect();

        // Check dock setting
        let dock_setting = settings_map
            .get(&("com.apple.dock".to_string(), "tilesize".to_string()))
            .unwrap();
        assert_eq!(
            dock_setting.original_value,
            Some(SerializablePrefValue::Integer(36))
        );

        // Check finder setting (null original)
        let finder_setting = settings_map
            .get(&("com.apple.finder".to_string(), "ShowPathbar".to_string()))
            .unwrap();
        assert_eq!(finder_setting.original_value, None);

        // Check global setting
        let global_setting = settings_map
            .get(&(
                "NSGlobalDomain".to_string(),
                "ApplePressAndHoldEnabled".to_string(),
            ))
            .unwrap();
        assert_eq!(
            global_setting.original_value,
            Some(SerializablePrefValue::Boolean(false))
        );
    }

    #[tokio::test]
    async fn test_snapshot_error_handling() {
        // Test loading from non-existent file
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");
        let snapshot = Snapshot::new(nonexistent_path);
        let result = snapshot.load().await;
        assert!(result.is_err());

        // Test loading from invalid JSON
        let invalid_path = temp_dir.path().join("invalid.json");
        fs::write(&invalid_path, "this is not valid json")
            .await
            .unwrap();

        let invalid_snapshot = Snapshot::new(invalid_path);
        let result = invalid_snapshot.load().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_snapshot_complex_types() {
        // Create a temporary directory for the snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_complex_snapshot.json");

        // Create a Snapshot wrapper and get an empty LoadedSnapshot
        let snapshot = Snapshot::new(snapshot_path.clone());
        let mut loaded_snapshot = snapshot.new_empty();

        // Array type
        loaded_snapshot.settings.push(SettingState {
            domain: "NSGlobalDomain".to_string(),
            key: "exampleArray".to_string(),
            original_value: Some(SerializablePrefValue::Array(vec![
                SerializablePrefValue::Integer(1),
                SerializablePrefValue::Integer(2),
                SerializablePrefValue::Integer(3),
            ])),
        });

        // Dictionary type
        let mut dict = HashMap::new();
        dict.insert("Preview".to_string(), SerializablePrefValue::Boolean(false));
        dict.insert("MetaData".to_string(), SerializablePrefValue::Boolean(true));

        loaded_snapshot.settings.push(SettingState {
            domain: "com.apple.finder".to_string(),
            key: "FXInfoPanesExpanded".to_string(),
            original_value: Some(SerializablePrefValue::Dictionary(dict)),
        });

        // Save the snapshot
        loaded_snapshot.save().await.unwrap();

        // Load the snapshot back using the Snapshot wrapper
        let reloaded_snapshot = snapshot.load().await.unwrap();

        // Verify array
        let array_setting = &reloaded_snapshot.settings[0];
        assert_eq!(array_setting.domain, "NSGlobalDomain");
        assert_eq!(array_setting.key, "exampleArray");
        match &array_setting.original_value {
            Some(SerializablePrefValue::Array(arr)) => {
                assert_eq!(arr.len(), 3);
                assert_eq!(arr[0], SerializablePrefValue::Integer(1));
                assert_eq!(arr[1], SerializablePrefValue::Integer(2));
                assert_eq!(arr[2], SerializablePrefValue::Integer(3));
            }
            _ => panic!("Expected array type"),
        }

        // Verify dictionary
        let dict_setting = &reloaded_snapshot.settings[1];
        assert_eq!(dict_setting.domain, "com.apple.finder");
        assert_eq!(dict_setting.key, "FXInfoPanesExpanded");
        match &dict_setting.original_value {
            Some(SerializablePrefValue::Dictionary(dict)) => {
                assert_eq!(dict.len(), 2);
                assert_eq!(
                    dict.get("Preview"),
                    Some(&SerializablePrefValue::Boolean(false))
                );
                assert_eq!(
                    dict.get("MetaData"),
                    Some(&SerializablePrefValue::Boolean(true))
                );
            }
            _ => panic!("Expected dictionary type"),
        }
    }

    #[tokio::test]
    async fn test_snapshot_is_loadable() {
        // Test is_loadable with non-existent file
        let temp_dir = TempDir::new().unwrap();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");
        let snapshot = Snapshot::new(nonexistent_path);
        assert!(!snapshot.is_loadable());

        // Test is_loadable with existing file
        let existing_path = temp_dir.path().join("existing.json");
        let existing_snapshot = Snapshot::new(existing_path.clone());
        let loaded = existing_snapshot.new_empty();
        loaded.save().await.unwrap();

        assert!(existing_snapshot.is_loadable());
    }

    #[tokio::test]
    async fn test_snapshot_delete() {
        // Create a temporary directory and snapshot
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_delete.json");

        let snapshot = Snapshot::new(snapshot_path.clone());
        let loaded_snapshot = snapshot.new_empty();
        loaded_snapshot.save().await.unwrap();

        // Verify file exists
        assert!(fs::try_exists(&snapshot_path).await.unwrap());

        // Delete the snapshot
        loaded_snapshot.delete().await.unwrap();

        // Verify file no longer exists
        assert!(!fs::try_exists(&snapshot_path).await.unwrap());
    }

    #[tokio::test]
    async fn test_snapshot_fallback_deserialization() {
        // Create a snapshot file with only the settings field (old format)
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_fallback.json");

        let settings_only_json = r#"{
            "settings": [
                {
                    "domain": "com.apple.dock",
                    "key": "tilesize",
                    "original_value": {"Integer": 42}
                }
            ]
        }"#;

        fs::write(&snapshot_path, settings_only_json)
            .await
            .unwrap();

        // Load the snapshot - should use fallback deserialization
        let snapshot = Snapshot::new(snapshot_path.clone());
        let loaded_snapshot = snapshot.load().await.unwrap();

        // Verify settings were loaded
        assert_eq!(loaded_snapshot.settings.len(), 1);
        assert_eq!(loaded_snapshot.settings[0].domain, "com.apple.dock");
        assert_eq!(loaded_snapshot.settings[0].key, "tilesize");
        assert_eq!(
            loaded_snapshot.settings[0].original_value,
            Some(SerializablePrefValue::Integer(42))
        );

        // Verify default values were set for missing fields
        assert_eq!(loaded_snapshot.exec_run_count, 0);
        assert_eq!(loaded_snapshot.version, env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_snapshot_path_method() {
        // Test that path() returns the correct path
        let temp_dir = TempDir::new().unwrap();
        let snapshot_path = temp_dir.path().join("test_path.json");

        let snapshot = Snapshot::new(snapshot_path.clone());
        assert_eq!(snapshot.path(), snapshot_path.as_path());

        let loaded_snapshot = snapshot.new_empty();
        assert_eq!(loaded_snapshot.path(), snapshot_path.as_path());
    }
}
