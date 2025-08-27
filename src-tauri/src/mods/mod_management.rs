use crate::mods::mod_enablement::*;
use crate::mods::mod_utils::*;
use crate::mods::types::ModError;
use crate::settings::Settings;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use tauri::AppHandle;
use tokio::fs;

#[derive(Debug, Serialize, Deserialize)]
pub struct ModResult {
    success: bool,
    message: Option<String>,
}

/// Finds the directory for a given mod, checking the profile-specific download path first, then sideload.
async fn find_mod_dir(
    settings: &Settings,
    mod_name: &str,
    profile_name: &str,
) -> Result<PathBuf, ModError> {
    // Find the profile to get the repo_url
    let profile = settings
        .profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| {
            ModError::SettingsError(format!("Profile '{}' not found for finding mod dir", profile_name))
        })?;

    // Calculate the XML-specific path
    let base_downloads_dir = PathBuf::from(&settings.download_path);
    let mut hasher = Sha256::new();
    hasher.update(profile.repo_url.as_bytes());
    let hash_result = hasher.finalize();
    let repo_hash = format!("{:x}", hash_result);
    let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
    let xml_specific_path = base_downloads_dir.join(repo_hash);
    let mod_path_in_xml_dir = xml_specific_path.join(mod_name);

    println!("Searching for mod '{}' in specific path: {}", mod_name, mod_path_in_xml_dir.display());
    if mod_path_in_xml_dir.is_dir() {
        return Ok(mod_path_in_xml_dir);
    }
    println!("Mod '{}' not found in specific path.", mod_name);

    // If not found in profile-specific dir, check sideload path
    if !settings.sideload_path.is_empty() {
        println!("Checking sideload path: {}", settings.sideload_path);
        let sideload_dir = PathBuf::from(&settings.sideload_path).join(mod_name);
        if sideload_dir.exists() {
            return Ok(sideload_dir);
        }
         println!("Mod '{}' not found in sideload path.", mod_name);
    } else {
        println!("Sideload path is empty, skipping check.");
    }

    Err(ModError::DirectoryStructureError(format!(
        "Could not find mod '{}' in profile-specific download directory ({}) or sideload directory",
        mod_name,
        xml_specific_path.display()
    )))
}

#[tauri::command]
pub async fn enable_mod(mod_name: String, profile_name: String) -> Result<ModResult, String> {
    let result: Result<ModResult, ModError> = async move {
        let settings = Settings::load().map_err(ModError::SettingsError)?;
        let profile = settings
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| ModError::SettingsError("Profile not found".to_string()))?;

        let dcs_dir = PathBuf::from(&profile.dcs_path);
        if !dcs_dir.exists() {
            return Err(ModError::DirectoryStructureError(
                "DCS path does not exist".to_string(),
            ));
        }

        // Pass profile_name to find_mod_dir
        let mod_dir = find_mod_dir(&settings, &mod_name, &profile_name).await?;
        verify_mod_structure(&mod_dir)?;

        let enabled_path = get_enabled_file_path(&mod_dir, &profile_name);
        let enabling_path = get_enabling_file_path(&mod_dir, &profile_name);

        if enabled_path.exists() {
            return Ok(ModResult {
                success: true,
                message: Some("Mod already enabled".to_string()),
            });
        }

        if enabling_path.exists() {
            return Err(ModError::EnablementError(
                "Mod is currently being enabled".to_string(),
            ));
        }

        fs::write(&enabling_path, "")
            .await
            .map_err(ModError::IoError)?;

        let version = get_mod_version(&mod_dir)?;
        let main_subdir = mod_dir.join(&mod_name);

        let process_result =
            process_second_level_dirs(&main_subdir, &dcs_dir, &mod_name, &version, false).await;

        if let Err(ref e) = process_result {
            println!("Error during enablement: {}", e);
            if let Err(cleanup_err) =
                process_second_level_dirs(&main_subdir, &dcs_dir, &mod_name, &version, true).await
            {
                println!("Warning: Cleanup also failed: {}", cleanup_err);
            }
        }

        if let Err(e) = fs::remove_file(&enabling_path).await {
            println!("Warning: Failed to clean up ENABLING file: {}", e);
        }

        process_result?;
        fs::write(&enabled_path, "")
            .await
            .map_err(ModError::IoError)?;

        Ok(ModResult {
            success: true,
            message: None,
        })
    }
    .await;

    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn disable_mod(mod_name: String, profile_name: String) -> Result<ModResult, String> {
    let result: Result<ModResult, ModError> = async move {
        let settings = Settings::load().map_err(ModError::SettingsError)?;
        let profile = settings
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| ModError::SettingsError("Profile not found".to_string()))?;

        // Pass profile_name to find_mod_dir
        let mod_dir = find_mod_dir(&settings, &mod_name, &profile_name).await?;
        verify_mod_structure(&mod_dir)?;

        let enabled_path = get_enabled_file_path(&mod_dir, &profile_name);
        if !enabled_path.exists() {
            return Ok(ModResult {
                success: true,
                message: Some("Mod already disabled".to_string()),
            });
        }

        let version = get_mod_version(&mod_dir)?;
        let main_subdir = mod_dir.join(&mod_name);
        let dcs_dir = PathBuf::from(&profile.dcs_path);

        process_second_level_dirs(&main_subdir, &dcs_dir, &mod_name, &version, true).await?;
        fs::remove_file(&enabled_path)
            .await
            .map_err(ModError::IoError)?;

        Ok(ModResult {
            success: true,
            message: None,
        })
    }
    .await;

    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn delete_mod(mod_name: String, profile_name: String) -> Result<ModResult, String> {
    let result: Result<ModResult, ModError> = async move {
        let settings = Settings::load().map_err(ModError::SettingsError)?;

        // Check if mod is in sideload directory
        if !settings.sideload_path.is_empty() {
            let sideload_dir = PathBuf::from(&settings.sideload_path);
            if sideload_dir.join(&mod_name).exists() {
                return Err(ModError::EnablementError(
                    "Cannot delete sideloaded mods".to_string(),
                ));
            }
        }

        // Pass profile_name to find_mod_dir
        let mod_dir = find_mod_dir(&settings, &mod_name, &profile_name).await?;

        // Check if the mod is enabled for the current profile
        let enabled_path = get_enabled_file_path(&mod_dir, &profile_name);
        if enabled_path.exists() {
            // Disable the mod first
            disable_mod(mod_name.clone(), profile_name.clone())
                .await
                .map_err(ModError::EnablementError)?;
        }

        // Delete the mod directory
        match fs::remove_dir_all(&mod_dir).await {
            Ok(_) => Ok(ModResult {
                success: true,
                message: Some("Mod deleted successfully".to_string()),
            }),
            Err(e) => Err(ModError::IoError(e)),
        }
    }
    .await;

    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub async fn update_mod(
    app_handle: AppHandle,
    mod_name: String,
    profile_name: String,
    url: String,
) -> Result<ModResult, String> {
    let result: Result<ModResult, ModError> = async move {
        let settings = Settings::load().map_err(ModError::SettingsError)?;

        // Check if mod is in sideload directory
        if !settings.sideload_path.is_empty() {
            let sideload_dir = PathBuf::from(&settings.sideload_path);
            if sideload_dir.join(&mod_name).exists() {
                return Err(ModError::EnablementError(
                    "Cannot update sideloaded mods".to_string(),
                ));
            }
        }

        // Find the mod directory using the profile name
        let mod_dir = find_mod_dir(&settings, &mod_name, &profile_name).await?;

        // Check if mod is enabled for the current profile
        let was_enabled =
            fs::metadata(get_enabled_file_path(&mod_dir, &profile_name)).await.is_ok();

        // If mod is being enabled, error out
        fs::metadata(get_enabling_file_path(&mod_dir, &profile_name)).await.map_err(|_|
            ModError::EnablementError(
                "Cannot update mod while it is being enabled".to_string(),
            )
        )?;

        // If enabled, disable first
        if was_enabled {
            disable_mod(mod_name.clone(), profile_name.clone())
                .await
                .map_err(ModError::EnablementError)?;
        }

        // Find the profile to get the repo_url for the download
        let profile = settings
            .profiles
            .iter()
            .find(|p| p.name == profile_name)
            .ok_or_else(|| {
                ModError::SettingsError(format!("Profile '{}' not found for update", profile_name))
            })?;
        let repo_url = profile.repo_url.clone();

        // Download the updated version, passing the repo_url
        let filename = format!("{}.zip", mod_name);
        let download_result =
            super::mod_download::download_mod(app_handle, url, filename, repo_url).await;

        match download_result {
            Ok(_) => {
                // Re-enable if it was enabled before
                if was_enabled {
                    enable_mod(mod_name.clone(), profile_name)
                        .await
                        .map_err(ModError::EnablementError)?;
                }

                Ok(ModResult {
                    success: true,
                    message: Some("Mod updated successfully".to_string()),
                })
            }
            Err(e) => {
                // If download fails and mod was enabled, try to re-enable it
                if was_enabled {
                    if let Err(enable_err) = enable_mod(mod_name.clone(), profile_name).await {
                        println!(
                            "Failed to re-enable mod after failed update: {}",
                            enable_err
                        );
                    }
                }
                Err(ModError::DownloadError(e))
            }
        }
    }
    .await;

    match result {
        Ok(result) => Ok(result),
        Err(e) => Err(e.to_string()),
    }
}
