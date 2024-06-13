use serde::{Deserialize, Serialize};
use serde_json;

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
        let settings_path = crate::utils::settings_path();
        match settings_path.is_file() {
            true => match std::fs::read_to_string(&settings_path) {
                Ok(settings) => match serde_json::from_str(&settings) {
                    Ok(settings) => return settings,
                    Err(e) => log::error!("Json Parse Error:{e}"),
                },
                Err(e) => log::error!("Read File Error:{e}"),
            },
            false => {
                let display_path = settings_path.to_str().unwrap_or_default();
                log::info!("\"{display_path}\" is not a file.");
            }
        }
        Self::default()
    }
    pub fn save(&self) -> Result<(), String> {
        let settings_path = crate::utils::settings_path();
        if let Ok(settings) = serde_json::to_string_pretty(self) {
            if let Err(e) = std::fs::write(&settings_path, settings) {
                return Err(e.to_string());
            }
        } else {
            return Err("Failed to serialize settings".to_string());
        }
        Ok(())
    }
}
