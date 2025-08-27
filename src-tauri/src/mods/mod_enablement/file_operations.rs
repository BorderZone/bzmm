use std::path::Path;
use std::fs;
use crate::mods::types::ModError;
use super::patching::{check_lua_patch, add_lua_patch, remove_lua_patch};

/// Patch a lua file in place
pub fn patch_lua_file(
    file_path: &Path,
    mod_name: &str,
    version: &str,
    patch: &str,
) -> Result<(), ModError> {
    let content = fs::read_to_string(file_path).map_err(ModError::IoError)?;
    
    if check_lua_patch(&content, mod_name, version) {
        return Ok(());
    }

    let new_content = add_lua_patch(&content, mod_name, version, patch);
    fs::write(file_path, new_content).map_err(ModError::IoError)?;

    Ok(())
}

/// Remove a lua patch from a file in place
pub fn remove_lua_patch_from_file(
    file_path: &Path,
    mod_name: &str,
    version: &str,
) -> Result<(), ModError> {
    let content = fs::read_to_string(file_path).map_err(ModError::IoError)?;
    let new_content = remove_lua_patch(&content, mod_name, version)?;
    fs::write(file_path, new_content).map_err(ModError::IoError)?;
    Ok(())
}