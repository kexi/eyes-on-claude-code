import type { SessionStatus, EventType, NotificationType } from '@/types';

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

export const getStatusText = (status: SessionStatus): string => {
  switch (status) {
    case 'WaitingPermission':
      return 'Permission Required';
    case 'WaitingInput':
      return 'Waiting for Input';
    case 'Completed':
      return 'Completed';
    case 'Active':
      return 'Active';
    default:
      return status;
  }
};

export const getEventEmoji = (event: EventType, notificationType?: NotificationType): string => {
  if (event === 'notification') {
    switch (notificationType) {
      case 'permission_prompt':
        return '🔐';
      case 'idle_prompt':
        return '⏳';
      default:
        return '🔔';
    }
  }
  switch (event) {
    case 'stop':
      return '✅';
    case 'session_start':
      return '🚀';
    case 'session_end':
      return '🏁';
    case 'post_tool_use':
      return '🔧';
    default:
      return '📌';
  }
};

export const formatTime = (timestamp: string): string => {
  try {
    const date = new Date(timestamp);
    return date.toLocaleTimeString('ja-JP', { hour: '2-digit', minute: '2-digit' });
  } catch {
    return timestamp;
  }
};
