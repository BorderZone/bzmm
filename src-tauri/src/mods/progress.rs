use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub mod_name: String,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub progress_percent: f32,
}

pub fn calculate_progress(downloaded: u64, total_size: u64) -> f32 {
    if total_size > 0 {
        (downloaded as f32 / total_size as f32) * 100.0
    } else {
        0.0
    }
}