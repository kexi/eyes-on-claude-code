use std::path::Path;
use std::sync::Arc;
use tauri::{Manager, WebviewUrl, WebviewWindowBuilder};

use crate::constants::{MINI_VIEW_HEIGHT, MINI_VIEW_WIDTH, NORMAL_VIEW_HEIGHT, NORMAL_VIEW_WIDTH};
use crate::difit::{start_difit_server, DiffType, DifitProcessRegistry};
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

/// Generate a unique window label for a diff based on project and type
fn generate_diff_window_label(project_dir: &str, diff_type: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    project_dir.hash(&mut hasher);
    diff_type.hash(&mut hasher);
    format!("difit-{:x}", hasher.finish())
}

#[tauri::command]
pub fn open_diff(
    project_dir: String,
    diff_type: String,
    base_branch: Option<String>,
    app: tauri::AppHandle,
    difit_registry: tauri::State<'_, Arc<DifitProcessRegistry>>,
) -> Result<(), String> {
    // Validate project directory
    let path = Path::new(&project_dir);
    if !path.exists() {
        return Err(format!("Directory does not exist: {}", project_dir));
    }
    if !path.is_dir() {
        return Err(format!("Path is not a directory: {}", project_dir));
    }
    // Check if it's a git repository
    if !path.join(".git").exists() {
        return Err(format!("Not a git repository: {}", project_dir));
    }

    // Generate unique window label based on project and diff type
    let window_label = generate_diff_window_label(&project_dir, &diff_type);

    // Check if window already exists - if so, focus it and return
    if let Some(existing_window) = app.get_webview_window(&window_label) {
        let _ = existing_window.show();
        let _ = existing_window.set_focus();
        return Ok(());
    }

    let diff = match diff_type.as_str() {
        "unstaged" => DiffType::Unstaged,
        "commit" => DiffType::LatestCommit,
        "branch" => DiffType::Branch,
        _ => return Err(format!("Unknown diff type: {}", diff_type)),
    };

    // Get next available port
    let port = difit_registry.get_next_port();

    // Start difit server
    let server_info = start_difit_server(&project_dir, diff, base_branch.as_deref(), port)?;

    // Create a new window for the diff viewer
    let window = WebviewWindowBuilder::new(
        &app,
        &window_label,
        WebviewUrl::External(server_info.url.parse().map_err(|e| format!("Invalid URL: {}", e))?),
    )
    .title(format!("Diff - {}", diff_type))
    .inner_size(1200.0, 800.0)
    .center()
    .build()
    .map_err(|e| format!("Failed to create diff window: {}", e))?;

    // Register the process with the window label
    difit_registry.register(window_label.clone(), server_info.process);

    // Set up window close handler to kill the difit process
    let registry_clone = Arc::clone(&difit_registry);
    let label_clone = window_label.clone();
    window.on_window_event(move |event| {
        if let tauri::WindowEvent::Destroyed = event {
            registry_clone.kill(&label_clone);
        }
    });

    Ok(())
}
