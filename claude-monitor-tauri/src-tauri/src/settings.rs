use std::fs;
use std::path::PathBuf;

use crate::state::Settings;

pub fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude-monitor")
}

pub fn get_log_dir() -> PathBuf {
    get_config_dir().join("logs")
}

pub fn get_events_file() -> PathBuf {
    get_log_dir().join("events.jsonl")
}

pub fn get_settings_file() -> PathBuf {
    get_config_dir().join("settings.json")
}

pub fn load_settings() -> Settings {
    let settings_file = get_settings_file();
    if settings_file.exists() {
        if let Ok(content) = fs::read_to_string(&settings_file) {
            if let Ok(settings) = serde_json::from_str(&content) {
                return settings;
            }
        }
    }
    Settings::default()
}

pub fn save_settings(settings: &Settings) {
    let settings_file = get_settings_file();
    if let Ok(content) = serde_json::to_string_pretty(settings) {
        let _ = fs::create_dir_all(get_config_dir());
        let _ = fs::write(settings_file, content);
    }
}
