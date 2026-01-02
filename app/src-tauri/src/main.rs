#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod constants;
mod difit;
mod events;
mod git;
mod menu;
mod settings;
mod state;
mod tray;

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use std::fs;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::Color,
    Emitter, Manager, WebviewUrl, WebviewWindowBuilder,
};

use difit::DifitProcessRegistry;

use commands::{
    clear_all_sessions, get_always_on_top, get_dashboard_data, get_mini_view, get_repo_git_info,
    get_settings, open_diff, remove_session, set_always_on_top, set_mini_view, set_opacity_active,
    set_opacity_inactive,
};
use constants::{
    ICON_NORMAL, MINI_VIEW_HEIGHT, MINI_VIEW_WIDTH, NORMAL_VIEW_HEIGHT, NORMAL_VIEW_WIDTH,
};
use events::{process_event, read_new_events};
use menu::{build_app_menu, build_tray_menu, parse_opacity_menu_id};
use settings::{get_events_file, get_log_dir, load_settings, save_settings};
use state::{AppState, EventInfo, ManagedState};
use tray::{emit_state_update, update_tray_and_badge};

fn show_dashboard(app: &tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_always_on_top(app: &tauri::AppHandle, state: &mut AppState) {
    state.settings.always_on_top = !state.settings.always_on_top;
    save_settings(&state.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_always_on_top(state.settings.always_on_top);
    }
}

fn toggle_mini_view(app: &tauri::AppHandle, state: &mut AppState) {
    state.settings.mini_view = !state.settings.mini_view;
    save_settings(&state.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_decorations(!state.settings.mini_view);
        if state.settings.mini_view {
            let _ = window.set_size(tauri::LogicalSize::new(MINI_VIEW_WIDTH, MINI_VIEW_HEIGHT));
        } else {
            let _ = window.set_size(tauri::LogicalSize::new(
                NORMAL_VIEW_WIDTH,
                NORMAL_VIEW_HEIGHT,
            ));
            let _ = window.center();
        }
    }

    let _ = app.emit("settings-updated", &state.settings);
}

fn create_dashboard_window(
    app: &tauri::App,
    always_on_top: bool,
    mini_view: bool,
) -> tauri::Result<tauri::WebviewWindow> {
    let (width, height) = if mini_view {
        (MINI_VIEW_WIDTH, MINI_VIEW_HEIGHT)
    } else {
        (NORMAL_VIEW_WIDTH, NORMAL_VIEW_HEIGHT)
    };

    let transparent_color = Color(0, 0, 0, 0);

    let base_builder = WebviewWindowBuilder::new(app, "dashboard", WebviewUrl::App("index.html".into()))
        .title("Eyes on Claude Code")
        .inner_size(width, height)
        .min_inner_size(200.0, 300.0)
        .center()
        .visible(true)
        .always_on_top(always_on_top)
        .decorations(!mini_view)
        .transparent(true)
        .background_color(transparent_color);

    match Image::from_bytes(ICON_NORMAL) {
        Ok(icon) => base_builder.icon(icon)?.build(),
        Err(_) => base_builder.build(),
    }
}

fn start_file_watcher(app_handle: tauri::AppHandle, state: Arc<Mutex<AppState>>) {
    std::thread::spawn(move || {
        let Some(log_dir) = get_log_dir() else {
            eprintln!("[eocc] Cannot start file watcher: home directory not found");
            return;
        };
        if let Err(e) = fs::create_dir_all(&log_dir) {
            eprintln!("[eocc] Failed to create log directory: {:?}", e);
            return;
        }

        let (tx, rx) = std::sync::mpsc::channel();

        let mut watcher = match RecommendedWatcher::new(tx, Config::default()) {
            Ok(w) => w,
            Err(e) => {
                eprintln!("[eocc] Failed to create file watcher: {:?}", e);
                return;
            }
        };

        if let Err(e) = watcher.watch(&log_dir, RecursiveMode::NonRecursive) {
            eprintln!("[eocc] Failed to watch directory: {:?}", e);
            return;
        }

        loop {
            match rx.recv() {
                Ok(_event) => {
                    let Ok(mut state_guard) = state.lock() else {
                        eprintln!("[eocc] Failed to acquire state lock in watcher");
                        continue;
                    };
                    let new_events = read_new_events(&mut state_guard);

                    if !new_events.is_empty() {
                        update_tray_and_badge(&app_handle, &state_guard);
                        emit_state_update(&app_handle, &state_guard);
                    }
                }
                Err(e) => {
                    eprintln!("[eocc] Watch channel error: {:?}", e);
                    break;
                }
            }
        }
    });
}

fn load_existing_events(state: &mut AppState) {
    let Some(events_file) = get_events_file() else {
        eprintln!("[eocc] Cannot load events: home directory not found");
        return;
    };

    if events_file.exists() {
        if let Ok(content) = fs::read_to_string(&events_file) {
            for line in content.lines() {
                if !line.is_empty() {
                    if let Ok(event) = serde_json::from_str::<EventInfo>(line) {
                        process_event(state, event);
                    }
                }
            }
            if let Ok(metadata) = fs::metadata(&events_file) {
                state.last_file_pos = metadata.len();
            }
        }
    }
}

fn main() {
    let state = Arc::new(Mutex::new(AppState::default()));
    let difit_registry = Arc::new(DifitProcessRegistry::new());

    // Load settings and existing events
    {
        let Ok(mut state_guard) = state.lock() else {
            eprintln!("[eocc] Failed to acquire state lock during initialization");
            return;
        };
        state_guard.settings = load_settings();
        load_existing_events(&mut state_guard);
    }

    let state_clone = Arc::clone(&state);
    let state_for_managed = Arc::clone(&state);
    let difit_registry_clone = Arc::clone(&difit_registry);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ManagedState(state_for_managed))
        .manage(difit_registry_clone)
        .invoke_handler(tauri::generate_handler![
            get_dashboard_data,
            remove_session,
            clear_all_sessions,
            get_always_on_top,
            set_always_on_top,
            get_mini_view,
            set_mini_view,
            get_settings,
            set_opacity_active,
            set_opacity_inactive,
            get_repo_git_info,
            open_diff
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_for_tray = Arc::clone(&state_clone);

            // Get initial settings
            let (always_on_top, mini_view) = {
                let state_guard = state_for_tray
                    .lock()
                    .map_err(|_| tauri::Error::Anyhow(anyhow::anyhow!("Failed to acquire state lock")))?;
                (
                    state_guard.settings.always_on_top,
                    state_guard.settings.mini_view,
                )
            };

            // Create dashboard window
            let dashboard_window = create_dashboard_window(app, always_on_top, mini_view)?;

            // Set initial badge count
            if let Ok(state_guard) = state_for_tray.lock() {
                let waiting_count = state_guard.waiting_session_count();
                if waiting_count > 0 {
                    let _ = dashboard_window.set_badge_count(Some(waiting_count as i64));
                }
            }

            // Hide dashboard and close all diff windows when close button is clicked
            let app_handle_for_close = app_handle.clone();
            dashboard_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();

                    // Close all diff windows
                    for (label, window) in app_handle_for_close.webview_windows() {
                        if label.starts_with("difit-") {
                            let _ = window.close();
                        }
                    }

                    // Hide dashboard
                    if let Some(window) = app_handle_for_close.get_webview_window("dashboard") {
                        let _ = window.hide();
                    }
                }
            });

            // Build app menu bar
            let state_for_app_menu = Arc::clone(&state_clone);
            let app_menu = {
                let state_guard = state_for_tray
                    .lock()
                    .map_err(|_| tauri::Error::Anyhow(anyhow::anyhow!("Failed to acquire state lock")))?;
                build_app_menu(&app_handle, &state_guard)?
            };

            // Set app menu and handle events
            app.set_menu(app_menu)?;
            app.on_menu_event(move |app, event| {
                let state = &state_for_app_menu;
                match event.id.as_ref() {
                    "open_dashboard" => {
                        show_dashboard(app);
                    }
                    "always_on_top" => {
                        match state.lock() {
                            Ok(mut state_guard) => {
                                toggle_always_on_top(app, &mut state_guard);
                                update_tray_and_badge(app, &state_guard);
                            }
                            Err(e) => eprintln!("[eocc] Failed to acquire lock for always_on_top: {:?}", e),
                        }
                    }
                    "mini_view" => {
                        match state.lock() {
                            Ok(mut state_guard) => {
                                toggle_mini_view(app, &mut state_guard);
                                update_tray_and_badge(app, &state_guard);
                            }
                            Err(e) => eprintln!("[eocc] Failed to acquire lock for mini_view: {:?}", e),
                        }
                    }
                    "sound_enabled" => {
                        match state.lock() {
                            Ok(mut state_guard) => {
                                state_guard.settings.sound_enabled = !state_guard.settings.sound_enabled;
                                save_settings(&state_guard.settings);
                                let _ = app.emit("settings-updated", &state_guard.settings);
                                update_tray_and_badge(app, &state_guard);
                            }
                            Err(e) => eprintln!("[eocc] Failed to acquire lock for sound_enabled: {:?}", e),
                        }
                    }
                    other => {
                        if let Some((is_active, opacity)) = parse_opacity_menu_id(other) {
                            match state.lock() {
                                Ok(mut state_guard) => {
                                    if is_active {
                                        state_guard.settings.opacity_active = opacity;
                                    } else {
                                        state_guard.settings.opacity_inactive = opacity;
                                    }
                                    save_settings(&state_guard.settings);
                                    let _ = app.emit("settings-updated", &state_guard.settings);
                                    update_tray_and_badge(app, &state_guard);
                                }
                                Err(e) => eprintln!("[eocc] Failed to acquire lock for opacity: {:?}", e),
                            }
                        }
                    }
                }
            });

            // Build tray menu
            let tray_menu = {
                let state_guard = state_for_tray
                    .lock()
                    .map_err(|_| tauri::Error::Anyhow(anyhow::anyhow!("Failed to acquire state lock")))?;
                build_tray_menu(&app_handle, &state_guard)?
            };

            let initial_icon = Image::from_bytes(ICON_NORMAL)?;

            // Create tray icon
            let _tray = TrayIconBuilder::with_id("main")
                .icon(initial_icon)
                .menu(&tray_menu)
                .show_menu_on_left_click(true)
                .tooltip("Eyes on Claude Code")
                .on_menu_event(move |app, event| match event.id.as_ref() {
                    "open_dashboard" => {
                        show_dashboard(app);
                    }
                    "open_logs" => {
                        if let Some(log_dir) = get_log_dir() {
                            let _ = opener::open(&log_dir);
                        } else {
                            eprintln!("[eocc] Cannot open logs: home directory not found");
                        }
                    }
                    "clear_sessions" => {
                        match state_for_tray.lock() {
                            Ok(mut state_guard) => {
                                state_guard.sessions.clear();
                                update_tray_and_badge(app, &state_guard);
                                emit_state_update(app, &state_guard);
                            }
                            Err(e) => eprintln!("[eocc] Failed to acquire lock for clear_sessions: {:?}", e),
                        }
                    }
                    _ => {}
                })
                .on_tray_icon_event(|_tray, event| {
                    if let TrayIconEvent::Click {
                        button: MouseButton::Left,
                        button_state: MouseButtonState::Up,
                        ..
                    } = event
                    {
                        // Menu will show automatically
                    }
                })
                .build(app)?;

            // Start file watcher
            start_file_watcher(app.handle().clone(), Arc::clone(&state_clone));

            Ok(())
        })
        .on_window_event(move |_window, _event| {
            // Note: This is called for each window event
            // We handle difit window cleanup in commands.rs on_window_event
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            if let tauri::RunEvent::Exit = event {
                // Kill all difit processes on app exit
                difit_registry.kill_all();
            }
        });
}
