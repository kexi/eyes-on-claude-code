// Session status matching Rust enum
export type SessionStatus = 'Active' | 'WaitingPermission' | 'WaitingInput' | 'Completed';

// Notification type matching Rust enum (snake_case from serde)
export type NotificationType = 'permission_prompt' | 'idle_prompt' | 'other';

// Event type matching Rust enum (snake_case from serde)
export type EventType =
  | 'session_start'
  | 'session_end'
  | 'notification'
  | 'stop'
  | 'post_tool_use'
  | 'unknown';

export interface SessionInfo {
  project_name: string;
  project_dir: string;
  status: SessionStatus;
  last_event: string;
  waiting_for: string;
}

export interface EventInfo {
  timestamp: string;
  event: EventType;
  matcher: string;
  project_name: string;
  project_dir: string;
  session_id: string;
  message: string;
  notification_type: NotificationType;
  tool_name: string;
}

export interface DashboardData {
  sessions: SessionInfo[];
  events: EventInfo[];
}

export interface Settings {
  always_on_top: boolean;
  mini_view: boolean;
  opacity_active: number;
  opacity_inactive: number;
  sound_enabled: boolean;
}

export interface GitInfo {
  branch: string;
  latest_commit_hash: string;
  latest_commit_time: string;
  has_unstaged_changes: boolean;
  is_git_repo: boolean;
}
