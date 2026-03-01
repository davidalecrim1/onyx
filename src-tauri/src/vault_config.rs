use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::OnyxError;

/// Per-vault settings stored at `<vault>/.onyx/config.toml`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VaultConfig {
    pub name: String,
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
}
