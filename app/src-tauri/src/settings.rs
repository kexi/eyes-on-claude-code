use std::fs;
use std::path::PathBuf;
use tauri::Manager;

use crate::state::Settings;

/// Get the config directory using Tauri's path API
pub fn get_config_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_data_dir()
        .map_err(|e| format!("Failed to get app data dir: {:?}", e))
}

/// Get the log directory (~/.eocc/logs)
/// This path matches the hook script's log directory
pub fn get_log_dir(_app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let home = dirs::home_dir().ok_or("Failed to get home directory")?;
    Ok(home.join(".eocc").join("logs"))
}

/// Get the application log directory (Tauri's log directory)
/// This is where tauri-plugin-log writes log files
pub fn get_app_log_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_log_dir()
        .map_err(|e| format!("Failed to get app log dir: {:?}", e))
}

pub fn get_events_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    get_log_dir(app).map(|dir| dir.join("events.jsonl"))
}

pub fn get_settings_file(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    get_config_dir(app).map(|dir| dir.join("settings.json"))
}

pub fn load_settings(app: &tauri::AppHandle) -> Settings {
    let settings_file = match get_settings_file(app) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[eocc] Cannot determine settings file path: {}", e);
            return Settings::default();
        }
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

pub fn save_settings(app: &tauri::AppHandle, settings: &Settings) {
    let config_dir = match get_config_dir(app) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("[eocc] Cannot save settings: {}", e);
            return;
        }
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
