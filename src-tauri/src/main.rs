#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod file_tree;
mod global_config;
mod tag_index;
mod vault;
mod vault_config;

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

use commands::{
    build_tag_index, create_file, create_folder, create_vault, delete_file, get_default_vault_dir,
    get_file_tree, get_known_vaults, get_last_active_vault, get_settings, get_tags,
    load_vault_session_cmd, maximize_window, move_file, open_vault, open_vault_window,
    open_welcome_window, read_file, rename_file, resolve_asset_path, resolve_wikilink,
    save_settings, save_vault_session_cmd, update_file_tags, write_file,
};
use tag_index::TagIndex;
use tauri_plugin_log::{Target, TargetKind};
use tauri_plugin_prevent_default::Flags;

fn main() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_prevent_default::Builder::new()
                .with_flags(Flags::RELOAD | Flags::CONTEXT_MENU)
                .build(),
        )
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .level_for("tao", log::LevelFilter::Warn)
                .build(),
        )
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(HashMap::<PathBuf, TagIndex>::new()))
        .invoke_handler(tauri::generate_handler![
            create_vault,
            open_vault,
            get_file_tree,
            read_file,
            write_file,
            get_known_vaults,
            maximize_window,
            create_file,
            create_folder,
            load_vault_session_cmd,
            save_vault_session_cmd,
            get_default_vault_dir,
            move_file,
            rename_file,
            get_settings,
            save_settings,
            get_last_active_vault,
            build_tag_index,
            get_tags,
            update_file_tags,
            resolve_wikilink,
            resolve_asset_path,
            open_vault_window,
            open_welcome_window,
            delete_file,
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
