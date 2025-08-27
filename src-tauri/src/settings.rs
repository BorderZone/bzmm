use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub dcs_path: String,
    pub repo_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DarkMode {
    System,
    Light,
    Dark,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Settings {
    pub dark_mode: DarkMode,
    pub download_path: String,
    #[serde(default)]
    pub sideload_path: String,
    pub profiles: Vec<Profile>,
    #[serde(default)]
    pub cached_xml_paths: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsUpdate {
    pub key: String,
    pub value: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppVersion {
    pub version: String,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            dark_mode: DarkMode::System,
            download_path: "".to_string(),
            sideload_path: "".to_string(),
            profiles: vec![],
            cached_xml_paths: vec![],
        }
    }
}

impl Settings {
    fn get_settings_path() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "borderzone", "bzmm")?;
        let config_dir = proj_dirs.config_dir();
        if let Err(e) = fs::create_dir_all(config_dir) {
            eprintln!("Failed to create config directory: {}", e);
            return None;
        }
        Some(config_dir.join("settings.json"))
    }

    pub fn load() -> Result<Self, String> {
        let path = Self::get_settings_path()
            .ok_or_else(|| "Could not determine settings path".to_string())?;

        if path.exists() {
            let content = fs::read_to_string(&path)
                .map_err(|e| format!("Failed to read settings file: {}", e))?;

            serde_json::from_str(&content).map_err(|e| format!("Failed to parse settings: {}", e))
        } else {
            let settings = Settings::default();
            settings.save()?;
            Ok(settings)
        }
    }

    pub fn save(&self) -> Result<(), String> {
        let path = Self::get_settings_path()
            .ok_or_else(|| "Could not determine settings path".to_string())?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize settings: {}", e))?;

        fs::write(&path, content).map_err(|e| format!("Failed to write settings file: {}", e))
    }
}

#[tauri::command]
pub async fn get_settings() -> Result<Settings, String> {
    Settings::load()
}

#[tauri::command]
pub async fn get_app_version() -> Result<AppVersion, String> {
    Ok(AppVersion {
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

#[tauri::command]
pub async fn update_settings(update: SettingsUpdate) -> Result<Settings, String> {
    let mut settings = Settings::load()?;

    match update.key.as_str() {
        "download_path" => settings.download_path = update.value,
        "sideload_path" => settings.sideload_path = update.value,
        _ => return Err("Invalid settings key".to_string()),
    }

    settings.save()?;
    Ok(settings)
}

#[tauri::command]
pub async fn update_profile(index: usize, profile: Profile) -> Result<Settings, String> {
    let mut settings = Settings::load()?;

    if index >= settings.profiles.len() {
        settings.profiles.push(profile);
    } else {
        settings.profiles[index] = profile;
    }

    settings.save()?;
    Ok(settings)
}

#[tauri::command]
pub async fn delete_profile(index: usize) -> Result<Settings, String> {
    let mut settings = Settings::load()?;

    if index >= settings.profiles.len() {
        return Err("Profile index out of bounds".to_string());
    }

    settings.profiles.remove(index);
    settings.save()?;
    Ok(settings)
}
