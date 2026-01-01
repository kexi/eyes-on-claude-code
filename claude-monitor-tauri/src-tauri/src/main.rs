#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use notify::{Config, RecommendedWatcher, RecursiveMode, Watcher};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tauri::{
    image::Image,
    menu::{MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder, Menu},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

const ICON_NORMAL: &[u8] = include_bytes!("../icons/icon.png");
const ICON_WAITING: &[u8] = include_bytes!("../icons/icon-waiting.png");

#[derive(Debug, Clone, Serialize, Deserialize)]
struct EventInfo {
    timestamp: String,
    event: String,
    matcher: String,
    project_name: String,
    project_dir: String,
    session_id: String,
    message: String,
    notification_type: String,
    #[serde(default)]
    tool_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionInfo {
    project_name: String,
    project_dir: String,
    status: SessionStatus,
    last_event: String,
    #[serde(default)]
    waiting_for: String,  // Tool name or message when waiting
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum SessionStatus {
    Active,
    WaitingPermission,
    WaitingInput,
    Completed,
}

impl SessionStatus {
    fn emoji(&self) -> &str {
        match self {
            SessionStatus::Active => "🟢",
            SessionStatus::WaitingPermission => "🔐",
            SessionStatus::WaitingInput => "⏳",
            SessionStatus::Completed => "✅",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DashboardData {
    sessions: Vec<SessionInfo>,
    events: Vec<EventInfo>,
}

struct AppState {
    sessions: HashMap<String, SessionInfo>,
    recent_events: Vec<EventInfo>,
    last_file_pos: u64,
}

impl AppState {
    fn has_waiting_sessions(&self) -> bool {
        self.sessions.values().any(|s| {
            s.status == SessionStatus::WaitingPermission || s.status == SessionStatus::WaitingInput
        })
    }

    fn to_dashboard_data(&self) -> DashboardData {
        DashboardData {
            sessions: self.sessions.values().cloned().collect(),
            events: self.recent_events.clone(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            recent_events: Vec::new(),
            last_file_pos: 0,
        }
    }
}

// Wrapper for thread-safe state management
struct ManagedState(Arc<Mutex<AppState>>);

fn get_log_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude-monitor")
        .join("logs")
}

fn get_events_file() -> PathBuf {
    get_log_dir().join("events.jsonl")
}

fn process_event(state: &mut AppState, event: EventInfo) {
    state.recent_events.push(event.clone());
    if state.recent_events.len() > 50 {
        state.recent_events.remove(0);
    }

    let key = if event.project_dir.is_empty() {
        event.project_name.clone()
    } else {
        event.project_dir.clone()
    };

    match event.event.as_str() {
        "session_start" => {
            state.sessions.insert(
                key,
                SessionInfo {
                    project_name: event.project_name,
                    project_dir: event.project_dir,
                    status: SessionStatus::Active,
                    last_event: event.timestamp,
                    waiting_for: String::new(),
                },
            );
        }
        "session_end" => {
            state.sessions.remove(&key);
        }
        "notification" => {
            let new_status = match event.notification_type.as_str() {
                "permission_prompt" => SessionStatus::WaitingPermission,
                "idle_prompt" => SessionStatus::WaitingInput,
                _ => SessionStatus::Active,
            };
            // Use message if available, otherwise use tool_name
            let waiting_info = if !event.message.is_empty() {
                event.message.clone()
            } else if !event.tool_name.is_empty() {
                event.tool_name.clone()
            } else {
                String::new()
            };
            state.sessions
                .entry(key)
                .and_modify(|s| {
                    s.status = new_status.clone();
                    s.last_event = event.timestamp.clone();
                    s.waiting_for = waiting_info.clone();
                })
                .or_insert_with(|| SessionInfo {
                    project_name: event.project_name,
                    project_dir: event.project_dir,
                    status: new_status,
                    last_event: event.timestamp,
                    waiting_for: waiting_info,
                });
        }
        "stop" => {
            state.sessions
                .entry(key)
                .and_modify(|s| {
                    s.status = SessionStatus::Completed;
                    s.last_event = event.timestamp.clone();
                    s.waiting_for = String::new();
                })
                .or_insert_with(|| SessionInfo {
                    project_name: event.project_name,
                    project_dir: event.project_dir,
                    status: SessionStatus::Completed,
                    last_event: event.timestamp,
                    waiting_for: String::new(),
                });
        }
        "post_tool_use" => {
            // After a tool is used, the session is active again
            state.sessions
                .entry(key)
                .and_modify(|s| {
                    s.status = SessionStatus::Active;
                    s.last_event = event.timestamp.clone();
                    s.waiting_for = String::new();
                })
                .or_insert_with(|| SessionInfo {
                    project_name: event.project_name,
                    project_dir: event.project_dir,
                    status: SessionStatus::Active,
                    last_event: event.timestamp,
                    waiting_for: String::new(),
                });
        }
        _ => {
            // For unknown events, only update if session exists (don't auto-create)
            if let Some(session) = state.sessions.get_mut(&key) {
                session.last_event = event.timestamp;
            }
        }
    }
}

fn read_new_events(state: &mut AppState) -> Vec<EventInfo> {
    let events_file = get_events_file();
    let mut new_events = Vec::new();

    if !events_file.exists() {
        return new_events;
    }

    if let Ok(mut file) = File::open(&events_file) {
        if let Ok(metadata) = file.metadata() {
            let file_size = metadata.len();

            if file_size < state.last_file_pos {
                state.last_file_pos = 0;
            }

            if file_size > state.last_file_pos {
                let _ = file.seek(SeekFrom::Start(state.last_file_pos));
                let reader = BufReader::new(&file);

                for line in reader.lines().map_while(Result::ok) {
                    if !line.is_empty() {
                        if let Ok(event) = serde_json::from_str::<EventInfo>(&line) {
                            process_event(state, event.clone());
                            new_events.push(event);
                        }
                    }
                }

                state.last_file_pos = file_size;
            }
        }
    }

    new_events
}

fn build_menu<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) -> tauri::Result<Menu<R>> {
    let waiting_count = state
        .sessions
        .values()
        .filter(|s| {
            s.status == SessionStatus::WaitingPermission || s.status == SessionStatus::WaitingInput
        })
        .count();

    // Header
    let header_text = if waiting_count > 0 {
        format!("⚠️ {} session(s) waiting", waiting_count)
    } else if state.sessions.is_empty() {
        "No active sessions".to_string()
    } else {
        format!("{} active session(s)", state.sessions.len())
    };

    let header = MenuItemBuilder::with_id("header", &header_text)
        .enabled(false)
        .build(app)?;

    let sep1 = PredefinedMenuItem::separator(app)?;

    // Open Dashboard
    let open_dashboard = MenuItemBuilder::with_id("open_dashboard", "Open Dashboard")
        .accelerator("CmdOrCtrl+D")
        .build(app)?;

    let sep_dashboard = PredefinedMenuItem::separator(app)?;

    // Sessions section
    let mut session_items = Vec::new();
    if !state.sessions.is_empty() {
        let sessions_header = MenuItemBuilder::with_id("sessions_header", "Sessions")
            .enabled(false)
            .build(app)?;
        session_items.push(sessions_header);

        for (_, session) in &state.sessions {
            let emoji = session.status.emoji();
            let title = format!("{} {}", emoji, session.project_name);
            let item = MenuItemBuilder::with_id(format!("session_{}", session.project_name), &title)
                .enabled(false)
                .build(app)?;
            session_items.push(item);
        }
    }

    // Recent events submenu
    let events_submenu = if !state.recent_events.is_empty() {
        let mut submenu_builder = SubmenuBuilder::new(app, "Recent Events");
        for (idx, event) in state.recent_events.iter().rev().take(10).enumerate() {
            let emoji = match event.event.as_str() {
                "notification" => match event.notification_type.as_str() {
                    "permission_prompt" => "🔐",
                    "idle_prompt" => "⏳",
                    _ => "🔔",
                },
                "stop" => "✅",
                "session_start" => "🚀",
                "session_end" => "🏁",
                _ => "📌",
            };
            let title = format!("{} {}: {}", emoji, event.project_name, event.event);
            let item = MenuItemBuilder::with_id(format!("event_{}", idx), &title)
                .enabled(false)
                .build(app)?;
            submenu_builder = submenu_builder.item(&item);
        }
        Some(submenu_builder.build()?)
    } else {
        None
    };

    // Actions
    let open_logs = MenuItemBuilder::with_id("open_logs", "Open Log Folder")
        .build(app)?;
    let clear_sessions = MenuItemBuilder::with_id("clear_sessions", "Clear Sessions")
        .build(app)?;

    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    // Build menu
    let menu = Menu::with_items(app, &[&header, &sep1, &open_dashboard, &sep_dashboard])?;

    for item in &session_items {
        menu.append(item)?;
    }

    if !session_items.is_empty() {
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    if let Some(submenu) = &events_submenu {
        menu.append(submenu)?;
        menu.append(&PredefinedMenuItem::separator(app)?)?;
    }

    menu.append(&open_logs)?;
    menu.append(&clear_sessions)?;
    menu.append(&sep2)?;
    menu.append(&quit)?;

    Ok(menu)
}

fn update_tray<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) {
    if let Some(tray) = app.tray_by_id("main") {
        // Update menu
        if let Ok(new_menu) = build_menu(app, state) {
            let _ = tray.set_menu(Some(new_menu));
        }

        // Update icon based on state
        let icon_bytes = if state.has_waiting_sessions() {
            ICON_WAITING
        } else {
            ICON_NORMAL
        };

        if let Ok(icon) = Image::from_bytes(icon_bytes) {
            let _ = tray.set_icon(Some(icon));
        }

        // Update tooltip
        let tooltip = if state.has_waiting_sessions() {
            "Claude Monitor - Action Required!"
        } else if state.sessions.is_empty() {
            "Claude Monitor - No active sessions"
        } else {
            "Claude Monitor"
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

fn emit_state_update<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) {
    let data = state.to_dashboard_data();
    let _ = app.emit("state-updated", &data);
}

fn open_dashboard<R: Runtime>(app: &tauri::AppHandle<R>) {
    // Check if window already exists
    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.show();
        let _ = window.set_focus();
        return;
    }

    // Create new window
    let _ = WebviewWindowBuilder::new(
        app,
        "dashboard",
        WebviewUrl::App("index.html".into()),
    )
    .title("Claude Monitor - Dashboard")
    .inner_size(900.0, 700.0)
    .min_inner_size(600.0, 400.0)
    .center()
    .build();
}

// Tauri command to get dashboard data
#[tauri::command]
fn get_dashboard_data(state: tauri::State<'_, ManagedState>) -> DashboardData {
    let state_guard = state.0.lock().unwrap();
    state_guard.to_dashboard_data()
}

// Tauri command to remove a single session
#[tauri::command]
fn remove_session(project_dir: String, state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.sessions.remove(&project_dir);
    update_tray(&app, &state_guard);
    emit_state_update(&app, &state_guard);
}

// Tauri command to clear all sessions
#[tauri::command]
fn clear_all_sessions(state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.sessions.clear();
    update_tray(&app, &state_guard);
    emit_state_update(&app, &state_guard);
}

fn main() {
    let state = Arc::new(Mutex::new(AppState::default()));

    // Load existing events
    {
        let mut state_guard = state.lock().unwrap();
        let events_file = get_events_file();
        if events_file.exists() {
            if let Ok(content) = fs::read_to_string(&events_file) {
                for line in content.lines() {
                    if !line.is_empty() {
                        if let Ok(event) = serde_json::from_str::<EventInfo>(line) {
                            process_event(&mut state_guard, event);
                        }
                    }
                }
                if let Ok(metadata) = fs::metadata(&events_file) {
                    state_guard.last_file_pos = metadata.len();
                }
            }
        }
    }

    let state_clone = Arc::clone(&state);
    let state_for_managed = Arc::clone(&state);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(ManagedState(state_for_managed))
        .invoke_handler(tauri::generate_handler![get_dashboard_data, remove_session, clear_all_sessions])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_for_tray = Arc::clone(&state_clone);

            // Build initial menu
            let menu = {
                let state_guard = state_for_tray.lock().unwrap();
                build_menu(&app_handle, &state_guard)?
            };

            // Determine initial icon
            let initial_icon = {
                let state_guard = state_for_tray.lock().unwrap();
                if state_guard.has_waiting_sessions() {
                    Image::from_bytes(ICON_WAITING)?
                } else {
                    Image::from_bytes(ICON_NORMAL)?
                }
            };

            // Create tray icon
            let _tray = TrayIconBuilder::with_id("main")
                .icon(initial_icon)
                .menu(&menu)
                .show_menu_on_left_click(true)
                .tooltip("Claude Monitor")
                .on_menu_event(move |app, event| {
                    match event.id.as_ref() {
                        "quit" => {
                            app.exit(0);
                        }
                        "open_dashboard" => {
                            open_dashboard(app);
                        }
                        "open_logs" => {
                            let log_dir = get_log_dir();
                            let _ = opener::open(&log_dir);
                        }
                        "clear_sessions" => {
                            let mut state_guard = state_for_tray.lock().unwrap();
                            state_guard.sessions.clear();
                            update_tray(app, &state_guard);
                            emit_state_update(app, &state_guard);
                        }
                        _ => {}
                    }
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
            let state_for_watcher = Arc::clone(&state_clone);
            let app_handle_for_watcher = app.handle().clone();

            std::thread::spawn(move || {
                let log_dir = get_log_dir();
                let _ = fs::create_dir_all(&log_dir);

                let (tx, rx) = std::sync::mpsc::channel();

                let mut watcher =
                    RecommendedWatcher::new(tx, Config::default()).expect("Failed to create watcher");

                watcher
                    .watch(&log_dir, RecursiveMode::NonRecursive)
                    .expect("Failed to watch directory");

                loop {
                    match rx.recv() {
                        Ok(_event) => {
                            let mut state_guard = state_for_watcher.lock().unwrap();
                            let new_events = read_new_events(&mut state_guard);

                            if !new_events.is_empty() {
                                update_tray(&app_handle_for_watcher, &state_guard);
                                emit_state_update(&app_handle_for_watcher, &state_guard);
                            }
                        }
                        Err(e) => {
                            eprintln!("Watch error: {:?}", e);
                            break;
                        }
                    }
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
