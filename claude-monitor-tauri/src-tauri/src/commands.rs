use tauri::Manager;

use crate::constants::{MINI_VIEW_HEIGHT, MINI_VIEW_WIDTH, NORMAL_VIEW_HEIGHT, NORMAL_VIEW_WIDTH};
use crate::difit::{open_difit, DiffType};
use crate::git::{get_git_info, GitInfo};
use crate::settings::save_settings;
use crate::state::{DashboardData, ManagedState, Settings};
use crate::tray::{emit_state_update, update_tray_and_badge};

const LOCK_ERROR: &str = "Failed to acquire state lock";

#[tauri::command]
pub fn get_dashboard_data(state: tauri::State<'_, ManagedState>) -> Result<DashboardData, String> {
    let state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    Ok(state_guard.to_dashboard_data())
}

#[tauri::command]
pub fn remove_session(
    project_dir: String,
    state: tauri::State<'_, ManagedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.sessions.remove(&project_dir);
    update_tray_and_badge(&app, &state_guard);
    emit_state_update(&app, &state_guard);
    Ok(())
}

#[tauri::command]
pub fn clear_all_sessions(
    state: tauri::State<'_, ManagedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.sessions.clear();
    update_tray_and_badge(&app, &state_guard);
    emit_state_update(&app, &state_guard);
    Ok(())
}

#[tauri::command]
pub fn get_always_on_top(state: tauri::State<'_, ManagedState>) -> Result<bool, String> {
    let state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    Ok(state_guard.settings.always_on_top)
}

#[tauri::command]
pub fn set_always_on_top(
    enabled: bool,
    state: tauri::State<'_, ManagedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.settings.always_on_top = enabled;
    save_settings(&state_guard.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_always_on_top(enabled);
    }

    update_tray_and_badge(&app, &state_guard);
    Ok(())
}

#[tauri::command]
pub fn get_mini_view(state: tauri::State<'_, ManagedState>) -> Result<bool, String> {
    let state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    Ok(state_guard.settings.mini_view)
}

#[tauri::command]
pub fn set_mini_view(
    enabled: bool,
    state: tauri::State<'_, ManagedState>,
    app: tauri::AppHandle,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.settings.mini_view = enabled;
    save_settings(&state_guard.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_decorations(!enabled);
        if enabled {
            let _ = window.set_size(tauri::LogicalSize::new(MINI_VIEW_WIDTH, MINI_VIEW_HEIGHT));
        } else {
            let _ = window.set_size(tauri::LogicalSize::new(NORMAL_VIEW_WIDTH, NORMAL_VIEW_HEIGHT));
            let _ = window.center();
        }
    }

    update_tray_and_badge(&app, &state_guard);
    Ok(())
}

#[tauri::command]
pub fn get_settings(state: tauri::State<'_, ManagedState>) -> Result<Settings, String> {
    let state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    Ok(state_guard.settings.clone())
}

#[tauri::command]
pub fn set_opacity_active(
    opacity: f64,
    state: tauri::State<'_, ManagedState>,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.settings.opacity_active = opacity.clamp(0.1, 1.0);
    save_settings(&state_guard.settings);
    Ok(())
}

#[tauri::command]
pub fn set_opacity_inactive(
    opacity: f64,
    state: tauri::State<'_, ManagedState>,
) -> Result<(), String> {
    let mut state_guard = state.0.lock().map_err(|_| LOCK_ERROR)?;
    state_guard.settings.opacity_inactive = opacity.clamp(0.1, 1.0);
    save_settings(&state_guard.settings);
    Ok(())
}

#[tauri::command]
pub fn get_repo_git_info(project_dir: String) -> GitInfo {
    get_git_info(&project_dir)
}

#[tauri::command]
pub fn open_diff(project_dir: String, diff_type: String, base_branch: Option<String>) -> Result<(), String> {
    let diff = match diff_type.as_str() {
        "unstaged" => DiffType::Unstaged,
        "commit" => DiffType::LatestCommit,
        "branch" => DiffType::Branch,
        _ => return Err(format!("Unknown diff type: {}", diff_type)),
    };

    open_difit(&project_dir, diff, base_branch.as_deref())
}
