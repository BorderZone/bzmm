use std::fs;
use std::path::{Path, PathBuf};
use directories::ProjectDirs;
use super::types::{ModError, ModsFile};
use super::parser::ModParser;
use std::io;

/// Handler for caching and loading XML files
pub struct XmlCache;

impl XmlCache {
    /// Get the directory for cached XML files
    pub fn get_cache_dir() -> Option<PathBuf> {
        let proj_dirs = ProjectDirs::from("com", "borderzone", "bzmm")?;
        let cache_dir = proj_dirs.cache_dir().join("xml_cache");
        if let Err(e) = fs::create_dir_all(&cache_dir) {
            eprintln!("Failed to create XML cache directory: {}", e);
            return None;
        }
        Some(cache_dir)
    }

    /// Generate a filename for a cached XML based on the URL
    pub fn generate_cache_filename(url: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        url.hash(&mut hasher);
        let hash = hasher.finish();
        
        format!("repo_{}.xml", hash)
    }

    /// Save XML content to cache
    pub fn save_xml(url: &str, xml_content: &str) -> Result<PathBuf, ModError> {
        let cache_dir = Self::get_cache_dir()
            .ok_or_else(|| ModError::IoError(io::Error::new(
                io::ErrorKind::NotFound, 
                "Could not find or create cache directory"
            )))?;
        
        let filename = Self::generate_cache_filename(url);
        let file_path = cache_dir.join(&filename);
        
        fs::write(&file_path, xml_content)
            .map_err(ModError::IoError)?;
        
        println!("Saved XML cache to {}", file_path.display());
        Ok(file_path)
    }

    /// Load XML content from cache
    pub fn load_xml(path: &Path) -> Result<ModsFile, ModError> {
        if !path.exists() {
            return Err(ModError::IoError(io::Error::new(
                io::ErrorKind::NotFound,
                format!("Cached XML file not found: {}", path.display())
            )));
        }
        
        let xml_content = fs::read_to_string(path)
            .map_err(ModError::IoError)?;
        
        ModParser::parse_mod_list(&xml_content)
    }

    /// Get the cache path for a repo URL
    pub fn get_cache_path(url: &str) -> Option<PathBuf> {
        let cache_dir = Self::get_cache_dir()?;
        let filename = Self::generate_cache_filename(url);
        Some(cache_dir.join(filename))
    }
}

/// Add cache path to settings
pub fn update_cache_path_in_settings(
    settings: &mut crate::settings::Settings, 
    url: &str, 
    cache_path: &Path
) -> Result<(), String> {
    // Convert cache_path to string
    let cache_path_str = cache_path.to_string_lossy().to_string();
    
    // Find index for this URL
    let index = settings.profiles.iter().position(|p| p.repo_url == url);
    
    if let Some(index) = index {
        // Ensure the cached_xml_paths vector has enough elements
        while settings.cached_xml_paths.len() <= index {
            settings.cached_xml_paths.push(String::new());
        }
        
        // Update the cache path
        settings.cached_xml_paths[index] = cache_path_str;
        
        // Save settings
        settings.save()?;
    }
    
    Ok(())
}
