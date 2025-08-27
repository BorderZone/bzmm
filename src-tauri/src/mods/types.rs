use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Mod {
    #[serde(rename(deserialize = "@name"))]
    pub name: String,
    #[serde(rename(deserialize = "@version"))]
    pub version: String,
    #[serde(rename(deserialize = "@url"))]
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub new_version: Option<String>,
    #[serde(default)]
    #[serde(rename(deserialize = "$text"))]
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Category {
    #[serde(rename(deserialize = "@name"))]
    pub name: String,
    #[serde(rename(deserialize = "@sort_order"))]
    pub sort_order: i32,
    #[serde(rename(deserialize = "mod"))]
    pub mods: Vec<Mod>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ModsFile {
    #[serde(rename(deserialize = "category"))]
    pub categories: Vec<Category>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModsResult {
    pub categories: Vec<Category>,
    pub error: Option<String>,
}

#[derive(Debug, Error)]
pub enum ModError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("XML parsing failed: {0}")]
    ParseError(#[from] quick_xml::de::DeError),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Settings error: {0}")]
    SettingsError(String),

    #[error("Tauri error: {0}")]
    TauriError(#[from] tauri::Error),

    #[error("Invalid mod directory structure: {0}")]
    DirectoryStructureError(String),

    #[error("File conflict: {0}")]
    FileConflictError(String),

    #[error("Mod enablement error: {0}")]
    EnablementError(String),

    #[error("Download error: {0}")]
    DownloadError(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),

    #[error("HTTP error: {0}")]
    HttpError(String),
}

impl Mod {
    pub fn new_sideloaded(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            url: None,
            new_version: None,
            description,
        }
    }
    
    pub fn new_deprecated(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            url: None,
            new_version: None,
            description,
        }
    }
}

impl Category {
    pub fn new_sideloaded(mods: Vec<Mod>) -> Self {
        Self {
            name: "Sideloaded".to_string(),
            sort_order: 0, // Will be set by the handler
            mods,
        }
    }
    
    pub fn new_deprecated(mods: Vec<Mod>) -> Self {
        Self {
            name: "Deprecated".to_string(),
            sort_order: -999, // Always first (with an extra safety margin)
            mods,
        }
    }
}

