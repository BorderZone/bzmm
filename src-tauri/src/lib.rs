mod mods;
mod settings;

use mods::{
    delete_mod, disable_mod, download_mod, enable_mod, get_downloaded_mods, get_mods,
    handlers::get_enabled_mods, queue_download, update_mod,
};
use settings::{delete_profile, get_app_version, get_settings, update_profile, update_settings};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            get_settings,
            update_settings,
            update_profile,
            delete_profile,
            get_mods,
            get_downloaded_mods,
            get_enabled_mods,
            download_mod,
            queue_download,
            enable_mod,
            disable_mod,
            update_mod,
            delete_mod,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

