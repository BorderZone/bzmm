use std::path::Path;
use tokio::fs;
use futures_util::future::BoxFuture;
use crate::mods::types::ModError;
use super::file_operations::*;
use crate::mods::mod_utils::verify_symlink;

/// Remove a symlink in a cross-platform way
async fn remove_symlink(path: &Path) -> Result<(), ModError> {
    #[cfg(windows)]
    {
        if path.is_dir() {
            tokio::fs::remove_dir(path).await.map_err(ModError::IoError)
        } else {
            tokio::fs::remove_file(path).await.map_err(ModError::IoError)
        }
    }
    #[cfg(not(windows))]
    {
        tokio::fs::remove_file(path).await.map_err(ModError::IoError)
    }
}

/// Create a symlink in a cross-platform way
async fn create_symlink(source: &Path, dest: &Path) -> Result<(), ModError> {
    #[cfg(windows)]
    {
        if source.is_dir() {
            tokio::fs::symlink_dir(source, dest).await.map_err(ModError::IoError)
        } else {
            tokio::fs::symlink_file(source, dest).await.map_err(ModError::IoError)
        }
    }
    #[cfg(not(windows))]
    {
        tokio::fs::symlink(source, dest).await.map_err(ModError::IoError)
    }
}

/// Process a directory at the 4th level and below (create symlinks, patch lua files)
fn process_deep_directory<'a>(
    source_dir: &'a Path,
    dest_dir: &'a Path,
    mod_name: &'a str,
    version: &'a str,
) -> BoxFuture<'a, Result<(), ModError>> {
    Box::pin(async move {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir).await.map_err(ModError::IoError)?;
        }

        let mut entries = fs::read_dir(source_dir).await.map_err(ModError::IoError)?;
        while let Some(entry) = entries.next_entry().await.map_err(ModError::IoError)? {
            let path = entry.path();
            let dest_path = dest_dir.join(path.file_name().unwrap());

            if path.is_dir() {
                if dest_path.exists() {
                    if dest_path.is_symlink() {
                        if !verify_symlink(&dest_path, &path)? {
                            remove_symlink(&dest_path).await?;
                            create_symlink(&path, &dest_path).await?;
                        }
                    } else {
                        process_deep_directory(&path, &dest_path, mod_name, version).await?;
                    }
                } else {
                    create_symlink(&path, &dest_path).await?;
                }
            } else if let Some(extension) = path.extension() {
                if extension == "lua" {
                    if dest_path.exists() {
                        let patch_content = fs::read_to_string(&path).await.map_err(ModError::IoError)?;
                        patch_lua_file(&dest_path, mod_name, version, &patch_content)?;
                    } else {
                        create_symlink(&path, &dest_path).await?;
                    }
                } else if dest_path.exists() {
                    return Err(ModError::FileConflictError(format!(
                        "File {} already exists",
                        dest_path.display()
                    )));
                } else {
                    create_symlink(&path, &dest_path).await?;
                }
            }
        }

        Ok(())
    })
}

/// Clean up symlinks and patches from a directory (4th level and below only)
fn cleanup_deep_directory<'a>(
    source_dir: &'a Path,
    dest_dir: &'a Path,
    mod_name: &'a str,
    version: &'a str,
) -> BoxFuture<'a, Result<(), ModError>> {
    Box::pin(async move {
        if !dest_dir.exists() {
            return Ok(());
        }

        let mut entries = fs::read_dir(source_dir).await.map_err(ModError::IoError)?;
        while let Some(entry) = entries.next_entry().await.map_err(ModError::IoError)? {
            let path = entry.path();
            let dest_path = dest_dir.join(path.file_name().unwrap());

            if !dest_path.exists() {
                continue;
            }

            if path.is_dir() {
                if dest_path.is_symlink() && verify_symlink(&dest_path, &path)? {
                    remove_symlink(&dest_path).await?;
                } else if !dest_path.is_symlink() {
                    cleanup_deep_directory(&path, &dest_path, mod_name, version).await?;
                    // Remove directory if empty
                    let mut read_dir = fs::read_dir(&dest_path).await.map_err(ModError::IoError)?;
                    if read_dir.next_entry().await.map_err(ModError::IoError)?.is_none() {
                        fs::remove_dir(&dest_path).await.map_err(ModError::IoError)?;
                    }
                }
            } else if let Some(extension) = path.extension() {
                if extension == "lua" {
                    if dest_path.is_symlink() && verify_symlink(&dest_path, &path)? {
                        remove_symlink(&dest_path).await?;
                    } else {
                        remove_lua_patch_from_file(&dest_path, mod_name, version)?;
                        let content = fs::read_to_string(&dest_path).await.map_err(ModError::IoError)?;
                        if content.trim().is_empty() {
                            fs::remove_file(&dest_path).await.map_err(ModError::IoError)?;
                        }
                    }
                } else if dest_path.is_symlink() && verify_symlink(&dest_path, &path)? {
                    remove_symlink(&dest_path).await?;
                }
            }
        }

        Ok(())
    })
}

/// Process directories recursively, handling different levels appropriately
fn process_directory<'a>(
    source_dir: &'a Path,
    dest_dir: &'a Path,
    mod_name: &'a str,
    version: &'a str,
    level: u8,
    cleanup: bool,
) -> BoxFuture<'a, Result<(), ModError>> {
    Box::pin(async move {
        if !dest_dir.exists() {
            fs::create_dir_all(dest_dir).await.map_err(ModError::IoError)?;
        }

        let mut entries = fs::read_dir(source_dir).await.map_err(ModError::IoError)?;
        while let Some(entry) = entries.next_entry().await.map_err(ModError::IoError)? {
            let path = entry.path();
            let file_name = path.file_name().unwrap(); // Safe to unwrap as we are reading directory entries
            let dest_path = dest_dir.join(file_name);

            if path.is_dir() {
                // Handle directories based on level
                match level {
                    // Level 2: Create directory and recurse
                    2 => {
                        if !dest_path.exists() {
                            fs::create_dir_all(&dest_path).await.map_err(ModError::IoError)?;
                        }
                        process_directory(&path, &dest_path, mod_name, version, level + 1, cleanup).await?;
                    }
                    // Level 3: Create directory and handle 4th level content
                    3 => {
                        if !dest_path.exists() && !cleanup {
                            fs::create_dir_all(&dest_path).await.map_err(ModError::IoError)?;
                        }
                        if cleanup {
                            cleanup_deep_directory(&path, &dest_path, mod_name, version).await?;
                            // Attempt to remove the directory if it's empty after cleanup
                            let mut read_dir = fs::read_dir(&dest_path).await.map_err(ModError::IoError)?;
                            if read_dir.next_entry().await.map_err(ModError::IoError)?.is_none() {
                                fs::remove_dir(&dest_path).await.map_err(ModError::IoError)?;
                            }
                        } else {
                            process_deep_directory(&path, &dest_path, mod_name, version).await?;
                        }
                    }
                    // Level 4+: Should never get here as it's handled by process_deep_directory/cleanup_deep_directory
                    _ => {}
                }
            } else if path.is_file() {
                // Handle files directly at levels 2 and 3
                if cleanup {
                    // Cleanup: Remove symlink if it exists and points to the correct source
                    if dest_path.exists() && dest_path.is_symlink() && verify_symlink(&dest_path, &path)? {
                        remove_symlink(&dest_path).await?;
                    }
                } else {
                    // Enable: Create symlink, handling conflicts
                    if dest_path.exists() {
                        if dest_path.is_symlink() {
                            // If it's already a symlink, verify it points to the correct source
                            if !verify_symlink(&dest_path, &path)? {
                                // Incorrect symlink, replace it
                                remove_symlink(&dest_path).await?;
                                create_symlink(&path, &dest_path).await?;
                            }
                            // If correct symlink, do nothing
                        } else {
                            // Conflict: A real file/directory exists where we want to put a symlink
                            return Err(ModError::FileConflictError(format!(
                                "File conflict: {} already exists and is not a symlink.",
                                dest_path.display()
                            )));
                        }
                    } else {
                        // Destination doesn't exist, create the symlink
                        create_symlink(&path, &dest_path).await?;
                    }
                }
            }
            // Ignore other entry types (like symlinks in the source mod directory)
        }

        Ok(())
    })
}

// This is the entry point - starts at level 2 (mods, liveries, etc.)
pub async fn process_second_level_dirs(
    source_dir: &Path,
    dcs_dir: &Path,
    mod_name: &str,
    version: &str,
    cleanup: bool,
) -> Result<(), ModError> {
    process_directory(source_dir, dcs_dir, mod_name, version, 2, cleanup).await
}
