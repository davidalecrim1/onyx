use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use tauri::State;

use crate::file_tree::{scan_file_tree, FileTreeEntry};
use crate::global_config::{load_global_config, register_vault, save_global_config, GlobalConfig};
use crate::tag_index::TagIndex;
use crate::vault::Vault;
use crate::vault_config::{load_vault_session, save_vault_session, VaultSession};

/// Serializable vault summary returned to the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultInfo {
    pub name: String,
    pub root: String,
}

/// Serializable file tree node returned to the frontend.
#[derive(Debug, Serialize, Deserialize)]
pub struct FileTreeEntryDto {
    pub name: String,
    pub path: String,
    pub is_directory: bool,
    pub depth: usize,
    pub children: Vec<FileTreeEntryDto>,
}

/// Serializable vault entry from the global config.
#[derive(Debug, Serialize, Deserialize)]
pub struct VaultEntry {
    pub name: String,
    pub path: String,
}

fn entry_to_dto(entry: &FileTreeEntry) -> FileTreeEntryDto {
    FileTreeEntryDto {
        name: entry.name.clone(),
        path: entry.path.to_string_lossy().to_string(),
        is_directory: entry.is_directory,
        depth: entry.depth,
        children: entry.children.iter().map(entry_to_dto).collect(),
    }
}

/// Creates a new vault at the given path and registers it in the global config.
#[tauri::command]
pub fn create_vault(path: String) -> Result<VaultInfo, String> {
    let vault_path = PathBuf::from(&path);
    let vault = Vault::create(&vault_path).map_err(|e| e.to_string())?;
    register_vault(vault_path).map_err(|e| e.to_string())?;
    Ok(VaultInfo {
        name: vault.config.name,
        root: vault.root.to_string_lossy().to_string(),
    })
}

/// Opens an existing vault at the given path and registers it in the global config.
#[tauri::command]
pub fn open_vault(path: String) -> Result<VaultInfo, String> {
    let vault_path = PathBuf::from(&path);
    let vault = Vault::open(&vault_path).map_err(|e| e.to_string())?;
    register_vault(vault_path).map_err(|e| e.to_string())?;
    Ok(VaultInfo {
        name: vault.config.name,
        root: vault.root.to_string_lossy().to_string(),
    })
}

/// Returns the file tree for the given vault root path.
#[tauri::command]
pub fn get_file_tree(vault_path: String) -> Result<Vec<FileTreeEntryDto>, String> {
    let root = Path::new(&vault_path);
    let entries = scan_file_tree(root).map_err(|e| e.to_string())?;
    Ok(entries.iter().map(entry_to_dto).collect())
}

/// Reads and returns the UTF-8 contents of a file.
#[tauri::command]
pub fn read_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|e| e.to_string())
}

/// Writes content to a file, creating it if it doesn't exist.
#[tauri::command]
pub fn write_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|e| e.to_string())
}

/// Maximizes the window — called immediately after a vault is opened.
#[tauri::command]
pub fn maximize_window(window: tauri::Window) -> Result<(), String> {
    window.maximize().map_err(|e| e.to_string())
}

/// Creates a new empty file inside the vault and returns its absolute path.
#[tauri::command]
pub fn create_file(vault_path: String, name: String) -> Result<String, String> {
    let path = PathBuf::from(&vault_path).join(&name);
    std::fs::write(&path, "").map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// Creates a new empty directory inside the vault and returns its absolute path.
#[tauri::command]
pub fn create_folder(vault_path: String, name: String) -> Result<String, String> {
    let path = PathBuf::from(&vault_path).join(&name);
    std::fs::create_dir(&path).map_err(|e| e.to_string())?;
    Ok(path.to_string_lossy().to_string())
}

/// Loads the saved session (open tabs, active tab) for the given vault.
#[tauri::command]
pub fn load_vault_session_cmd(vault_path: String) -> Result<VaultSession, String> {
    load_vault_session(Path::new(&vault_path)).map_err(|e| e.to_string())
}

/// Persists the session (open tabs, active tab) for the given vault.
#[tauri::command]
pub fn save_vault_session_cmd(
    vault_path: String,
    open_tabs: Vec<String>,
    active_tab: Option<String>,
) -> Result<(), String> {
    let session = VaultSession {
        open_tabs,
        active_tab,
    };
    save_vault_session(Path::new(&vault_path), &session).map_err(|e| e.to_string())
}

/// Returns the recommended default directory for storing new vaults.
/// On macOS this is the app's iCloud Drive container when available,
/// otherwise falls back to `~/Documents/Onyx`.
#[tauri::command]
pub fn get_default_vault_dir() -> Result<String, String> {
    #[cfg(target_os = "macos")]
    {
        if let Some(home) = dirs_next::home_dir() {
            let icloud = home.join("Library/Mobile Documents/iCloud~md~onyx/Documents");
            if icloud.exists() {
                return Ok(icloud.to_string_lossy().to_string());
            }
        }
    }
    let docs = dirs_next::document_dir()
        .ok_or_else(|| "Cannot determine Documents dir".to_string())?
        .join("Onyx");
    Ok(docs.to_string_lossy().to_string())
}

/// Returns the current application settings from the global config.
#[tauri::command]
pub fn get_settings() -> Result<GlobalConfig, String> {
    load_global_config().map_err(|e| e.to_string())
}

/// Persists a settings change without clobbering the vault list or other fields.
#[tauri::command]
pub fn save_settings(vim_mode: bool) -> Result<(), String> {
    let mut config = load_global_config().map_err(|e| e.to_string())?;
    config.vim_mode = vim_mode;
    save_global_config(&config).map_err(|e| e.to_string())
}

/// Renames a file within its current directory, preserving the extension, and returns the new absolute path.
#[tauri::command]
pub fn rename_file(old_path: String, new_stem: String) -> Result<String, String> {
    let source = PathBuf::from(&old_path);
    let parent = source
        .parent()
        .ok_or_else(|| "Path has no parent directory".to_string())?;
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");
    let new_file_name = if extension.is_empty() {
        new_stem.clone()
    } else {
        format!("{}.{}", new_stem, extension)
    };
    let destination = parent.join(&new_file_name);
    if destination.exists() {
        return Err(format!("A file named '{}' already exists", new_file_name));
    }
    std::fs::rename(&source, &destination).map_err(|e| e.to_string())?;
    Ok(destination.to_string_lossy().to_string())
}

/// Moves a file or directory to a new parent directory, preserving the original name.
#[tauri::command]
pub fn move_file(source_path: String, target_dir: String) -> Result<(), String> {
    let source = PathBuf::from(&source_path);
    let file_name = source
        .file_name()
        .ok_or_else(|| "Invalid source path".to_string())?;
    let destination = PathBuf::from(&target_dir).join(file_name);
    std::fs::rename(&source, &destination).map_err(|e| e.to_string())
}

/// Returns the last active vault, or `None` if no vault has been opened yet or the path is gone.
#[tauri::command]
pub fn get_last_active_vault() -> Result<Option<VaultEntry>, String> {
    let config = load_global_config().map_err(|e| e.to_string())?;
    let Some(path) = config.last_active_vault else {
        return Ok(None);
    };
    let vault = Vault::open(&path).map_err(|e| e.to_string())?;
    Ok(Some(VaultEntry {
        name: vault.config.name,
        path: path.to_string_lossy().to_string(),
    }))
}

/// Returns all known vaults from the global config.
#[tauri::command]
pub fn get_known_vaults() -> Result<Vec<VaultEntry>, String> {
    let config = load_global_config().map_err(|e| e.to_string())?;
    let entries = config
        .vaults
        .iter()
        .map(|vault_path| {
            let name = vault_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("vault")
                .to_string();
            VaultEntry {
                name,
                path: vault_path.to_string_lossy().to_string(),
            }
        })
        .collect();
    Ok(entries)
}

/// Scans the vault and builds the in-memory tag index; called once when a vault is opened.
#[tauri::command]
pub fn build_tag_index(
    vault_path: String,
    state: State<'_, Mutex<Option<TagIndex>>>,
) -> Result<(), String> {
    let index = TagIndex::build(Path::new(&vault_path)).map_err(|e| e.to_string())?;
    *state.lock().map_err(|e| e.to_string())? = Some(index);
    Ok(())
}

/// Returns all known tags across the vault; returns an empty list if the index has not been built.
#[tauri::command]
pub fn get_tags(state: State<'_, Mutex<Option<TagIndex>>>) -> Result<Vec<String>, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    Ok(guard.as_ref().map_or_else(Vec::new, TagIndex::all_tags))
}

/// Updates the tag index for a single file after it has been saved.
#[tauri::command]
pub fn update_file_tags(
    file_path: String,
    content: String,
    state: State<'_, Mutex<Option<TagIndex>>>,
) -> Result<(), String> {
    let mut guard = state.lock().map_err(|e| e.to_string())?;
    if let Some(index) = guard.as_mut() {
        index.update_file(&file_path, &content);
    }
    Ok(())
}

/// Searches the vault for a `.md` file whose stem matches `link_target` (case-insensitive).
/// Returns the absolute path of the first match, or `None` if not found.
#[tauri::command]
pub fn resolve_wikilink(vault_path: String, link_target: String) -> Result<Option<String>, String> {
    let root = Path::new(&vault_path);
    let target_lower = link_target.to_lowercase();
    for entry in walkdir::WalkDir::new(root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            continue;
        }
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        if stem.to_lowercase() == target_lower {
            return Ok(Some(path.to_string_lossy().to_string()));
        }
    }
    Ok(None)
}

/// Resolves a relative asset path to an absolute path for display in the editor.
#[tauri::command]
pub fn resolve_asset_path(
    vault_path: String,
    file_path: String,
    relative_path: String,
) -> Result<String, String> {
    let base = Path::new(&file_path)
        .parent()
        .unwrap_or_else(|| Path::new(&vault_path));
    let resolved = base.join(&relative_path);
    Ok(resolved.to_string_lossy().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn resolve_wikilink_finds_nested_file() {
        let dir = TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("sub")).unwrap();
        std::fs::write(dir.path().join("sub/meeting.md"), "").unwrap();
        let result =
            resolve_wikilink(dir.path().to_string_lossy().into(), "meeting".into()).unwrap();
        assert!(result.unwrap().ends_with("meeting.md"));
    }

    #[test]
    fn resolve_wikilink_is_case_insensitive() {
        let dir = TempDir::new().unwrap();
        std::fs::write(dir.path().join("Meeting.md"), "").unwrap();
        let result =
            resolve_wikilink(dir.path().to_string_lossy().into(), "meeting".into()).unwrap();
        assert!(result.is_some());
    }

    #[test]
    fn resolve_wikilink_returns_none_when_not_found() {
        let dir = TempDir::new().unwrap();
        let result = resolve_wikilink(dir.path().to_string_lossy().into(), "ghost".into()).unwrap();
        assert!(result.is_none());
    }
}
