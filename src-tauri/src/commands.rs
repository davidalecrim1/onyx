use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::file_tree::{scan_file_tree, FileTreeEntry};
use crate::global_config::{load_global_config, register_vault};
use crate::vault::Vault;

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
