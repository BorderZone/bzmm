use super::parser::ModParser;
use super::progress::{calculate_progress, DownloadProgress};
use super::types::{ModError, ModsFile};
use futures_util::StreamExt;
use reqwest::Client;
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

pub struct ModDownloader {
    client: Client,
}

impl ModDownloader {
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent("BZMM/1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client }
    }

    pub async fn fetch_mod_list(&self, url: &str) -> Result<String, ModError> {
        Ok(self.client.get(url).send().await?.text().await?)
    }

    #[tracing::instrument(skip(self, app_handle))]
    pub async fn download_mod(
        &self,
        app_handle: tauri::AppHandle,
        url: &str,
        path: &std::path::Path,
        mod_name: &str,
    ) -> Result<(), ModError> {
        // Function to emit error event
        let emit_error = |e: &ModError| {
            tracing::warn!("Download error for {}: {:?}", mod_name, e);
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": mod_name,
                    "error": e.to_string()
                }),
            );
        };

        // Validate URL
        tracing::info!("Download started for {} from URL: '{}'", mod_name, url);
        if url.is_empty() || !url.starts_with("http") {
            tracing::warn!("Invalid URL for {}: '{}'", mod_name, url);
            let err = ModError::InvalidUrl(format!("Invalid URL provided: {}", url));
            emit_error(&err);
            return Err(err);
        }

        // Emit download started event
        if let Err(e) = app_handle.emit("download-started", mod_name) {
            let err = ModError::TauriError(e);
            emit_error(&err);
            return Err(err);
        }

        // First make a HEAD request to get the content length
        let resp = match self.client.head(url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("HEAD request failed for {}: {}", mod_name, e);
                let err = ModError::RequestError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        let total_size = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse().ok())
            .unwrap_or(0u64);

        tracing::info!("Starting download of {} bytes for {}", total_size, mod_name);

        // Now make the actual download request
        let res = match self.client.get(url).send().await {
            Ok(r) => {
                // Check if the response is successful (status code 200-299)
                if !r.status().is_success() {
                    let status = r.status();
                    let error_text = r
                        .text()
                        .await
                        .unwrap_or_else(|_| format!("HTTP Error: {}", status));
                    tracing::error!("HTTP error for {}: {} - {}", mod_name, status, error_text);
                    let err = ModError::HttpError(format!(
                        "Server returned error: {} - {}",
                        status, error_text
                    ));
                    emit_error(&err);
                    return Err(err);
                }
                r
            }
            Err(e) => {
                tracing::error!("GET request failed for {}: {}", mod_name, e);
                let err = ModError::RequestError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut last_emitted_percent = 0i32;

        let mut file = match tokio::fs::File::create(path).await {
            Ok(f) => f,
            Err(e) => {
                tracing::error!("Failed to create file {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        use tokio::io::AsyncWriteExt;

        tracing::info!("Downloading to path: {}", path.display());
        while let Some(chunk) = stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Download stream error for {}: {}", mod_name, e);
                    let err = ModError::RequestError(e);
                    emit_error(&err);
                    return Err(err);
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                tracing::error!("Failed to write chunk to file {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }

            downloaded += chunk.len() as u64;
            let progress = calculate_progress(downloaded, total_size);

            // Get the current percentage as an integer
            let current_percent = progress.floor() as i32;

            // Only emit if we've crossed a whole percentage point
            if current_percent > last_emitted_percent {
                if let Err(e) = app_handle.emit(
                    "download-progress",
                    DownloadProgress {
                        mod_name: mod_name.to_string(),
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        progress_percent: progress,
                    },
                ) {
                    let err = ModError::TauriError(e);
                    emit_error(&err);
                    return Err(err);
                }
                last_emitted_percent = current_percent;
            }
        }

        // Ensure file is flushed and closed correctly
        if let Err(e) = file.flush().await {
            tracing::error!("Failed to flush file {}: {}", path.display(), e);
            let err = ModError::IoError(e);
            emit_error(&err);
            return Err(err);
        }

        if let Err(e) = file.sync_all().await {
            tracing::error!("Failed to sync file {}: {}", path.display(), e);
            // Log but continue, as this is not critical
        }

        // Drop the file handle to ensure it's closed
        drop(file);

        // Always emit 100% at the end
        if last_emitted_percent < 100 {
            if let Err(e) = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    mod_name: mod_name.to_string(),
                    downloaded_bytes: total_size,
                    total_bytes: total_size,
                    progress_percent: 100.0,
                },
            ) {
                let err = ModError::TauriError(e);
                emit_error(&err);
                return Err(err);
            }
        }

        // Verify the downloaded file exists and has content
        let metadata = match tokio::fs::metadata(path).await {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to get metadata for {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        if metadata.len() == 0 {
            tracing::error!("Downloaded file is empty: {}", path.display());
            let err = ModError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Downloaded file is empty",
            ));
            emit_error(&err);
            return Err(err);
        }

        // Emit completion event
        tracing::info!(
            "Download completed for {} - File size: {} bytes",
            mod_name,
            metadata.len()
        );
        if let Err(e) = app_handle.emit("download-complete", mod_name) {
            let err = ModError::TauriError(e);
            emit_error(&err);
            return Err(err);
        }

        // Note: Queue processing will be triggered when new downloads are added

        Ok(())
    }

    #[tracing::instrument(skip(self, app_handle))]
    pub async fn download_mod_with_cancellation(
        &self,
        app_handle: tauri::AppHandle,
        url: &str,
        path: &std::path::Path,
        mod_name: &str,
        cancel_token: &CancellationToken,
    ) -> Result<(), ModError> {
        // Function to emit error event
        let emit_error = |e: &ModError| {
            tracing::error!("Download error for {}: {:?}", mod_name, e);
            let _ = app_handle.emit(
                "download-error",
                serde_json::json!({
                    "mod_name": mod_name,
                    "error": e.to_string()
                }),
            );
        };

        // Check if cancelled at start
        if cancel_token.is_cancelled() {
            return Err(ModError::IoError(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Download was cancelled",
            )));
        }

        // Validate URL
        tracing::debug!("Download started for {} from URL: '{}'", mod_name, url);
        if url.is_empty() || !url.starts_with("http") {
            tracing::error!("Invalid URL for {}: '{}'", mod_name, url);
            let err = ModError::InvalidUrl(format!("Invalid URL provided: {}", url));
            emit_error(&err);
            return Err(err);
        }

        // Emit download started event
        if let Err(e) = app_handle.emit("download-started", mod_name) {
            let err = ModError::TauriError(e);
            emit_error(&err);
            return Err(err);
        }

        // First make a HEAD request to get the content length
        let resp = match self.client.head(url).send().await {
            Ok(r) => r,
            Err(e) => {
                tracing::error!("HEAD request failed for {}: {}", mod_name, e);
                let err = ModError::RequestError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        let total_size = resp
            .headers()
            .get(reqwest::header::CONTENT_LENGTH)
            .and_then(|ct_len| ct_len.to_str().ok())
            .and_then(|ct_len| ct_len.parse().ok())
            .unwrap_or(0u64);

        tracing::debug!("Starting download of {} bytes for {}", total_size, mod_name);

        // Check if cancelled before main download
        if cancel_token.is_cancelled() {
            return Err(ModError::IoError(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Download was cancelled",
            )));
        }

        // Now make the actual download request
        let res = match self.client.get(url).send().await {
            Ok(r) => {
                // Check if the response is successful (status code 200-299)
                if !r.status().is_success() {
                    let status = r.status();
                    let error_text = r
                        .text()
                        .await
                        .unwrap_or_else(|_| format!("HTTP Error: {}", status));
                    tracing::info!("HTTP error for {}: {} - {}", mod_name, status, error_text);
                    let err = ModError::HttpError(format!(
                        "Server returned error: {} - {}",
                        status, error_text
                    ));
                    emit_error(&err);
                    return Err(err);
                }
                r
            }
            Err(e) => {
                tracing::warn!("GET request failed for {}: {}", mod_name, e);
                let err = ModError::RequestError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        let mut downloaded: u64 = 0;
        let mut stream = res.bytes_stream();
        let mut last_emitted_percent = 0i32;

        let mut file = match tokio::fs::File::create(path).await {
            Ok(f) => f,
            Err(e) => {
                tracing::info!("Failed to create file {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        use tokio::io::AsyncWriteExt;

        tracing::debug!("Downloading to path: {}", path.display());
        while let Some(chunk) = stream.next().await {
            // Check if cancelled during download
            if cancel_token.is_cancelled() {
                return Err(ModError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "Download was cancelled",
                )));
            }

            let chunk = match chunk {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("Download stream error for {}: {}", mod_name, e);
                    let err = ModError::RequestError(e);
                    emit_error(&err);
                    return Err(err);
                }
            };

            if let Err(e) = file.write_all(&chunk).await {
                tracing::error!("Failed to write chunk to file {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }

            downloaded += chunk.len() as u64;
            let progress = calculate_progress(downloaded, total_size);

            // Get the current percentage as an integer
            let current_percent = progress.floor() as i32;

            // Only emit if we've crossed a whole percentage point
            if current_percent > last_emitted_percent {
                if let Err(e) = app_handle.emit(
                    "download-progress",
                    DownloadProgress {
                        mod_name: mod_name.to_string(),
                        downloaded_bytes: downloaded,
                        total_bytes: total_size,
                        progress_percent: progress,
                    },
                ) {
                    let err = ModError::TauriError(e);
                    emit_error(&err);
                    return Err(err);
                }
                last_emitted_percent = current_percent;
            }
        }

        // Check if cancelled before finalizing
        if cancel_token.is_cancelled() {
            return Err(ModError::IoError(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "Download was cancelled",
            )));
        }

        // Ensure file is flushed and closed correctly
        if let Err(e) = file.flush().await {
            tracing::error!("Failed to flush file {}: {}", path.display(), e);
            let err = ModError::IoError(e);
            emit_error(&err);
            return Err(err);
        }

        if let Err(e) = file.sync_all().await {
            tracing::error!("Failed to sync file {}: {}", path.display(), e);
            // Log but continue, as this is not critical
        }

        // Drop the file handle to ensure it's closed
        drop(file);

        // Always emit 100% at the end
        if last_emitted_percent < 100 {
            if let Err(e) = app_handle.emit(
                "download-progress",
                DownloadProgress {
                    mod_name: mod_name.to_string(),
                    downloaded_bytes: total_size,
                    total_bytes: total_size,
                    progress_percent: 100.0,
                },
            ) {
                let err = ModError::TauriError(e);
                emit_error(&err);
                return Err(err);
            }
        }

        // Verify the downloaded file exists and has content
        let metadata = match tokio::fs::metadata(path).await {
            Ok(m) => m,
            Err(e) => {
                tracing::error!("Failed to get metadata for {}: {}", path.display(), e);
                let err = ModError::IoError(e);
                emit_error(&err);
                return Err(err);
            }
        };

        if metadata.len() == 0 {
            tracing::error!("Downloaded file is empty: {}", path.display());
            let err = ModError::IoError(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "Downloaded file is empty",
            ));
            emit_error(&err);
            return Err(err);
        }

        // Emit completion event
        tracing::debug!(
            "Download completed for {} - File size: {} bytes",
            mod_name,
            metadata.len()
        );
        if let Err(e) = app_handle.emit("download-complete", mod_name) {
            let err = ModError::TauriError(e);
            emit_error(&err);
            return Err(err);
        }

        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn fetch_and_parse_mods(
        &self,
        url: &str,
    ) -> Result<(ModsFile, Option<std::path::PathBuf>), ModError> {
        let xml_content = self.fetch_mod_list(url).await?;
        let mods_file = ModParser::parse_mod_list(&xml_content)?;

        // Save the successful XML to cache
        let cache_path = match super::xml_cache::XmlCache::save_xml(url, &xml_content) {
            Ok(path) => Some(path),
            Err(e) => {
                tracing::warn!(url, "Failed to cache XML: {}", e);
                None
            }
        };

        Ok((mods_file, cache_path))
    }
}

impl Default for ModDownloader {
    fn default() -> Self {
        Self::new()
    }
}
