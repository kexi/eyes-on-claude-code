use tauri::{Emitter, Manager};

use crate::menu::build_tray_menu;
use crate::state::AppState;

pub fn update_tray_and_badge(app: &tauri::AppHandle, state: &AppState) {
    // Update tray menu
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok(new_menu) = build_tray_menu(app, state) {
            let _ = tray.set_menu(Some(new_menu));
        }

        let waiting_count = state.waiting_session_count();

        // Update tooltip
        let tooltip = if waiting_count > 0 {
            format!("Claude Monitor - {} waiting", waiting_count)
        } else if state.sessions.is_empty() {
            "Claude Monitor - No active sessions".to_string()
        } else {
            "Claude Monitor".to_string()
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }

    // Update badge count using the dashboard window
    if let Some(window) = app.get_webview_window("dashboard") {
        let waiting_count = state.waiting_session_count();
        let badge_count = if waiting_count > 0 {
            Some(waiting_count as i64)
        } else {
            None
        };
        let _ = window.set_badge_count(badge_count);
    }
}

pub fn emit_state_update(app: &tauri::AppHandle, state: &AppState) {
    let data = state.to_dashboard_data();
    let _ = app.emit("state-updated", &data);
}
