use std::collections::{VecDeque, HashMap};
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tokio_util::sync::CancellationToken;
use tauri::Emitter;

const MAX_CONCURRENT_DOWNLOADS: usize = 2;

#[derive(Clone, Debug)]
pub struct QueuedDownload {
    pub url: String,
    pub filename: String,
    pub repo_url: String,
}

#[derive(Clone)]
pub struct DownloadQueue {
    queue: Arc<Mutex<VecDeque<QueuedDownload>>>,
    semaphore: Arc<Semaphore>,
    cancel_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
}

impl DownloadQueue {
    pub fn new() -> Self {
        Self {
            queue: Arc::new(Mutex::new(VecDeque::new())),
            semaphore: Arc::new(Semaphore::new(MAX_CONCURRENT_DOWNLOADS)),
            cancel_tokens: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn add_download(&self, app_handle: tauri::AppHandle, url: String, filename: String, repo_url: String) {
        let download = QueuedDownload { url, filename: filename.clone(), repo_url };
        
        // Add to queue
        {
            let mut queue = self.queue.lock().await;
            queue.push_back(download);
        }

        // Emit queued event
        if let Err(e) = app_handle.emit("download-queued", &filename) {
            eprintln!("Failed to emit download-queued event: {}", e);
        }

        // Start processing - this spawns a task to avoid Send issues
        let queue_ref = self.clone();
        tokio::spawn(async move {
            queue_ref.process_one_download(app_handle).await;
        });
    }

    pub async fn cancel_download(&self, filename: &str) -> Result<(), String> {
        let mut was_queued = false;
        let mut was_downloading = false;

        // Remove from queue if still queued
        {
            let mut queue = self.queue.lock().await;
            let original_len = queue.len();
            queue.retain(|download| download.filename != filename);
            was_queued = queue.len() < original_len;
        }

        // Cancel ongoing download if it exists
        {
            let mut cancel_tokens = self.cancel_tokens.lock().await;
            if let Some(token) = cancel_tokens.remove(filename) {
                token.cancel();
                was_downloading = true;
                println!("Cancelled ongoing download for: {}", filename);
            }
        }

        // Clean up any temporary files
        if was_downloading {
            if let Err(e) = self.cleanup_download_files(filename).await {
                eprintln!("Warning: Failed to clean up files for {}: {}", filename, e);
            }
        }

        if was_queued || was_downloading {
            println!("Successfully cancelled download for: {}", filename);
        } else {
            println!("No active download found for: {}", filename);
        }

        Ok(())
    }

    async fn cleanup_download_files(&self, filename: &str) -> Result<(), String> {
        use crate::settings;
        use std::path::PathBuf;

        let settings = settings::Settings::load().map_err(|e| e.to_string())?;
        let base_downloads_dir = PathBuf::from(&settings.download_path);
        
        // Try to find and remove any temporary files matching this filename
        let temp_filename = format!("{}.tmp", filename.trim_end_matches(".zip"));
        
        // Search through all subdirectories for the temp file
        if let Ok(entries) = std::fs::read_dir(&base_downloads_dir) {
            for entry in entries.flatten() {
                if entry.file_type().map(|ft| ft.is_dir()).unwrap_or(false) {
                    let subdir = entry.path();
                    let temp_path = subdir.join(&temp_filename);
                    let final_path = subdir.join(filename);
                    
                    // Remove temporary file if it exists
                    if temp_path.exists() {
                        if let Err(e) = std::fs::remove_file(&temp_path) {
                            eprintln!("Failed to remove temp file {}: {}", temp_path.display(), e);
                        } else {
                            println!("Cleaned up temp file: {}", temp_path.display());
                        }
                    }
                    
                    // Remove final file if it exists (partial download)
                    if final_path.exists() {
                        if let Err(e) = std::fs::remove_file(&final_path) {
                            eprintln!("Failed to remove partial file {}: {}", final_path.display(), e);
                        } else {
                            println!("Cleaned up partial file: {}", final_path.display());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn process_one_download(&self, app_handle: tauri::AppHandle) {
        // Wait for a permit (blocking)
        let permit = self.semaphore.clone().acquire_owned().await.unwrap();

        // Get next download from queue
        let download = {
            let mut queue = self.queue.lock().await;
            queue.pop_front()
        };

        if let Some(download) = download {
            // Create cancellation token for this download
            let cancel_token = CancellationToken::new();
            {
                let mut cancel_tokens = self.cancel_tokens.lock().await;
                cancel_tokens.insert(download.filename.clone(), cancel_token.clone());
            }

            // Actually perform the download with cancellation support
            let result = super::mod_download::download_mod_with_cancellation(
                app_handle.clone(),
                download.url,
                download.filename.clone(),
                download.repo_url,
                cancel_token.clone(),
            ).await;

            // Clean up cancellation token after download+extraction completes (success or failure)
            {
                let mut cancel_tokens = self.cancel_tokens.lock().await;
                cancel_tokens.remove(&download.filename);
            }

            if let Err(e) = result {
                eprintln!("Download failed: {}", e);
            }
        }

        // Permit is automatically released when dropped
        drop(permit);
    }
}

// Global queue instance
static DOWNLOAD_QUEUE: std::sync::OnceLock<DownloadQueue> = std::sync::OnceLock::new();

pub fn get_queue() -> &'static DownloadQueue {
    DOWNLOAD_QUEUE.get_or_init(|| DownloadQueue::new())
}

#[tauri::command]
pub async fn queue_download(
    app_handle: tauri::AppHandle,
    url: String,
    filename: String,
    repo_url: String,
) -> Result<(), String> {
    println!("Queuing download: {} from {} (Repo: {})", filename, url, repo_url);
    
    let queue = get_queue();
    queue.add_download(app_handle, url, filename, repo_url).await;
    
    Ok(())
}

#[tauri::command]
pub async fn cancel_download(
    app_handle: tauri::AppHandle,
    filename: String,
) -> Result<(), String> {
    println!("Cancelling download: {}", filename);
    
    let queue = get_queue();
    queue.cancel_download(&filename).await?;
    
    // Emit cancellation event
    if let Err(e) = app_handle.emit("download-cancelled", &filename) {
        eprintln!("Failed to emit download-cancelled event: {}", e);
    }
    
    Ok(())
}

