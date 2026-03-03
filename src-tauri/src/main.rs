#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod file_tree;
mod global_config;
mod tag_index;
mod vault;
mod vault_config;

use std::sync::Mutex;

use commands::{
    build_tag_index, create_file, create_folder, create_vault, get_default_vault_dir,
    get_file_tree, get_known_vaults, get_last_active_vault, get_settings, get_tags,
    load_vault_session_cmd, maximize_window, move_file, open_vault, read_file, rename_file,
    resolve_wikilink, save_settings, save_vault_session_cmd, update_file_tags, write_file,
};
use tag_index::TagIndex;

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(Mutex::new(Option::<TagIndex>::None))
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
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
