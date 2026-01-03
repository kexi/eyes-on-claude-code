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
