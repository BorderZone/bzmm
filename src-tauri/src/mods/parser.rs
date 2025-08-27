use std::fs;
use std::path::Path;
use super::types::{ModError, ModsFile};
use quick_xml::de::from_str;
use sha2::{Digest, Sha256};

pub struct ModParser;

impl ModParser {
    pub fn parse_mod_list(xml: &str) -> Result<ModsFile, ModError> {
        let mods_file: ModsFile = from_str(xml)?;
        Ok(mods_file)
    }

    /// Checks for local updates against the provided XML mod list, considering the source repository URL.
    pub fn check_for_updates(
        xml_mods: &ModsFile,
        base_download_path: &Path,
        repo_url: &str,
    ) -> Result<ModsFile, ModError> {
        let mut updated_mods = xml_mods.clone();

        // Calculate the XML-specific path
        let mut hasher = Sha256::new();
        hasher.update(repo_url.as_bytes());
        let hash_result = hasher.finalize();
        let repo_hash = format!("{:x}", hash_result);
        let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
        let xml_specific_path = base_download_path.join(&repo_hash);

        println!("Checking for updates within: {}", xml_specific_path.display());

        for category in &mut updated_mods.categories {
            for mod_entry in &mut category.mods {
                println!("Checking updates for mod: {}", mod_entry.name);

                // Check if mod is downloaded within the XML-specific directory
                let mod_dir = xml_specific_path.join(&mod_entry.name);
                if !mod_dir.is_dir() {
                    // Mod not downloaded from this specific source
                    println!("Mod dir not found in XML-specific path: {:?}", mod_dir);
                    continue;
                }

                // Read VERSION.txt
                let version_path = mod_dir.join("VERSION.txt");
                if !version_path.exists() {
                    println!("VERSION.txt not found in {:?}", version_path);
                    continue;
                }

                if let Ok(local_version) = fs::read_to_string(version_path) {
                    let local_version = local_version.trim();
                    println!("Local version: {}, XML version: {}", local_version, mod_entry.version);
                    
                    // If XML version is different from local version, set newVersion
                    if local_version != mod_entry.version {
                        println!("Update found! Setting new_version to {}", mod_entry.version);
                        mod_entry.new_version = Some(mod_entry.version.clone());
                        mod_entry.version = local_version.to_string();
                    }
                }
            }
        }

        Ok(updated_mods)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use super::types::{Category, Mod};
    use std::path::PathBuf;

    // Helper to create a dummy repo hash for testing
    fn get_test_repo_hash(url: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(url.as_bytes());
        let repo_hash = format!("{:x}", hasher.finalize());
        let repo_hash = &repo_hash[..6]; // Shrink the hash to 6 characters
        format!("{:x}", repo_hash)
    }

    #[test]
    fn test_parse_mod_list() {
        let xml = r#"<?xml version="1.0"?>
        <mods>
            <category name="Essential" sort_order="1">
                <mod name="Test Mod" version="1.0.0" url="http://example.com/mod.zip">
                    Description text
                </mod>
            </category>
        </mods>"#;

        let result = ModParser::parse_mod_list(xml);
        assert!(result.is_ok());

        let mods = result.unwrap();
        assert_eq!(mods.categories.len(), 1);
        assert_eq!(mods.categories[0].name, "Essential");
        assert_eq!(mods.categories[0].mods.len(), 1);
        assert_eq!(mods.categories[0].mods[0].name, "Test Mod");
    }

    fn test_check_for_updates() {
        let base_temp_dir = tempdir().unwrap();
        let repo_url = "http://example.com/repo.xml";
        let repo_hash = get_test_repo_hash(repo_url);
        let xml_specific_path = base_temp_dir.path().join(repo_hash);
        fs::create_dir(&xml_specific_path).unwrap(); // Create the hashed subdir

        let mod_dir = xml_specific_path.join("Test Mod"); // Mod inside hashed subdir
        fs::create_dir(&mod_dir).unwrap();
        fs::write(mod_dir.join("VERSION.txt"), "1.0.0").unwrap();

        // Create another mod in a different repo's subdir to ensure isolation
        let other_repo_url = "http://another.com/repo.xml";
        let other_repo_hash = get_test_repo_hash(other_repo_url);
        let other_xml_specific_path = base_temp_dir.path().join(other_repo_hash);
        fs::create_dir(&other_xml_specific_path).unwrap();
        let other_mod_dir = other_xml_specific_path.join("Test Mod"); // Same mod name, different source
        fs::create_dir(&other_mod_dir).unwrap();
        fs::write(other_mod_dir.join("VERSION.txt"), "0.9.0").unwrap(); // Different version

        let mods = ModsFile {
            categories: vec![
                Category {
                    name: "Essential".to_string(),
                    sort_order: 1,
                    mods: vec![
                        Mod {
                            name: "Test Mod".to_string(),
                            version: "1.0.1".to_string(), // XML has newer version
                            url: Some("http://example.com/mod.zip".to_string()),
                            new_version: None,
                            description: "Test description".to_string(),
                        }
                    ],
                }
            ],
        };

        // Check against the first repo URL
        let result = ModParser::check_for_updates(&mods, base_temp_dir.path(), repo_url).unwrap();
        let updated_mod = &result.categories[0].mods[0];

        assert_eq!(updated_mod.version, "1.0.0"); // Local version from the correct subdir
        assert_eq!(updated_mod.new_version, Some("1.0.1".to_string())); // Available update

        // Check against the second repo URL (should not find the mod in its specific dir)
        let mods_for_other_repo = ModsFile {
             categories: vec![
                Category {
                    name: "Essential".to_string(),
                    sort_order: 1,
                    mods: vec![
                        Mod {
                            name: "Test Mod".to_string(), // Same mod name
                            version: "1.0.0".to_string(), // XML version
                            url: Some("http://another.com/mod.zip".to_string()),
                            new_version: None,
                            description: "Test description".to_string(),
                        }
                    ],
                }
            ],
        };
        let result_other = ModParser::check_for_updates(&mods_for_other_repo, base_temp_dir.path(), other_repo_url).unwrap();
        let updated_mod_other = &result_other.categories[0].mods[0];

        // Since the local version is 0.9.0 for this repo, it should be updated
        assert_eq!(updated_mod_other.version, "0.9.0"); // Local version from the second subdir
        assert_eq!(updated_mod_other.new_version, Some("1.0.0".to_string())); // Available update based on XML
    }
}
