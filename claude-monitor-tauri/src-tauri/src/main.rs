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
    menu::{CheckMenuItem, CheckMenuItemBuilder, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    webview::Color,
    Emitter, Manager, Runtime, WebviewUrl, WebviewWindowBuilder,
};

const ICON_NORMAL: &[u8] = include_bytes!("../icons/icon.png");

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
    waiting_for: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Settings {
    #[serde(default = "default_always_on_top")]
    always_on_top: bool,
    #[serde(default = "default_mini_view")]
    mini_view: bool,
    #[serde(default = "default_opacity_active")]
    opacity_active: f64,
    #[serde(default = "default_opacity_inactive")]
    opacity_inactive: f64,
    #[serde(default = "default_sound_enabled")]
    sound_enabled: bool,
}

fn default_always_on_top() -> bool {
    true
}

fn default_mini_view() -> bool {
    true
}

fn default_opacity_active() -> f64 {
    1.0 // 100%
}

fn default_opacity_inactive() -> f64 {
    0.3 // 30%
}

fn default_sound_enabled() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            mini_view: true,
            opacity_active: 1.0,
            opacity_inactive: 0.3,
            sound_enabled: true,
        }
    }
}

const MINI_VIEW_WIDTH: f64 = 228.0;
const MINI_VIEW_HEIGHT: f64 = 416.0;
const NORMAL_VIEW_WIDTH: f64 = 900.0;
const NORMAL_VIEW_HEIGHT: f64 = 700.0;

struct AppState {
    sessions: HashMap<String, SessionInfo>,
    recent_events: Vec<EventInfo>,
    last_file_pos: u64,
    settings: Settings,
}

impl AppState {
    fn waiting_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| {
                s.status == SessionStatus::WaitingPermission || s.status == SessionStatus::WaitingInput
            })
            .count()
    }

    fn to_dashboard_data(&self) -> DashboardData {
        DashboardData {
            sessions: self.sessions.values().cloned().collect(),
            events: self.recent_events.clone(),
        }
    }

    /// Insert or update a session with the given status and waiting_for info
    fn upsert_session(
        &mut self,
        key: String,
        event: &EventInfo,
        status: SessionStatus,
        waiting_for: String,
    ) {
        self.sessions
            .entry(key)
            .and_modify(|s| {
                s.status = status.clone();
                s.last_event = event.timestamp.clone();
                s.waiting_for = waiting_for.clone();
            })
            .or_insert_with(|| SessionInfo {
                project_name: event.project_name.clone(),
                project_dir: event.project_dir.clone(),
                status,
                last_event: event.timestamp.clone(),
                waiting_for,
            });
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            sessions: HashMap::new(),
            recent_events: Vec::new(),
            last_file_pos: 0,
            settings: Settings::default(),
        }
    }
}

struct ManagedState(Arc<Mutex<AppState>>);

fn get_config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".claude-monitor")
}

fn get_log_dir() -> PathBuf {
    get_config_dir().join("logs")
}

fn get_events_file() -> PathBuf {
    get_log_dir().join("events.jsonl")
}

fn get_settings_file() -> PathBuf {
    get_config_dir().join("settings.json")
}

fn load_settings() -> Settings {
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

fn save_settings(settings: &Settings) {
    let settings_file = get_settings_file();
    if let Ok(content) = serde_json::to_string_pretty(settings) {
        let _ = fs::create_dir_all(get_config_dir());
        let _ = fs::write(settings_file, content);
    }
}

/// Parse opacity menu ID and return (is_active, opacity_value) if valid
/// Menu ID format: "opacity_{inactive|active}_{10|30|50|70|80|90|100}"
fn parse_opacity_menu_id(menu_id: &str) -> Option<(bool, f64)> {
    let suffix = menu_id.strip_prefix("opacity_")?;
    let (target, value_str) = suffix.rsplit_once('_')?;
    let value: i32 = value_str.parse().ok()?;
    let opacity = value as f64 / 100.0;

    match target {
        "active" => Some((true, opacity)),
        "inactive" => Some((false, opacity)),
        _ => None,
    }
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
            let waiting_info = if !event.message.is_empty() {
                event.message.clone()
            } else if !event.tool_name.is_empty() {
                event.tool_name.clone()
            } else {
                String::new()
            };
            state.upsert_session(key, &event, new_status, waiting_info);
        }
        "stop" => {
            state.upsert_session(key, &event, SessionStatus::Completed, String::new());
        }
        "post_tool_use" => {
            state.upsert_session(key, &event, SessionStatus::Active, String::new());
        }
        _ => {
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

fn build_menu<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) -> tauri::Result<(Menu<R>, CheckMenuItem<R>, CheckMenuItem<R>)> {
    let waiting_count = state
        .sessions
        .values()
        .filter(|s| {
            s.status == SessionStatus::WaitingPermission || s.status == SessionStatus::WaitingInput
        })
        .count();

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

    let open_dashboard = MenuItemBuilder::with_id("open_dashboard", "Open Dashboard")
        .build(app)?;

    let always_on_top = CheckMenuItemBuilder::with_id("always_on_top", "Always on Top")
        .checked(state.settings.always_on_top)
        .build(app)?;

    let mini_view = CheckMenuItemBuilder::with_id("mini_view", "Mini View")
        .checked(state.settings.mini_view)
        .build(app)?;

    let sound_enabled = CheckMenuItemBuilder::with_id("sound_enabled", "Sound")
        .checked(state.settings.sound_enabled)
        .build(app)?;

    // Opacity submenu
    let opacity_inactive_label = format!("Inactive: {}%", (state.settings.opacity_inactive * 100.0) as i32);
    let opacity_active_label = format!("Active: {}%", (state.settings.opacity_active * 100.0) as i32);

    let opacity_submenu = SubmenuBuilder::new(app, "Opacity")
        .item(&MenuItemBuilder::with_id("opacity_inactive_header", &opacity_inactive_label).enabled(false).build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_10", "  10%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_30", "  30%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_50", "  50%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_70", "  70%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_100", "  100%").build(app)?)
        .separator()
        .item(&MenuItemBuilder::with_id("opacity_active_header", &opacity_active_label).enabled(false).build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_active_70", "  70%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_active_80", "  80%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_active_90", "  90%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_active_100", "  100%").build(app)?)
        .build()?;

    let sep_dashboard = PredefinedMenuItem::separator(app)?;

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

    let open_logs = MenuItemBuilder::with_id("open_logs", "Open Log Folder")
        .build(app)?;
    let clear_sessions = MenuItemBuilder::with_id("clear_sessions", "Clear Sessions")
        .build(app)?;

    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let menu = Menu::with_items(app, &[&header, &sep1, &open_dashboard, &always_on_top, &mini_view, &sound_enabled, &opacity_submenu, &sep_dashboard])?;

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

    Ok((menu, always_on_top, mini_view))
}

fn update_tray_and_badge<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) {
    // Update tray menu
    if let Some(tray) = app.tray_by_id("main") {
        if let Ok((new_menu, _, _)) = build_menu(app, state) {
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

fn emit_state_update<R: Runtime>(app: &tauri::AppHandle<R>, state: &AppState) {
    let data = state.to_dashboard_data();
    let _ = app.emit("state-updated", &data);
}

fn show_dashboard<R: Runtime>(app: &tauri::AppHandle<R>) {
    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.show();
        let _ = window.set_focus();
    }
}

fn toggle_always_on_top<R: Runtime>(app: &tauri::AppHandle<R>, state: &mut AppState) {
    state.settings.always_on_top = !state.settings.always_on_top;
    save_settings(&state.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_always_on_top(state.settings.always_on_top);
    }
}

fn toggle_mini_view<R: Runtime>(app: &tauri::AppHandle<R>, state: &mut AppState) {
    state.settings.mini_view = !state.settings.mini_view;
    save_settings(&state.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_decorations(!state.settings.mini_view);
        if state.settings.mini_view {
            let _ = window.set_size(tauri::LogicalSize::new(MINI_VIEW_WIDTH, MINI_VIEW_HEIGHT));
        } else {
            let _ = window.set_size(tauri::LogicalSize::new(NORMAL_VIEW_WIDTH, NORMAL_VIEW_HEIGHT));
            let _ = window.center();
        }
    }

    // Emit settings update to frontend
    let _ = app.emit("settings-updated", &state.settings);
}

#[tauri::command]
fn get_dashboard_data(state: tauri::State<'_, ManagedState>) -> DashboardData {
    let state_guard = state.0.lock().unwrap();
    state_guard.to_dashboard_data()
}

#[tauri::command]
fn remove_session(project_dir: String, state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.sessions.remove(&project_dir);
    update_tray_and_badge(&app, &state_guard);
    emit_state_update(&app, &state_guard);
}

#[tauri::command]
fn clear_all_sessions(state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.sessions.clear();
    update_tray_and_badge(&app, &state_guard);
    emit_state_update(&app, &state_guard);
}

#[tauri::command]
fn get_always_on_top(state: tauri::State<'_, ManagedState>) -> bool {
    let state_guard = state.0.lock().unwrap();
    state_guard.settings.always_on_top
}

#[tauri::command]
fn set_always_on_top(enabled: bool, state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.settings.always_on_top = enabled;
    save_settings(&state_guard.settings);

    if let Some(window) = app.get_webview_window("dashboard") {
        let _ = window.set_always_on_top(enabled);
    }

    update_tray_and_badge(&app, &state_guard);
}

#[tauri::command]
fn get_mini_view(state: tauri::State<'_, ManagedState>) -> bool {
    let state_guard = state.0.lock().unwrap();
    state_guard.settings.mini_view
}

#[tauri::command]
fn set_mini_view(enabled: bool, state: tauri::State<'_, ManagedState>, app: tauri::AppHandle) {
    let mut state_guard = state.0.lock().unwrap();
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
}

#[tauri::command]
fn get_settings(state: tauri::State<'_, ManagedState>) -> Settings {
    let state_guard = state.0.lock().unwrap();
    state_guard.settings.clone()
}

#[tauri::command]
fn set_opacity_active(opacity: f64, state: tauri::State<'_, ManagedState>) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.settings.opacity_active = opacity.clamp(0.1, 1.0);
    save_settings(&state_guard.settings);
}

#[tauri::command]
fn set_opacity_inactive(opacity: f64, state: tauri::State<'_, ManagedState>) {
    let mut state_guard = state.0.lock().unwrap();
    state_guard.settings.opacity_inactive = opacity.clamp(0.1, 1.0);
    save_settings(&state_guard.settings);
}

fn main() {
    let state = Arc::new(Mutex::new(AppState::default()));

    // Load settings
    {
        let mut state_guard = state.lock().unwrap();
        state_guard.settings = load_settings();
    }

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
            set_opacity_inactive
        ])
        .setup(move |app| {
            let app_handle = app.handle().clone();
            let state_for_tray = Arc::clone(&state_clone);

            // Get initial settings
            let (always_on_top, mini_view) = {
                let state_guard = state_for_tray.lock().unwrap();
                (state_guard.settings.always_on_top, state_guard.settings.mini_view)
            };

            // Determine initial window size based on mini_view setting
            let (width, height) = if mini_view {
                (MINI_VIEW_WIDTH, MINI_VIEW_HEIGHT)
            } else {
                (NORMAL_VIEW_WIDTH, NORMAL_VIEW_HEIGHT)
            };

            // Create dashboard window with settings applied
            // Use transparent background color (RGBA with alpha = 0)
            let transparent_color = Color(0, 0, 0, 0);

            let base_builder = WebviewWindowBuilder::new(
                app,
                "dashboard",
                WebviewUrl::App("index.html".into()),
            )
            .title("Claude Monitor - Dashboard")
            .inner_size(width, height)
            .min_inner_size(200.0, 300.0)
            .center()
            .visible(true)
            .always_on_top(always_on_top)
            .decorations(!mini_view)
            .transparent(true)
            .background_color(transparent_color);

            let dashboard_window = match Image::from_bytes(ICON_NORMAL) {
                Ok(icon) => base_builder.icon(icon)?.build()?,
                Err(_) => base_builder.build()?,
            };

            // Set initial badge count
            {
                let state_guard = state_for_tray.lock().unwrap();
                let waiting_count = state_guard.waiting_session_count();
                if waiting_count > 0 {
                    let _ = dashboard_window.set_badge_count(Some(waiting_count as i64));
                }
            }

            // Hide window when close button is clicked instead of destroying it
            let app_handle_for_close = app_handle.clone();
            dashboard_window.on_window_event(move |event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    api.prevent_close();
                    if let Some(window) = app_handle_for_close.get_webview_window("dashboard") {
                        let _ = window.hide();
                    }
                }
            });

            // Build initial menu
            let (menu, _, _) = {
                let state_guard = state_for_tray.lock().unwrap();
                build_menu(&app_handle, &state_guard)?
            };

            let initial_icon = Image::from_bytes(ICON_NORMAL)?;

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
                            show_dashboard(app);
                        }
                        "always_on_top" => {
                            let mut state_guard = state_for_tray.lock().unwrap();
                            toggle_always_on_top(app, &mut state_guard);
                            update_tray_and_badge(app, &state_guard);
                        }
                        "mini_view" => {
                            let mut state_guard = state_for_tray.lock().unwrap();
                            toggle_mini_view(app, &mut state_guard);
                            update_tray_and_badge(app, &state_guard);
                        }
                        "sound_enabled" => {
                            let mut state_guard = state_for_tray.lock().unwrap();
                            state_guard.settings.sound_enabled = !state_guard.settings.sound_enabled;
                            save_settings(&state_guard.settings);
                            let _ = app.emit("settings-updated", &state_guard.settings);
                            update_tray_and_badge(app, &state_guard);
                        }
                        "open_logs" => {
                            let log_dir = get_log_dir();
                            let _ = opener::open(&log_dir);
                        }
                        "clear_sessions" => {
                            let mut state_guard = state_for_tray.lock().unwrap();
                            state_guard.sessions.clear();
                            update_tray_and_badge(app, &state_guard);
                            emit_state_update(app, &state_guard);
                        }
                        other => {
                            // Handle opacity menu items dynamically (opacity_active_*, opacity_inactive_*)
                            if let Some((is_active, opacity)) = parse_opacity_menu_id(other) {
                                let mut state_guard = state_for_tray.lock().unwrap();
                                if is_active {
                                    state_guard.settings.opacity_active = opacity;
                                } else {
                                    state_guard.settings.opacity_inactive = opacity;
                                }
                                save_settings(&state_guard.settings);
                                let _ = app.emit("settings-updated", &state_guard.settings);
                                update_tray_and_badge(app, &state_guard);
                            }
                        }
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
                                update_tray_and_badge(&app_handle_for_watcher, &state_guard);
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
