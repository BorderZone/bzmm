use super::types::{Category, Mod, ModError};
use std::fs;
use std::path::Path;
use std::collections::HashSet;

// Similar to sideload.rs, but for detecting deprecated mods
pub fn read_mod_metadata(mod_dir: &Path) -> Result<Mod, ModError> {
    println!("Reading metadata for deprecated mod: {:?}", mod_dir);
    let name = mod_dir
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| ModError::SettingsError("Invalid mod directory name".to_string()))?
        .to_string();

    // Read VERSION.txt
    let version = fs::read_to_string(mod_dir.join("VERSION.txt"))
        .unwrap_or_else(|_| "Unknown".to_string())
        .trim()
        .to_string();

    // Read README.txt
    let description = fs::read_to_string(mod_dir.join("README.txt"))
        .unwrap_or_else(|_| format!("Deprecated mod: {}", name))
        .trim()
        .to_string();

    println!("Found deprecated mod: {} ({})", name, version);
    Ok(Mod::new_deprecated(name, version, description))
}

/// Scans a specific XML source's download directory for mods that are present locally
/// but not listed in the active mod names set (derived from the corresponding XML).
pub fn scan_for_deprecated_mods(
    xml_specific_path: &Path,
    active_mod_names: &HashSet<String>,
) -> Result<Category, ModError> {
    println!(
        "Scanning for deprecated mods within specific path: {}",
        xml_specific_path.display()
    );
    if !xml_specific_path.exists() || !xml_specific_path.is_dir() {
        println!("XML-specific directory does not exist or is not a directory.");
        // Not an error, just means no mods downloaded for this source yet.
        return Ok(Category::new_deprecated(Vec::new()));
    }

    let mut deprecated_mods = Vec::new();

    // Iterate through the specific XML source directory
    for entry in fs::read_dir(xml_specific_path)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            if let Some(mod_name) = path.file_name().and_then(|n| n.to_str()) {
                // If the mod is not in the active mods list, it's deprecated
                if !active_mod_names.contains(mod_name) {
                    match read_mod_metadata(&path) {
                        Ok(mod_info) => {
                            println!("Successfully read metadata for deprecated mod: {:?}", path);
                            deprecated_mods.push(mod_info);
                        }
                        Err(e) => eprintln!("Failed to read metadata for deprecated mod {:?}: {}", path, e),
                    }
                }
            }
        }
    }

    println!("Found {} deprecated mods", deprecated_mods.len());
    Ok(Category::new_deprecated(deprecated_mods))
}
