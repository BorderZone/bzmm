use crate::mods::types::ModError;
use std::fs;
use std::path::{Path, PathBuf};

/// Check if a directory follows the expected mod structure
pub fn verify_mod_structure(mod_path: &Path) -> Result<(), ModError> {
    // Check for required files
    let version_path = mod_path.join("VERSION.txt");
    let readme_path = mod_path.join("README.txt");

    if !version_path.exists() {
        return Err(ModError::DirectoryStructureError(
            "VERSION.txt not found".to_string(),
        ));
    }

    if !readme_path.exists() {
        return Err(ModError::DirectoryStructureError(
            "README.txt not found".to_string(),
        ));
    }

    // Check for main subdirectory
    let dir_name = mod_path
        .file_name()
        .ok_or_else(|| ModError::DirectoryStructureError("Invalid mod path".to_string()))?;

    let main_subdir = mod_path.join(dir_name);
    if !main_subdir.is_dir() {
        return Err(ModError::DirectoryStructureError(
            "Main subdirectory not found".to_string(),
        ));
    }

    Ok(())
}

/// Check if a symlink points to the expected target
pub fn verify_symlink(link_path: &Path, expected_target: &Path) -> Result<bool, ModError> {
    if !link_path.is_symlink() {
        return Ok(false);
    }

    let target = fs::read_link(link_path).map_err(ModError::IoError)?;
    Ok(target == expected_target)
}

/// Get the version from VERSION.txt
pub fn get_mod_version(mod_path: &Path) -> Result<String, ModError> {
    let version = fs::read_to_string(mod_path.join("VERSION.txt"))
        .map_err(ModError::IoError)?
        .trim()
        .to_string();
    Ok(version)
}

/// Get path to ENABLED file for a profile
pub fn get_enabled_file_path(mod_path: &Path, profile_name: &str) -> PathBuf {
    mod_path.join(format!("ENABLED-{}.txt", profile_name))
}

/// Get path to ENABLING file for a profile
pub fn get_enabling_file_path(mod_path: &Path, profile_name: &str) -> PathBuf {
    mod_path.join(format!("ENABLING-{}.txt", profile_name))
}

/// Check if a mod is enabled for a profile
pub fn is_mod_enabled(mod_path: &Path, profile_name: &str) -> bool {
    get_enabled_file_path(mod_path, profile_name).exists()
}

