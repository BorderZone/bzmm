use serde_json::json;
use crate::mods::types::ModError;

pub const PATCH_MARKER: &str = "-- This was added automatically by BorderZone Mod Manager. DO NOT EDIT! --";

/// Check if a lua file already has a patch for this mod version
pub fn check_lua_patch(content: &str, mod_name: &str, version: &str) -> bool {
    let mut lines = content.lines().peekable();
    while let Some(line) = lines.next() {
        if line.trim() == PATCH_MARKER {
            // Found marker, read json line
            if let Some(json_line) = lines.next() {
                if let Ok(info) = serde_json::from_str::<serde_json::Value>(
                    json_line.trim_start_matches("-- "),
                ) {
                    if let (Some(name), Some(ver)) = (
                        info["mod_name"].as_str(),
                        info["version"].as_str(),
                    ) {
                        if name == mod_name && ver == version {
                            // Skip rest of the patch block
                            for line in lines.by_ref() {
                                if line.trim() == PATCH_MARKER {
                                    break;
                                }
                            }
                            return true;
                        }
                    }
                }
                // Skip rest of this block even if it wasn't our patch
                for line in lines.by_ref() {
                    if line.trim() == PATCH_MARKER {
                        break;
                    }
                }
            }
        }
    }
    false
}

/// Add a lua patch to the end of a file
pub fn add_lua_patch(content: &str, mod_name: &str, version: &str, patch: &str) -> String {
    let info_json = json!({
        "mod_name": mod_name,
        "version": version
    });

    format!(
        "{}\n\n{}\n-- {}\n{}\n{}",
        content.trim_end(),
        PATCH_MARKER,
        info_json,
        patch.trim(),
        PATCH_MARKER
    )
}

/// Remove a lua patch from a file
pub fn remove_lua_patch(content: &str, mod_name: &str, version: &str) -> Result<String, ModError> {
    let mut result = Vec::new();
    let mut lines = content.lines().peekable();
    
    while let Some(line) = lines.next() {
        if line.trim() == PATCH_MARKER {
            if let Some(json_line) = lines.next() {
                if let Ok(info) = serde_json::from_str::<serde_json::Value>(
                    json_line.trim_start_matches("-- "),
                ) {
                    if let (Some(name), Some(ver)) = (
                        info["mod_name"].as_str(),
                        info["version"].as_str(),
                    ) {
                        if name == mod_name && ver == version {
                            // Skip until end marker
                            for line in lines.by_ref() {
                                if line.trim() == PATCH_MARKER {
                                    break;
                                }
                            }
                            continue;
                        }
                    }
                }
                // If we get here, it wasn't our patch, add both lines back
                result.push(line);
                result.push(json_line);
                // Add rest of block
                for line in lines.by_ref() {
                    result.push(line);
                    if line.trim() == PATCH_MARKER {
                        break;
                    }
                }
            }
        } else {
            result.push(line);
        }
    }

    Ok(result.join("\n"))
}