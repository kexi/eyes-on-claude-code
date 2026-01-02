use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};

use crate::settings::get_events_file;
use crate::state::{AppState, EventInfo, EventType, NotificationType, SessionInfo, SessionStatus};

pub fn process_event(state: &mut AppState, event: EventInfo) {
    state.recent_events.push_back(event.clone());
    if state.recent_events.len() > 50 {
        state.recent_events.pop_front();
    }

    let key = if event.project_dir.is_empty() {
        event.project_name.clone()
    } else {
        event.project_dir.clone()
    };

    match event.event_type {
        EventType::SessionStart => {
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
        EventType::SessionEnd => {
            state.sessions.remove(&key);
        }
        EventType::Notification => {
            let new_status = match event.notification_type {
                NotificationType::PermissionPrompt => SessionStatus::WaitingPermission,
                NotificationType::IdlePrompt => SessionStatus::WaitingInput,
                NotificationType::Other => SessionStatus::Active,
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
        EventType::Stop => {
            state.upsert_session(key, &event, SessionStatus::Completed, String::new());
        }
        EventType::PostToolUse => {
            state.upsert_session(key, &event, SessionStatus::Active, String::new());
        }
        EventType::Unknown => {
            if let Some(session) = state.sessions.get_mut(&key) {
                session.last_event = event.timestamp;
            }
        }
    }
}

pub fn read_new_events(state: &mut AppState) -> Vec<EventInfo> {
    let mut new_events = Vec::new();

    let Some(events_file) = get_events_file() else {
        return new_events;
    };

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
