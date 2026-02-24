use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

// ── Per-vault config (.onyx/config.toml) ────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewModeState {
    LivePreview,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub file_path: PathBuf,
    pub cursor_line: usize,
    pub cursor_col: usize,
    pub view_mode: ViewModeState,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PaneLayout {
    pub file_tree_position: Option<String>,
    pub terminal_position: Option<String>,
    pub file_tree_visible: bool,
    pub terminal_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultConfig {
    #[serde(default)]
    pub open_tabs: Vec<TabState>,
    #[serde(default)]
    pub pane_layout: PaneLayout,
}

impl Default for VaultConfig {
    fn default() -> Self {
        VaultConfig {
            open_tabs: Vec::new(),
            pane_layout: PaneLayout {
                file_tree_position: Some("left".into()),
                terminal_position: Some("right".into()),
                file_tree_visible: true,
                terminal_visible: false,
            },
        }
    }
}

impl VaultConfig {
    /// Loads the vault config from `.onyx/config.toml`, returning the default if absent or unreadable.
    pub fn load(vault_root: &Path) -> Self {
        let path = vault_root.join(".onyx").join("config.toml");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Writes the vault config to `.onyx/config.toml`, creating the directory if needed.
    pub fn save(&self, vault_root: &Path) -> std::io::Result<()> {
        let dir = vault_root.join(".onyx");
        std::fs::create_dir_all(&dir)?;
        let toml = toml::to_string_pretty(self).expect("serialise vault config");
        std::fs::write(dir.join("config.toml"), toml)
    }
}

// ── Global config (~/.config/onyx/config.toml) ──────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    #[serde(default)]
    pub vaults: Vec<VaultEntry>,
    #[serde(default)]
    pub last_active: Vec<PathBuf>,
}

impl GlobalConfig {
    fn config_path() -> PathBuf {
        dirs_next::config_dir()
            .unwrap_or_else(|| PathBuf::from("~/.config"))
            .join("onyx")
            .join("config.toml")
    }

    /// Loads the global config from `~/.config/onyx/config.toml`, returning the default if absent.
    pub fn load() -> Self {
        let path = Self::config_path();
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|s| toml::from_str(&s).ok())
            .unwrap_or_default()
    }

    /// Writes the global config to `~/.config/onyx/config.toml`, creating the directory if needed.
    pub fn save(&self) -> std::io::Result<()> {
        let path = Self::config_path();
        std::fs::create_dir_all(path.parent().unwrap())?;
        let toml = toml::to_string_pretty(self).expect("serialise global config");
        std::fs::write(path, toml)
    }

    /// Registers a vault and promotes it to the front of `last_active`.
    pub fn add_vault(&mut self, name: String, path: PathBuf) {
        if !self.vaults.iter().any(|v| v.path == path) {
            self.vaults.push(VaultEntry { name, path: path.clone() });
        }
        self.last_active.retain(|p| *p != path);
        self.last_active.insert(0, path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn vault_config_round_trips() {
        let config = VaultConfig {
            open_tabs: vec![TabState {
                file_path: "notes.md".into(),
                cursor_line: 3,
                cursor_col: 7,
                view_mode: ViewModeState::LivePreview,
            }],
            pane_layout: PaneLayout::default(),
        };
        let toml = toml::to_string(&config).unwrap();
        let decoded: VaultConfig = toml::from_str(&toml).unwrap();
        assert_eq!(decoded.open_tabs[0].cursor_line, 3);
    }

    #[test]
    fn global_config_round_trips() {
        let config = GlobalConfig {
            vaults: vec![VaultEntry {
                name: "my-notes".into(),
                path: PathBuf::from("/Users/test/notes"),
            }],
            last_active: vec![PathBuf::from("/Users/test/notes")],
        };
        let toml = toml::to_string(&config).unwrap();
        let decoded: GlobalConfig = toml::from_str(&toml).unwrap();
        assert_eq!(decoded.vaults[0].name, "my-notes");
    }

    #[test]
    fn empty_global_config_means_first_launch() {
        let config = GlobalConfig::default();
        assert!(config.last_active.is_empty());
    }
}
