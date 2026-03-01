#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod error;
mod file_tree;
mod global_config;
mod vault;
mod vault_config;

use commands::{
    create_file, create_vault, get_file_tree, get_known_vaults, maximize_window, open_vault,
    read_file, write_file,
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
        ])
        .run(tauri::generate_context!())
        .expect("error running tauri app");
}
