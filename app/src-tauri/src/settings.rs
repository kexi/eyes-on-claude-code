use std::fs;
use std::path::PathBuf;

use crate::state::Settings;

pub fn get_config_dir() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".eocc"))
}

pub fn get_log_dir() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join("logs"))
}

pub fn get_events_file() -> Option<PathBuf> {
    get_log_dir().map(|dir| dir.join("events.jsonl"))
}

pub fn get_settings_file() -> Option<PathBuf> {
    get_config_dir().map(|dir| dir.join("settings.json"))
}

pub fn load_settings() -> Settings {
    let Some(settings_file) = get_settings_file() else {
        eprintln!("[eocc] Cannot determine settings file path: home directory not found");
        return Settings::default();
    };

    if settings_file.exists() {
        match fs::read_to_string(&settings_file) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(settings) => return settings,
                Err(e) => eprintln!("[eocc] Failed to parse settings file: {:?}", e),
            },
            Err(e) => eprintln!("[eocc] Failed to read settings file: {:?}", e),
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) {
    let Some(config_dir) = get_config_dir() else {
        eprintln!("[eocc] Cannot save settings: home directory not found");
        return;
    };

    let settings_file = config_dir.join("settings.json");

    let content = match serde_json::to_string_pretty(settings) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("[eocc] Failed to serialize settings: {:?}", e);
            return;
        }
    };

    if let Err(e) = fs::create_dir_all(&config_dir) {
        eprintln!("[eocc] Failed to create config directory: {:?}", e);
        return;
    }

    if let Err(e) = fs::write(&settings_file, content) {
        eprintln!("[eocc] Failed to write settings file: {:?}", e);
    }
}
