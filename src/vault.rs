use std::path::{Path, PathBuf};

use crate::error::OnyxError;
use crate::vault_config::{ensure_vault_config, VaultConfig};

/// An open vault rooted at a directory on disk.
#[derive(Debug)]
pub struct Vault {
    pub root: PathBuf,
    pub config: VaultConfig,
}

impl Vault {
    /// Initialises a new vault at the given path, creating `.onyx/config.toml`.
    pub fn create(path: &Path) -> Result<Self, OnyxError> {
        std::fs::create_dir_all(path)?;
        let config = ensure_vault_config(path)?;
        Ok(Self {
            root: path.to_path_buf(),
            config,
        })
    }

    /// Opens an existing directory as a vault, creating config if absent.
    pub fn open(path: &Path) -> Result<Self, OnyxError> {
        let config = ensure_vault_config(path)?;
        Ok(Self {
            root: path.to_path_buf(),
            config,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn create_vault_produces_config() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("new-vault");

        let vault = Vault::create(&vault_path).unwrap();

        assert_eq!(vault.config.name, "new-vault");
        assert!(vault_path.join(".onyx/config.toml").exists());
    }

    #[test]
    fn open_vault_reads_existing_config() {
        let temp = TempDir::new().unwrap();
        let vault_path = temp.path().join("existing");
        std::fs::create_dir_all(&vault_path).unwrap();
        ensure_vault_config(&vault_path).unwrap();

        let vault = Vault::open(&vault_path).unwrap();
        assert_eq!(vault.config.name, "existing");
    }
}
