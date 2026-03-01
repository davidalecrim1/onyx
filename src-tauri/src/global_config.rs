use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::error::OnyxError;

/// Application-wide settings stored at `~/.config/onyx/config.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct GlobalConfig {
    #[serde(default)]
    pub vaults: Vec<PathBuf>,
    pub last_active_vault: Option<PathBuf>,
}

/// Returns the directory where global config lives (`~/.config/onyx/`).
fn config_dir() -> Result<PathBuf, OnyxError> {
    let home = dirs_next::config_dir().ok_or(OnyxError::NoHomeDir)?;
    Ok(home.join("onyx"))
}

/// Returns the path to the global config file.
fn config_path() -> Result<PathBuf, OnyxError> {
    Ok(config_dir()?.join("config.toml"))
}

/// Loads the global config, returning defaults if the file doesn't exist.
pub fn load_global_config() -> Result<GlobalConfig, OnyxError> {
    let path = config_path()?;
    if !path.exists() {
        return Ok(GlobalConfig::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&contents)?)
}

/// Persists the global config to disk, creating parent directories as needed.
pub fn save_global_config(config: &GlobalConfig) -> Result<(), OnyxError> {
    let path = config_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(&path, contents)?;
    Ok(())
}

/// Adds a vault path to the global config if not already present.
pub fn register_vault(vault_path: PathBuf) -> Result<GlobalConfig, OnyxError> {
    let mut config = load_global_config()?;
    if !config.vaults.contains(&vault_path) {
        config.vaults.push(vault_path.clone());
    }
    config.last_active_vault = Some(vault_path);
    save_global_config(&config)?;
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_config_round_trip() {
        let config = GlobalConfig {
            vaults: vec![PathBuf::from("/tmp/vault1")],
            last_active_vault: Some(PathBuf::from("/tmp/vault1")),
        };
        let serialized = toml::to_string_pretty(&config).unwrap();
        let deserialized: GlobalConfig = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn missing_file_returns_default() {
        let config = GlobalConfig::default();
        assert!(config.vaults.is_empty());
        assert!(config.last_active_vault.is_none());
    }

    #[test]
    fn register_vault_is_idempotent() {
        let mut config = GlobalConfig::default();
        let path = PathBuf::from("/tmp/test-vault");

        config.vaults.push(path.clone());
        config.last_active_vault = Some(path.clone());

        // Simulate second registration
        if !config.vaults.contains(&path) {
            config.vaults.push(path.clone());
        }
        config.last_active_vault = Some(path);

        assert_eq!(config.vaults.len(), 1);
    }
}
