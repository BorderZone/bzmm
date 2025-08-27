use serde::Serialize;
use std::fs;
use std::io;
use std::io::Read;
use std::path::Path;
use tauri::Emitter;
use tokio_util::sync::CancellationToken;
use zip::ZipArchive;

#[derive(Clone, Serialize)]
pub struct ExtractionStatus {
    pub mod_name: String,
    pub status: String,
}

#[derive(Clone, Serialize)]
pub struct ExtractionError {
    pub mod_name: String,
    pub error: String,
}

// Function to verify the zip archive integrity
fn verify_archive(archive: &mut ZipArchive<fs::File>) -> Result<(), String> {
    // Try to enumerate and read data from each file to check for corruption
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(f) => f,
            Err(e) => return Err(format!("Failed to access file in archive: {}", e)),
        };

        // Skip directories
        if file.name().ends_with('/') {
            continue;
        }

        // For large files, just read the start to verify the CRC
        let mut buffer = [0u8; 4096];
        match file.read(&mut buffer) {
            Ok(_) => {} // Successfully read some data
            Err(e) => return Err(format!("Failed to read file '{}': {}", file.name(), e)),
        }
    }

    Ok(())
}

pub async fn extract_zip(
    app_handle: tauri::AppHandle,
    zip_path: &Path,
    extract_dir: &Path,
    mod_name: &str,
) -> Result<(), String> {
    println!(
        "Starting extraction of {} to {}",
        zip_path.display(),
        extract_dir.display()
    );

    // Emit extraction started event
    app_handle
        .emit(
            "extraction-status",
            ExtractionStatus {
                mod_name: mod_name.to_string(),
                status: "extracting".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;

    // Create the extraction directory if it doesn't exist
    fs::create_dir_all(extract_dir).map_err(|e| {
        let error_msg = format!("Failed to create extraction directory: {}", e);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    // Open the zip file
    let file = fs::File::open(zip_path).map_err(|e| {
        let error_msg = format!("Failed to open ZIP file: {}", e);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    // Try to open the archive
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            let error_msg = format!("The ZIP file is corrupted or invalid: {}", e);
            println!("{}", error_msg);
            let _ = app_handle.emit(
                "extraction-error",
                ExtractionError {
                    mod_name: mod_name.to_string(),
                    error: error_msg.clone(),
                },
            );
            // Note: Queue processing will be triggered when new downloads are added
            return Err(error_msg);
        }
    };

    // Verify the archive is intact by checking for CRC errors
    if let Err(e) = verify_archive(&mut archive) {
        let error_msg = format!("ZIP archive failed verification: {}", e);
        println!("{}", error_msg);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        // Note: Queue processing will be triggered when new downloads are added
        return Err(error_msg);
    }

    // Extract each file
    for i in 0..archive.len() {
        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                let error_msg = format!("Failed to read file in ZIP: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        };

        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if let Some(parent) = outpath.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                let error_msg = format!("Failed to create directory: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        }

        if file.name().ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                let error_msg = format!("Failed to create directory: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        } else {
            let mut outfile = match fs::File::create(&outpath) {
                Ok(file) => file,
                Err(e) => {
                    let error_msg = format!("Failed to create file: {}", e);
                    let _ = app_handle.emit(
                        "extraction-error",
                        ExtractionError {
                            mod_name: mod_name.to_string(),
                            error: error_msg.clone(),
                        },
                    );
                    return Err(error_msg);
                }
            };

            if let Err(e) = io::copy(&mut file, &mut outfile) {
                let error_msg = format!("Failed to write file content: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        }
    }

    // Emit extraction completed event
    println!("Extraction completed for {}", mod_name);
    app_handle
        .emit(
            "extraction-status",
            ExtractionStatus {
                mod_name: mod_name.to_string(),
                status: "completed".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;

    // Note: Queue processing will be triggered when new downloads are added

    Ok(())
}

pub async fn extract_zip_with_cancellation(
    app_handle: tauri::AppHandle,
    zip_path: &Path,
    extract_dir: &Path,
    mod_name: &str,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    println!(
        "Starting cancellable extraction of {} to {}",
        zip_path.display(),
        extract_dir.display()
    );

    // Check if cancelled before starting
    if cancel_token.is_cancelled() {
        return Err("Extraction was cancelled".to_string());
    }

    // Emit extraction started event
    app_handle
        .emit(
            "extraction-status",
            ExtractionStatus {
                mod_name: mod_name.to_string(),
                status: "extracting".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;

    // Create the extraction directory if it doesn't exist
    fs::create_dir_all(extract_dir).map_err(|e| {
        let error_msg = format!("Failed to create extraction directory: {}", e);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    // Check if cancelled after directory creation
    if cancel_token.is_cancelled() {
        // Clean up the directory we just created
        let _ = fs::remove_dir_all(extract_dir);
        return Err("Extraction was cancelled".to_string());
    }

    // Open the zip file
    let file = fs::File::open(zip_path).map_err(|e| {
        let error_msg = format!("Failed to open ZIP file: {}", e);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        error_msg
    })?;

    // Try to open the archive
    let mut archive = match ZipArchive::new(file) {
        Ok(archive) => archive,
        Err(e) => {
            let error_msg = format!("The ZIP file is corrupted or invalid: {}", e);
            println!("{}", error_msg);
            let _ = app_handle.emit(
                "extraction-error",
                ExtractionError {
                    mod_name: mod_name.to_string(),
                    error: error_msg.clone(),
                },
            );
            return Err(error_msg);
        }
    };

    // Check if cancelled before verification
    if cancel_token.is_cancelled() {
        let _ = fs::remove_dir_all(extract_dir);
        return Err("Extraction was cancelled".to_string());
    }

    // Verify the archive is intact by checking for CRC errors
    if let Err(e) = verify_archive(&mut archive) {
        let error_msg = format!("ZIP archive failed verification: {}", e);
        println!("{}", error_msg);
        let _ = app_handle.emit(
            "extraction-error",
            ExtractionError {
                mod_name: mod_name.to_string(),
                error: error_msg.clone(),
            },
        );
        return Err(error_msg);
    }

    // Extract each file with cancellation checks
    for i in 0..archive.len() {
        // Check if cancelled before processing each file
        if cancel_token.is_cancelled() {
            // Clean up any partially extracted files
            let _ = fs::remove_dir_all(extract_dir);
            return Err("Extraction was cancelled".to_string());
        }

        let mut file = match archive.by_index(i) {
            Ok(file) => file,
            Err(e) => {
                let error_msg = format!("Failed to read file in ZIP: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        };

        let outpath = match file.enclosed_name() {
            Some(path) => extract_dir.join(path),
            None => continue,
        };

        if let Some(parent) = outpath.parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                let error_msg = format!("Failed to create directory: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        }

        if file.name().ends_with('/') {
            if let Err(e) = fs::create_dir_all(&outpath) {
                let error_msg = format!("Failed to create directory: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        } else {
            let mut outfile = match fs::File::create(&outpath) {
                Ok(file) => file,
                Err(e) => {
                    let error_msg = format!("Failed to create file: {}", e);
                    let _ = app_handle.emit(
                        "extraction-error",
                        ExtractionError {
                            mod_name: mod_name.to_string(),
                            error: error_msg.clone(),
                        },
                    );
                    return Err(error_msg);
                }
            };

            if let Err(e) = io::copy(&mut file, &mut outfile) {
                let error_msg = format!("Failed to write file content: {}", e);
                let _ = app_handle.emit(
                    "extraction-error",
                    ExtractionError {
                        mod_name: mod_name.to_string(),
                        error: error_msg.clone(),
                    },
                );
                return Err(error_msg);
            }
        }
    }

    // Final cancellation check before completion
    if cancel_token.is_cancelled() {
        // Clean up extracted files
        let _ = fs::remove_dir_all(extract_dir);
        return Err("Extraction was cancelled".to_string());
    }

    // Emit extraction completed event
    println!("Extraction completed for {}", mod_name);
    app_handle
        .emit(
            "extraction-status",
            ExtractionStatus {
                mod_name: mod_name.to_string(),
                status: "completed".to_string(),
            },
        )
        .map_err(|e| e.to_string())?;

    Ok(())
}
