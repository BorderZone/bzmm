//! Mod manager internal state.
//!
//! Provides the mod manager with the ability to hold information in memory to be used by handlers
use std::sync::{RwLock, RwLockReadGuard, RwLockWriteGuard};

use crate::settings::Settings;

/// Stores any state held in memory by the mod manager
#[derive(Debug)]
pub struct AppState {
    settings: RwLock<Settings>,
}

impl AppState {
    /// Constructor for the application state
    ///
    /// # Examples
    /// ```
    /// # use bzmm_lib::settings::Settings;
    /// # use bzmm_lib::state::AppState;
    /// let settings = Settings::default();
    /// AppState::new(settings);
    /// ```
    pub fn new(settings: Settings) -> Self {
        Self {
            settings: RwLock::new(settings),
        }
    }

    /// Obtain a write guard on the settings
    ///
    /// # Examples
    /// ```
    /// # use bzmm_lib::state::AppState;
    /// let state = AppState::default();
    /// {
    ///     let mut settings = state.write_settings().expect("To get a write guard on settings");
    ///     settings.download_path = "./my_downloads".to_string();
    /// }
    ///
    /// # let snapshot = state.settings_snapshot().expect("To get a snapshot");
    /// # assert_eq!(snapshot.download_path, "./my_downloads".to_string());
    /// ```
    pub fn write_settings(&self) -> Result<RwLockWriteGuard<'_, Settings>, String> {
        self.settings.write().map_err(|e| e.to_string())
    }

    /// Get a read only lock on the settings
    ///
    /// # Examples
    /// ```
    /// # use bzmm_lib::state::AppState;
    /// let state = AppState::default();
    /// let settings = state.read_settings().expect("To get a read guard on settings");
    /// ```
    pub fn read_settings(&self) -> Result<RwLockReadGuard<'_, Settings>, String> {
        self.settings.read().map_err(|e| e.to_string())
    }

    /// Get a clone of the settings in the state
    ///
    /// # Examples
    /// ```
    /// # use bzmm_lib::state::AppState;
    /// let state = AppState::default();
    /// let settings = state.settings_snapshot().expect("To get a clone of the settings");
    /// ```
    pub fn settings_snapshot(&self) -> Result<Settings, String> {
        self.read_settings().map(|settings| settings.clone())
    }

    /// Save the settings into the state
    ///
    /// # Examples
    /// ```
    /// # use bzmm_lib::settings::Settings;
    /// # use bzmm_lib::state::AppState;
    /// let settings = Settings::default();
    /// let mut new_settings = settings.clone();
    /// new_settings.sideload_path = "./my_sideloads".to_string();
    ///
    /// let state = AppState::new(settings);
    /// state.update_settings(&new_settings).expect("Settings should save successfully");
    /// # let saved_settings = state.settings_snapshot().unwrap();
    /// # assert_eq!(saved_settings.sideload_path, "./my_sideloads".to_string());
    /// ```
    /// ```
    /// # use bzmm_lib::settings::Settings;
    /// # use bzmm_lib::state::AppState;
    /// let settings = Settings::default();
    /// let mut new_settings = settings.clone();
    /// new_settings.sideload_path = "./my_sideloads".to_string();
    ///
    /// let state = AppState::new(settings);
    /// state.update_settings(new_settings).expect("Settings should save successfully");
    /// # let saved_settings = state.settings_snapshot().unwrap();
    /// # assert_eq!(saved_settings.sideload_path, "./my_sideloads".to_string());
    /// ```
    pub fn update_settings(&self, settings: impl AsRef<Settings>) -> Result<(), String> {
        let mut settings = settings.as_ref().clone();
        let mut app_settings = self.write_settings()?;
        std::mem::swap(&mut *app_settings, &mut settings);
        app_settings.save()
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new(Settings::default())
    }
}
