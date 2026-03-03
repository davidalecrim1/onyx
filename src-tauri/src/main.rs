#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod file_tree;
mod global_config;
mod vault;
mod vault_config;

use commands::{
    create_file, create_folder, create_vault, get_default_vault_dir, get_file_tree,
    get_known_vaults, get_settings, load_vault_session_cmd, maximize_window, move_file, open_vault,
    read_file, rename_file, save_settings, save_vault_session_cmd, write_file,
};

fn main() {
    env_logger::init();
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
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
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
