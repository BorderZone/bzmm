// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod logging;
mod mods;
mod settings;

use mods::handlers::get_enabled_mods;
use mods::{
    cancel_download, delete_mod, disable_mod, download_mod, enable_mod, get_downloaded_mods,
    get_mods, queue_download, update_mod,
};
use settings::{delete_profile, get_app_version, get_settings, update_profile, update_settings};

use crate::logging::init_logging;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            init_logging(app.handle()).expect("failed to initialize logging");
            tracing::info!("App Starting up");
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
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
            cancel_download,
            enable_mod,
            disable_mod,
            update_mod,
            delete_mod,
            get_app_version
        ])
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            tracing::error!("Tauri could not start up due to {}", e);
            panic!("Tauri failed to start");
        });
}
