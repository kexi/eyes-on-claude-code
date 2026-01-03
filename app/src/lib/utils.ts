import type { SessionStatus, HookStatus } from '@/types';

export const getStatusEmoji = (status: SessionStatus): string => {
  switch (status) {
    case 'WaitingPermission':
      return '🔐';
    case 'WaitingInput':
      return '⏳';
    case 'Completed':
      return '✅';
    case 'Active':
      return '🟢';
    default:
      return '📌';
  }
};

export const getStatusClass = (status: SessionStatus): 'waiting' | 'completed' | 'active' => {
  switch (status) {
    case 'WaitingPermission':
    case 'WaitingInput':
      return 'waiting';
    case 'Completed':
      return 'completed';
    case 'Active':
    default:
      return 'active';
  }
};

export const allHooksConfigured = (hooks: HookStatus): boolean => {
  return (
    hooks.session_start &&
    hooks.session_end &&
    hooks.stop &&
    hooks.post_tool_use &&
    hooks.user_prompt_submit &&
    hooks.notification_permission &&
    hooks.notification_idle
  );
};

// Format ISO timestamp to relative time (e.g., "2m ago", "1h ago")
export const formatRelativeTime = (isoTimestamp: string): string => {
  if (!isoTimestamp) return '';

  const date = new Date(isoTimestamp);

  // Check for Invalid Date
  if (isNaN(date.getTime())) return '';

  const now = new Date();
  const diffMs = now.getTime() - date.getTime();

  // Handle future dates
  if (diffMs < 0) return 'just now';

  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHour = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHour / 24);

  if (diffSec < 60) return 'just now';
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHour < 24) return `${diffHour}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;

  // For older dates, show date
  return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
};
