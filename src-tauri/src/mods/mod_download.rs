use super::downloader::ModDownloader;
use super::extraction::extract_zip;
use crate::settings;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

/// Checks if a mod is successfully downloaded and extracted within a specific XML source directory.
///
/// # Arguments
///
/// * `xml_specific_path` - The path to the directory containing mods for a specific XML source (e.g., .../downloads/<repo_hash>/).
/// * `mod_name` - The name of the mod (directory name).
///
/// # Returns
///
/// * `true` if the mod directory exists and the corresponding zip file does NOT exist within `xml_specific_path`.
/// * `false` otherwise.
pub fn is_mod_successfully_downloaded(xml_specific_path: &Path, mod_name: &str) -> bool {
    // Check for the zip file *within* the XML-specific directory
    let zip_path = xml_specific_path.join(format!("{}.zip", mod_name));
    // Check for the extracted mod directory *within* the XML-specific directory
    let dir_path = xml_specific_path.join(mod_name);

    match (zip_path.exists(), dir_path.exists() && dir_path.is_dir()) {
        (true, true) => false,   // Both exist = failed extraction
        (true, false) => false,  // Only zip exists = incomplete download
        (false, true) => true,   // Only dir exists = successful download
        (false, false) => false, // Neither exists = not downloaded
    }
}

// Remove existing mod directory before downloading a new one
fn clean_existing_mod(extract_dir: &Path) -> Result<(), String> {
    if extract_dir.exists() {
        println!("Removing existing mod directory: {}", extract_dir.display());
        if let Err(e) = std::fs::remove_dir_all(extract_dir) {
            eprintln!("Failed to remove existing mod directory: {}", e);
            return Err(e.to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn download_mod(
    app_handle: tauri::AppHandle,
    url: String,
    filename: String,
    repo_url: String, // Added repo_url parameter
) -> Result<(), String> {
    println!("Starting mod download: {} from {} (Repo: {})", filename, url, repo_url);

    let settings = settings::Settings::load()?;
    let base_downloads_dir = PathBuf::from(&settings.download_path);

    // Generate a unique subdirectory name from the repo_url hash
    let mut hasher = Sha256::new();
    hasher.update(repo_url.as_bytes());
    let hash_result = hasher.finalize();
    let repo_hash = format!("{:x}", hash_result);
    let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
    let xml_specific_path = base_downloads_dir.join(repo_hash);

    // Create the XML-specific directory if it doesn't exist
    if !xml_specific_path.exists() {
        println!("Creating XML-specific download directory: {}", xml_specific_path.display());
        std::fs::create_dir_all(&xml_specific_path)
            .map_err(|e| format!("Failed to create XML-specific download directory: {}", e))?;
    } else {
         println!("Using existing XML-specific download directory: {}", xml_specific_path.display());
    }

    let mod_name = filename.trim_end_matches(".zip");
    // Use xml_specific_path as the base for download/extraction
    let file_path = xml_specific_path.join(&filename);
    let extract_dir = xml_specific_path.join(mod_name);
    let temp_file_path = file_path.with_extension("tmp");

    // Clean existing mod directory within the specific subdirectory
    // TODO: Update clean_existing_mod to handle potential errors better if needed
    clean_existing_mod(&extract_dir)?;

    // Notify that download is starting (this will update UI to show download is active)
    if let Err(e) = app_handle.emit("download-started", &filename) {
        eprintln!("Failed to emit download-started event: {}", e);
    }

    let downloader = ModDownloader::new();

    // Download to temporary file first
    println!(
        "Starting download for {} to temporary file: {}",
        filename,
        temp_file_path.display()
    );
    let download_result = downloader
        .download_mod(app_handle.clone(), &url, &temp_file_path, &filename)
        .await;

    // If download failed, return error
    if let Err(e) = download_result {
        println!("Download failed for {}: {}", filename, e);
        if temp_file_path.exists() {
            let _ = std::fs::remove_file(&temp_file_path);
        }
        return Err(e.to_string());
    }

    // Move temp file to final location
    println!(
        "Download completed, moving temporary file to: {}",
        file_path.display()
    );
    if let Err(e) = std::fs::rename(&temp_file_path, &file_path) {
        println!("Failed to move temporary file: {}", e);
        return Err(e.to_string());
    }

    // Verify file is a valid ZIP before trying to extract
    println!("Verifying ZIP file: {}", file_path.display());
    let file_size = match std::fs::metadata(&file_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            let error_message = format!("Failed to get file metadata: {}", e);
            println!("{}", error_message);
            
            // Emit the error event to the frontend
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": filename,
                    "error": error_message
                }),
            );
            
            return Err(error_message);
        }
    };

    // Check file size - a tiny file is probably an error message, not a ZIP
    if file_size < 100 {
        // ZIP files should be much larger than 100 bytes
        // Read the file content to see what the error is
        let error_message = match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                println!(
                    "File too small to be a valid ZIP ({}B): {}",
                    file_size, content
                );
                format!("Server returned error: {}", content)
            }
            Err(_) => {
                println!("File too small to be a valid ZIP ({}B)", file_size);
                format!(
                    "Downloaded file is too small to be a valid ZIP ({} bytes)",
                    file_size
                )
            }
        };
        
        // Emit the error event to the frontend
        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        // Clean up the corrupted file
        let _ = std::fs::remove_file(&file_path);
        
        return Err(error_message);
    }

    // Quick check if it starts with the ZIP header (PK..)
    let file = match std::fs::File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            let error_message = format!("Failed to open file for validation: {}", e);
            println!("{}", error_message);
            
            // Emit the error event to the frontend
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": filename,
                    "error": error_message
                }),
            );
            
            return Err(error_message);
        }
    };

    let mut reader = std::io::BufReader::new(file);
    let mut buffer = [0u8; 4];
    if let Err(e) = std::io::Read::read_exact(&mut reader, &mut buffer) {
        let error_message = format!("Failed to read file header: {}", e);
        println!("{}", error_message);
        
        // Emit the error event to the frontend
        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        // Clean up the corrupted file
        let _ = std::fs::remove_file(&file_path);
        
        return Err(error_message);
    }

    // ZIP files should start with "PK\x03\x04"
    if buffer != [0x50, 0x4B, 0x03, 0x04] {
        // Not a valid ZIP - could be an HTML error page
        let content = std::fs::read_to_string(&file_path)
            .map_or_else(|_| "<binary content>".to_string(), |c| c);

        println!(
            "Invalid ZIP header: {:?} - Content starts with: {}",
            buffer,
            content.chars().take(100).collect::<String>()
        );
        
        // Emit an error event
        let error_message =
            "Downloaded file is not a valid ZIP archive. File might be corrupted.".to_string();

        // Emit an error event to the frontend
        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        // Clean up the corrupted file
        let _ = std::fs::remove_file(&file_path);
        
        return Err(error_message);
    }

    // Extract the zip file
    println!(
        "Starting extraction from {} to {}",
        file_path.display(),
        extract_dir.display()
    );
    let extract_result = extract_zip(app_handle.clone(), &file_path, &extract_dir, &filename).await;

    // If extraction failed, clean up and return error
    if let Err(e) = extract_result {
        println!("Extraction failed for {}: {}", filename, e);
        
        // Remove the downloaded zip file
        let _ = std::fs::remove_file(&file_path);
        
        // Try to clean up any partially extracted files
        if extract_dir.exists() {
            println!("Cleaning up partial extraction at {}", extract_dir.display());
            let _ = std::fs::remove_dir_all(&extract_dir);
        }
        
        return Err(e);
    }

    println!("Extraction completed successfully for {}", filename);

    // Remove the zip file after successful extraction
    if let Err(e) = std::fs::remove_file(&file_path) {
        eprintln!(
            "Warning: Failed to remove zip file after successful extraction: {}",
            e
        );
        // Don't fail the operation just because we couldn't clean up the zip
    }

    Ok(())
}

pub async fn download_mod_with_cancellation(
    app_handle: tauri::AppHandle,
    url: String,
    filename: String,
    repo_url: String,
    cancel_token: CancellationToken,
) -> Result<(), String> {
    // Check if cancelled before starting
    if cancel_token.is_cancelled() {
        return Err("Download was cancelled".to_string());
    }

    println!("Starting cancellable mod download: {} from {} (Repo: {})", filename, url, repo_url);

    let settings = settings::Settings::load()?;
    let base_downloads_dir = PathBuf::from(&settings.download_path);

    // Generate a unique subdirectory name from the repo_url hash
    let mut hasher = Sha256::new();
    hasher.update(repo_url.as_bytes());
    let hash_result = hasher.finalize();
    let repo_hash = format!("{:x}", hash_result);
    let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
    let xml_specific_path = base_downloads_dir.join(repo_hash);

    // Create the XML-specific directory if it doesn't exist
    if !xml_specific_path.exists() {
        println!("Creating XML-specific download directory: {}", xml_specific_path.display());
        std::fs::create_dir_all(&xml_specific_path)
            .map_err(|e| format!("Failed to create XML-specific download directory: {}", e))?;
    } else {
         println!("Using existing XML-specific download directory: {}", xml_specific_path.display());
    }

    let mod_name = filename.trim_end_matches(".zip");
    // Use xml_specific_path as the base for download/extraction
    let file_path = xml_specific_path.join(&filename);
    let extract_dir = xml_specific_path.join(mod_name);
    let temp_file_path = file_path.with_extension("tmp");

    // Check if cancelled before proceeding
    if cancel_token.is_cancelled() {
        return Err("Download was cancelled".to_string());
    }

    // Clean existing mod directory within the specific subdirectory
    clean_existing_mod(&extract_dir)?;

    // Notify that download is starting (this will update UI to show download is active)
    if let Err(e) = app_handle.emit("download-started", &filename) {
        eprintln!("Failed to emit download-started event: {}", e);
    }

    let downloader = ModDownloader::new();

    // Download to temporary file first with cancellation support
    println!(
        "Starting cancellable download for {} to temporary file: {}",
        filename,
        temp_file_path.display()
    );
    
    let download_result = downloader
        .download_mod_with_cancellation(app_handle.clone(), &url, &temp_file_path, &filename, cancel_token.clone())
        .await;

    // Check if cancelled after download attempt
    if cancel_token.is_cancelled() {
        // Clean up temp file if it exists
        if temp_file_path.exists() {
            let _ = std::fs::remove_file(&temp_file_path);
        }
        return Err("Download was cancelled".to_string());
    }

    // If download failed, return error
    if let Err(e) = download_result {
        let error_msg = e.to_string();
        
        // Don't log as error for user-initiated cancellations
        if !error_msg.to_lowercase().contains("cancelled") {
            println!("Download failed for {}: {}", filename, e);
        } else {
            println!("Download cancelled for {}", filename);
        }
        
        if temp_file_path.exists() {
            let _ = std::fs::remove_file(&temp_file_path);
        }
        return Err(error_msg);
    }

    // Move temp file to final location
    println!(
        "Download completed, moving temporary file to: {}",
        file_path.display()
    );
    if let Err(e) = std::fs::rename(&temp_file_path, &file_path) {
        println!("Failed to move temporary file: {}", e);
        return Err(e.to_string());
    }

    // Check if cancelled before extraction
    if cancel_token.is_cancelled() {
        // Clean up downloaded file
        let _ = std::fs::remove_file(&file_path);
        return Err("Download was cancelled".to_string());
    }

    // Verify file is a valid ZIP before trying to extract (same validation as original)
    println!("Verifying ZIP file: {}", file_path.display());
    let file_size = match std::fs::metadata(&file_path) {
        Ok(metadata) => metadata.len(),
        Err(e) => {
            let error_message = format!("Failed to get file metadata: {}", e);
            println!("{}", error_message);
            
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": filename,
                    "error": error_message
                }),
            );
            
            return Err(error_message);
        }
    };

    // Check file size - a tiny file is probably an error message, not a ZIP
    if file_size < 100 {
        let error_message = match std::fs::read_to_string(&file_path) {
            Ok(content) => {
                println!("File too small to be a valid ZIP ({}B): {}", file_size, content);
                format!("Server returned error: {}", content)
            }
            Err(_) => {
                println!("File too small to be a valid ZIP ({}B)", file_size);
                format!("Downloaded file is too small to be a valid ZIP ({} bytes)", file_size)
            }
        };
        
        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        let _ = std::fs::remove_file(&file_path);
        return Err(error_message);
    }

    // Quick check if it starts with the ZIP header (PK..)
    let file = match std::fs::File::open(&file_path) {
        Ok(f) => f,
        Err(e) => {
            let error_message = format!("Failed to open file for validation: {}", e);
            println!("{}", error_message);
            
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": filename,
                    "error": error_message
                }),
            );
            
            return Err(error_message);
        }
    };

    let mut reader = std::io::BufReader::new(file);
    let mut buffer = [0u8; 4];
    if let Err(e) = std::io::Read::read_exact(&mut reader, &mut buffer) {
        let error_message = format!("Failed to read file header: {}", e);
        println!("{}", error_message);
        
        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        let _ = std::fs::remove_file(&file_path);
        return Err(error_message);
    }

    // ZIP files should start with "PK\x03\x04"
    if buffer != [0x50, 0x4B, 0x03, 0x04] {
        let content = std::fs::read_to_string(&file_path)
            .map_or_else(|_| "<binary content>".to_string(), |c| c);

        println!(
            "Invalid ZIP header: {:?} - Content starts with: {}",
            buffer,
            content.chars().take(100).collect::<String>()
        );
        
        let error_message =
            "Downloaded file is not a valid ZIP archive. File might be corrupted.".to_string();

        let _ = app_handle.emit(
            "download-error",
            serde_json::json!({
                "mod_name": filename,
                "error": error_message
            }),
        );
        
        let _ = std::fs::remove_file(&file_path);
        return Err(error_message);
    }

    // Check if cancelled before extraction
    if cancel_token.is_cancelled() {
        // Clean up downloaded file
        let _ = std::fs::remove_file(&file_path);
        return Err("Download was cancelled".to_string());
    }

    // Extract the zip file with cancellation support
    println!(
        "Starting cancellable extraction from {} to {}",
        file_path.display(),
        extract_dir.display()
    );
    let extract_result = super::extraction::extract_zip_with_cancellation(
        app_handle.clone(), 
        &file_path, 
        &extract_dir, 
        &filename, 
        cancel_token.clone()
    ).await;

    // If extraction failed, clean up and return error
    if let Err(e) = extract_result {
        println!("Extraction failed for {}: {}", filename, e);
        
        // Remove the downloaded zip file
        let _ = std::fs::remove_file(&file_path);
        
        // Try to clean up any partially extracted files
        if extract_dir.exists() {
            println!("Cleaning up partial extraction at {}", extract_dir.display());
            let _ = std::fs::remove_dir_all(&extract_dir);
        }
        
        return Err(e);
    }

    println!("Extraction completed successfully for {}", filename);

    // Remove the zip file after successful extraction
    if let Err(e) = std::fs::remove_file(&file_path) {
        eprintln!(
            "Warning: Failed to remove zip file after successful extraction: {}",
            e
        );
        // Don't fail the operation just because we couldn't clean up the zip
    }

    Ok(())
}
