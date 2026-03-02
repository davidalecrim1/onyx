use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::OnyxError;

/// Per-vault settings stored at `<vault>/.onyx/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultConfig {
    pub name: String,
}

/// UI session state stored at `<vault>/.onyx/session.toml`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct VaultSession {
    #[serde(default)]
    pub open_tabs: Vec<String>,
    pub active_tab: Option<String>,
}

/// Loads the session from `<vault>/.onyx/session.toml`, returning defaults if absent.
pub fn load_vault_session(vault_path: &Path) -> Result<VaultSession, OnyxError> {
    let path = vault_path.join(".onyx/session.toml");
    if !path.exists() {
        return Ok(VaultSession::default());
    }
    let contents = std::fs::read_to_string(&path)?;
    Ok(toml::from_str(&contents)?)
}

/// Persists the session to `<vault>/.onyx/session.toml`.
pub fn save_vault_session(vault_path: &Path, session: &VaultSession) -> Result<(), OnyxError> {
    let onyx_dir = vault_path.join(".onyx");
    std::fs::create_dir_all(&onyx_dir)?;
    let contents = toml::to_string_pretty(session)?;
    std::fs::write(onyx_dir.join("session.toml"), contents)?;
    Ok(())
}

/// Creates the `.onyx/` directory and default config file if they don't exist.
pub fn ensure_vault_config(vault_path: &Path) -> Result<VaultConfig, OnyxError> {
    let onyx_dir = vault_path.join(".onyx");
    let config_path = onyx_dir.join("config.toml");

    if config_path.exists() {
        let contents = std::fs::read_to_string(&config_path)?;
        return Ok(toml::from_str(&contents)?);
    }

    std::fs::create_dir_all(&onyx_dir)?;

    let name = vault_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("vault")
        .to_string();

    let config = VaultConfig { name };
    let contents = toml::to_string_pretty(&config)?;
    std::fs::write(&config_path, contents)?;

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn ensure_vault_config_creates_onyx_dir_and_file() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("my-vault");
        std::fs::create_dir_all(&vault_path).unwrap();

        let config = ensure_vault_config(&vault_path).unwrap();

        assert_eq!(config.name, "my-vault");
        assert!(vault_path.join(".onyx/config.toml").exists());
    }

    #[test]
    fn ensure_vault_config_is_idempotent() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("notes");
        std::fs::create_dir_all(&vault_path).unwrap();

        let first = ensure_vault_config(&vault_path).unwrap();
        let second = ensure_vault_config(&vault_path).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn load_vault_session_returns_default_when_missing() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("vault");
        std::fs::create_dir_all(&vault_path).unwrap();

        let session = load_vault_session(&vault_path).unwrap();

        assert!(session.open_tabs.is_empty());
        assert!(session.active_tab.is_none());
    }

    #[test]
    fn vault_session_round_trip() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("vault");
        std::fs::create_dir_all(&vault_path).unwrap();

        let session = VaultSession {
            open_tabs: vec!["/vault/a.md".to_string(), "/vault/b.md".to_string()],
            active_tab: Some("/vault/b.md".to_string()),
        };

        save_vault_session(&vault_path, &session).unwrap();
        let loaded = load_vault_session(&vault_path).unwrap();

        assert_eq!(session, loaded);
    }
}
