use super::types::{Category, Mod, ModError};
use std::fs;
use std::path::Path;

pub fn read_mod_metadata(mod_dir: &Path) -> Result<Mod, ModError> {
    println!("Reading metadata for directory: {:?}", mod_dir);
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
        .unwrap_or_else(|_| format!("Sideloaded mod: {}", name))
        .trim()
        .to_string();

    println!("Found sideloaded mod: {} ({})", name, version);
    Ok(Mod::new_sideloaded(name, version, description))
}

pub fn scan_sideload_directory(sideload_path: &str) -> Result<Category, ModError> {
    println!("Scanning sideload directory: {}", sideload_path);
    let sideload_dir = Path::new(sideload_path);
    if !sideload_dir.exists() {
        println!("Sideload directory does not exist");
        return Ok(Category::new_sideloaded(Vec::new()));
    }

    let mut sideloaded_mods = Vec::new();

    for entry in fs::read_dir(sideload_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            match read_mod_metadata(&path) {
                Ok(mod_info) => {
                    println!("Successfully read metadata for {:?}", path);
                    sideloaded_mods.push(mod_info);
                }
                Err(e) => eprintln!("Failed to read metadata for {:?}: {}", path, e),
            }
        }
    }

    println!("Found {} sideloaded mods", sideloaded_mods.len());
    Ok(Category::new_sideloaded(sideloaded_mods))
}

