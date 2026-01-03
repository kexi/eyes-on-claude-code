use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EventType {
    SessionStart,
    SessionEnd,
    Notification,
    Stop,
    PostToolUse,
    UserPromptSubmit,
    #[serde(other)]
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationType {
    PermissionPrompt,
    IdlePrompt,
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventInfo {
    pub timestamp: String,
    #[serde(rename = "event")]
    pub event_type: EventType,
    pub matcher: String,
    pub project_name: String,
    pub project_dir: String,
    pub session_id: String,
    pub message: String,
    #[serde(default)]
    pub notification_type: NotificationType,
    #[serde(default)]
    pub tool_name: String,
}

impl Default for NotificationType {
    fn default() -> Self {
        NotificationType::Other
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub project_name: String,
    pub project_dir: String,
    pub status: SessionStatus,
    pub last_event: String,
    #[serde(default)]
    pub waiting_for: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SessionStatus {
    Active,
    WaitingPermission,
    WaitingInput,
    Completed,
}

impl SessionStatus {
    pub fn emoji(&self) -> &str {
        match self {
            SessionStatus::Active => "🟢",
            SessionStatus::WaitingPermission => "🔐",
            SessionStatus::WaitingInput => "⏳",
            SessionStatus::Completed => "✅",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardData {
    pub sessions: Vec<SessionInfo>,
    pub events: Vec<EventInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    #[serde(default = "Settings::default_always_on_top")]
    pub always_on_top: bool,
    #[serde(default = "Settings::default_opacity_active")]
    pub opacity_active: f64,
    #[serde(default = "Settings::default_opacity_inactive")]
    pub opacity_inactive: f64,
    #[serde(default = "Settings::default_sound_enabled")]
    pub sound_enabled: bool,
}

impl Settings {
    pub const DEFAULT_ALWAYS_ON_TOP: bool = true;
    pub const DEFAULT_OPACITY_ACTIVE: f64 = 1.0;
    pub const DEFAULT_OPACITY_INACTIVE: f64 = 0.3;
    pub const DEFAULT_SOUND_ENABLED: bool = true;

    fn default_always_on_top() -> bool {
        Self::DEFAULT_ALWAYS_ON_TOP
    }

    fn default_opacity_active() -> f64 {
        Self::DEFAULT_OPACITY_ACTIVE
    }

    fn default_opacity_inactive() -> f64 {
        Self::DEFAULT_OPACITY_INACTIVE
    }

    fn default_sound_enabled() -> bool {
        Self::DEFAULT_SOUND_ENABLED
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            always_on_top: Self::DEFAULT_ALWAYS_ON_TOP,
            opacity_active: Self::DEFAULT_OPACITY_ACTIVE,
            opacity_inactive: Self::DEFAULT_OPACITY_INACTIVE,
            sound_enabled: Self::DEFAULT_SOUND_ENABLED,
        }
    }
}

pub struct AppState {
    pub sessions: HashMap<String, SessionInfo>,
    pub recent_events: VecDeque<EventInfo>,
    pub last_file_pos: u64,
    pub settings: Settings,
}

impl AppState {
    pub fn waiting_session_count(&self) -> usize {
        self.sessions
            .values()
            .filter(|s| {
                s.status == SessionStatus::WaitingPermission || s.status == SessionStatus::WaitingInput
            })
            .count()
    }

    pub fn to_dashboard_data(&self) -> DashboardData {
        DashboardData {
            sessions: self.sessions.values().cloned().collect(),
            events: self.recent_events.iter().cloned().collect(),
        }
    }

    /// Insert or update a session with the given status and waiting_for info
    pub fn upsert_session(
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
            recent_events: VecDeque::new(),
            last_file_pos: 0,
            settings: Settings::default(),
        }
    }
}

pub struct ManagedState(pub Arc<Mutex<AppState>>);
