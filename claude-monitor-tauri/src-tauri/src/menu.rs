use tauri::{
    menu::{CheckMenuItem, CheckMenuItemBuilder, Menu, MenuItemBuilder, PredefinedMenuItem, SubmenuBuilder},
    Runtime,
};

use crate::state::{AppState, SessionStatus};

/// Parse opacity menu ID and return (is_active, opacity_value) if valid
/// Menu ID format: "opacity_{inactive|active}_{10|30|50|70|80|90|100}"
pub fn parse_opacity_menu_id(menu_id: &str) -> Option<(bool, f64)> {
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

pub fn build_menu<R: Runtime>(
    app: &tauri::AppHandle<R>,
    state: &AppState,
) -> tauri::Result<(Menu<R>, CheckMenuItem<R>, CheckMenuItem<R>)> {
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

    let open_dashboard = MenuItemBuilder::with_id("open_dashboard", "Open Dashboard").build(app)?;

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
    let opacity_inactive_label = format!(
        "Inactive: {}%",
        (state.settings.opacity_inactive * 100.0) as i32
    );
    let opacity_active_label =
        format!("Active: {}%", (state.settings.opacity_active * 100.0) as i32);

    let opacity_submenu = SubmenuBuilder::new(app, "Opacity")
        .item(
            &MenuItemBuilder::with_id("opacity_inactive_header", &opacity_inactive_label)
                .enabled(false)
                .build(app)?,
        )
        .item(&MenuItemBuilder::with_id("opacity_inactive_10", "  10%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_30", "  30%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_50", "  50%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_70", "  70%").build(app)?)
        .item(&MenuItemBuilder::with_id("opacity_inactive_100", "  100%").build(app)?)
        .separator()
        .item(
            &MenuItemBuilder::with_id("opacity_active_header", &opacity_active_label)
                .enabled(false)
                .build(app)?,
        )
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
            let item =
                MenuItemBuilder::with_id(format!("session_{}", session.project_name), &title)
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

    let open_logs = MenuItemBuilder::with_id("open_logs", "Open Log Folder").build(app)?;
    let clear_sessions = MenuItemBuilder::with_id("clear_sessions", "Clear Sessions").build(app)?;

    let sep2 = PredefinedMenuItem::separator(app)?;
    let quit = MenuItemBuilder::with_id("quit", "Quit")
        .accelerator("CmdOrCtrl+Q")
        .build(app)?;

    let menu = Menu::with_items(
        app,
        &[
            &header,
            &sep1,
            &open_dashboard,
            &always_on_top,
            &mini_view,
            &sound_enabled,
            &opacity_submenu,
            &sep_dashboard,
        ],
    )?;

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
