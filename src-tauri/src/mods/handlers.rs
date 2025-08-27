use super::downloader::ModDownloader;
use super::mod_download::is_mod_successfully_downloaded;
use super::parser::ModParser;
use super::sideload::scan_sideload_directory;
use super::deprecated::scan_for_deprecated_mods;
use super::types::ModsResult;
use crate::settings;
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::PathBuf;

#[tauri::command]
pub async fn get_enabled_mods(profile_name: String) -> Result<Vec<String>, String> {
    let settings = settings::Settings::load()?;
    let base_downloads_dir = PathBuf::from(&settings.download_path);
    let mut enabled_mods = Vec::new();

    // Find the profile to get the repo_url
    let profile = settings
        .profiles
        .iter()
        .find(|p| p.name == profile_name)
        .ok_or_else(|| format!("Profile '{}' not found", profile_name))?;

    // Calculate the XML-specific path for this profile
    let mut hasher = Sha256::new();
    hasher.update(profile.repo_url.as_bytes());
    let hash_result = hasher.finalize();
    let repo_hash = format!("{:x}", hash_result);
    // Shrink the hash to 6 characters
    let repo_hash = &repo_hash[..6];
    let xml_specific_path = base_downloads_dir.join(&repo_hash);

    println!("Checking for enabled mods within: {}", xml_specific_path.display());

    if xml_specific_path.exists() && xml_specific_path.is_dir() {
        // Iterate within the specific XML source directory
        let mod_dir_entries = std::fs::read_dir(&xml_specific_path).map_err(|e| e.to_string())?;
        for mod_entry in mod_dir_entries.filter_map(Result::ok) {
            let mod_path = mod_entry.path(); // Path to the specific mod directory
            if mod_path.is_dir() {
                if let Some(mod_name) = mod_path.file_name().and_then(|n| n.to_str()) {
                    // Check if this specific mod is enabled for the given profile
                    if super::mod_utils::is_mod_enabled(&mod_path, &profile_name) {
                        enabled_mods.push(mod_name.to_string());
                    }
                }
            }
        }
    }

    Ok(enabled_mods)
}

#[tauri::command]
pub async fn get_mods(profile_index: usize) -> Result<ModsResult, String> {
    let mut settings = settings::Settings::load()?;
    
    if profile_index >= settings.profiles.len() {
        return Ok(ModsResult {
            categories: Vec::new(),
            error: Some("Profile index out of bounds".to_string()),
        });
    }
    
    let url = settings.profiles[profile_index].repo_url.trim_end_matches('/').to_string();
    let downloader = ModDownloader::new();
    let mut categories = Vec::new();
    let mut error = None;
    let mut xml_loaded_from_cache = false;
    let download_path = PathBuf::from(&settings.download_path);

    // Try to fetch and parse mods from the URL
    match downloader.fetch_and_parse_mods(&url).await {
        Ok((mods_file, cache_path)) => {
            // Save the cache path if available
            if let Some(path) = cache_path {
                if let Err(e) = super::xml_cache::update_cache_path_in_settings(&mut settings, &url, &path) {
                    println!("Warning: Failed to update cache path in settings: {}", e);
                }
            }

            // Pass the repo URL to check_for_updates
            let updated_mods = match ModParser::check_for_updates(
                &mods_file,
                &download_path,
                &url,
            ) {
                Ok(updated) => {
                    // Debug logging for each mod after update check
                    for category in &updated.categories {
                        for mod_entry in &category.mods {
                            println!("After update check - Mod: {}, Version: {}, New Version: {:?}",
                                mod_entry.name, mod_entry.version, mod_entry.new_version);
                        }
                    }
                    updated
                },
                Err(e) => {
                    println!("Warning: Failed to check for updates: {}", e);
                    mods_file
                }
            };

            categories = updated_mods.categories;
            categories.sort_by_key(|cat| cat.sort_order);
        },
        Err(e) => {
            // Could not fetch from URL, try to load from cache
            println!("Failed to load repository mods: {}", e);
            error = Some(format!("Failed to load repository XML: {}", e));
            
            // Try to find a cached XML file for this profile
            let cached_xml_path = if profile_index < settings.cached_xml_paths.len() && !settings.cached_xml_paths[profile_index].is_empty() {
                Some(PathBuf::from(&settings.cached_xml_paths[profile_index]))
            } else {
                super::xml_cache::XmlCache::get_cache_path(&url)
            };
            
            if let Some(path) = cached_xml_path {
                match super::xml_cache::XmlCache::load_xml(&path) {
                    Ok(cached_mods_file) => {
                        println!("Successfully loaded cached XML from: {}", path.display());
                        xml_loaded_from_cache = true;

                        // Check for updates using the cached file, passing the repo URL
                        let updated_mods = match ModParser::check_for_updates(
                            &cached_mods_file,
                            &download_path,
                            &url,
                        ) {
                            Ok(updated) => updated,
                            Err(e) => {
                                println!("Warning: Failed to check for updates using cached XML: {}", e);
                                cached_mods_file
                            }
                        };
                        
                        categories = updated_mods.categories;
                        categories.sort_by_key(|cat| cat.sort_order);
                    },
                    Err(cache_err) => {
                        println!("Failed to load cached XML: {}", cache_err);
                        error = Some(format!("Failed to load repository XML and could not read cache: {}", e));
                    }
                }
            } else {
                println!("No cached XML available for URL: {}", url);
                error = Some(format!("Failed to load repository XML: {}. No cached version available.", e));
            }
        }
    }
    
    // Handle error message for cached data
    if xml_loaded_from_cache {
        // If there's an error message that already mentions cached data, keep it
        if let Some(err_msg) = &error {
            if !err_msg.contains("cached") && !err_msg.contains("Cached") {
                error = Some(format!("{}. Using cached XML file.", err_msg));
            }
        } else {
            // If no error, still inform user we're using cached data
            error = Some("Using cached repository data. Could not connect to repository.".to_string());
        }
    }
    
    // Collect active mod names to identify deprecated mods
    let active_mod_names: HashSet<String> = categories
        .iter()
        .flat_map(|cat| cat.mods.iter().map(|m| m.name.clone()))
        .collect();

    // Scan for deprecated mods within the specific XML source directory
    if !settings.download_path.is_empty() {
        // Calculate the XML-specific path for deprecation scanning
        let base_downloads_dir = PathBuf::from(&settings.download_path);
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes()); // url holds the repo_url here
        let hash_result = hasher.finalize();
        let repo_hash = format!("{:x}", hash_result);
        let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
        let xml_specific_path = base_downloads_dir.join(&repo_hash);

        match scan_for_deprecated_mods(&xml_specific_path, &active_mod_names) {
            Ok(deprecated_category) => {
                if !deprecated_category.mods.is_empty() {
                    // Add the deprecated mods to the categories list
                    categories.push(deprecated_category);
                }
            },
            Err(e) => {
                println!("Failed to scan for deprecated mods: {}", e);
            }
        }
    }
    
    // Add sideloaded mods
    if !settings.sideload_path.is_empty() {
        match scan_sideload_directory(&settings.sideload_path) {
            Ok(mut sideload_category) => {
                if !sideload_category.mods.is_empty() {
                    sideload_category.sort_order = categories
                        .last()
                        .map(|cat| cat.sort_order + 1)
                        .unwrap_or(0);
                    categories.push(sideload_category);
                }
            },
            Err(e) => {
                println!("Failed to scan sideload directory: {}", e);
            }
        }
    }
    
    Ok(ModsResult {
        categories,
        error,
    })
}

#[tauri::command]
pub async fn get_downloaded_mods() -> Result<Vec<String>, String> {
    let settings = settings::Settings::load()?;
    let base_downloads_dir = PathBuf::from(&settings.download_path);

    let mut downloaded_mods = Vec::new();

    if base_downloads_dir.exists() {
        // Iterate through the hashed subdirectories first
        let hash_dir_entries = std::fs::read_dir(&base_downloads_dir).map_err(|e| e.to_string())?;
        for hash_entry in hash_dir_entries.filter_map(Result::ok) {
            let xml_specific_path = hash_entry.path();
            // Ensure it's a directory (could be a stray file)
            if xml_specific_path.is_dir() {
                // Now iterate inside the XML-specific directory for mod directories
                let mod_dir_entries = std::fs::read_dir(&xml_specific_path).map_err(|e| e.to_string())?;
                for mod_entry in mod_dir_entries.filter_map(Result::ok) {
                    let mod_path = mod_entry.path();
                    if mod_path.is_dir() {
                        if let Some(mod_name) = mod_path.file_name().and_then(|n| n.to_str()) {
                            // Call the updated function with the XML-specific path
                            if is_mod_successfully_downloaded(&xml_specific_path, mod_name) {
                                // Avoid duplicates if a mod exists under multiple XML sources (unlikely but possible)
                                if !downloaded_mods.contains(&mod_name.to_string()) {
                                    downloaded_mods.push(mod_name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if !settings.sideload_path.is_empty() {
        let sideload_dir = PathBuf::from(&settings.sideload_path);
        if sideload_dir.exists() {
            let entries = std::fs::read_dir(&sideload_dir).map_err(|e| e.to_string())?;
            for entry in entries.filter_map(Result::ok) {
                let path = entry.path();
                if path.is_dir() {
                    if let Some(mod_name) = path.file_name().and_then(|n| n.to_str()) {
                        downloaded_mods.push(mod_name.to_string());
                    }
                }
            }
        }
    }

    Ok(downloaded_mods)
}
