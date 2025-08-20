use serde::{Deserialize, Serialize};
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::result::Result;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct BrowserInfo {
    pub name: String,
    pub path: String,
    pub cmd: Vec<String>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct Settings {
    pub browsers: Vec<BrowserInfo>,
    pub cols: usize,
    pub rows: usize,
    pub cache_expire_days: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            browsers: Vec::new(),
            cols: 3,
            rows: 2,
            cache_expire_days: 7,
        }
    }
}

impl Settings {
    pub fn load() -> Self {
        let path = crate::utils::settings_path();
        match Self::load_from_path(&path) {
            Ok(settings) => settings,
            Err(e) => {
                log::error!("Failed to load settings: {}", e);
                Self::default()
            }
        }
    }

    fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let content = match fs::read_to_string(&path) {
            Ok(content) => content,
            Err(e) => {
                return Err(Error::new(
                    e.kind(),
                    format!(
                        "Failed to read settings from {}: {}",
                        path.as_ref().display(),
                        e
                    ),
                ))
            }
        };

        match serde_json::from_str(&content) {
            Ok(settings) => Ok(settings),
            Err(e) => Err(Error::new(
                ErrorKind::InvalidData,
                format!("Failed to parse settings JSON: {}", e),
            )),
        }
    }

    pub fn create(&self) -> Result<(), Error> {
        let path = crate::utils::settings_path();
        self.save_to_path(&path)
    }

    fn save_to_path<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        // Create parent directory if it doesn't exist
        if let Some(parent) = path.as_ref().parent() {
            if let Err(e) = fs::create_dir_all(parent) {
                return Err(Error::new(
                    e.kind(),
                    format!("Failed to create directory {}: {}", parent.display(), e),
                ));
            }
        }

        let settings = match serde_json::to_string_pretty(self) {
            Ok(s) => s,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::InvalidData,
                    format!("Failed to serialize settings: {}", e),
                ))
            }
        };

        match fs::write(&path, settings) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::new(
                e.kind(),
                format!(
                    "Failed to write settings to {}: {}",
                    path.as_ref().display(),
                    e
                ),
            )),
        }
    }
}
