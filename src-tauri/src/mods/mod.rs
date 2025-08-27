pub mod downloader;
pub mod download_queue;
pub mod extraction;
pub mod handlers;
pub mod mod_download;
pub mod mod_enablement;
pub mod mod_management;
pub mod mod_utils;
pub mod parser;
pub mod progress;
pub mod sideload;
pub mod deprecated;
pub mod types;
pub mod xml_cache;

// Re-export functions used by main.rs
pub use handlers::{get_mods, get_downloaded_mods};
pub use mod_management::{enable_mod, disable_mod, update_mod, delete_mod};
pub use mod_download::download_mod;
pub use download_queue::{queue_download, cancel_download};