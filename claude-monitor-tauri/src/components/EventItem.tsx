import type { EventInfo } from '@/types';
import { getEventEmoji, formatTime } from '@/lib/utils';

interface EventItemProps {
  event: EventInfo;
}

export const EventItem = ({ event }: EventItemProps) => {
  return (
    <div className="flex items-center gap-3 py-3 px-4 bg-bg-secondary rounded-lg text-sm">
      <div className="text-base">{getEventEmoji(event.event, event.notification_type)}</div>
      <div className="flex-1">
        <span className="font-medium">{event.project_name}</span>
        <span className="text-text-secondary">
          {' '}
          - {event.event}
          {event.notification_type && event.notification_type !== 'other'
            ? ` (${event.notification_type})`
            : ''}
        </span>
      </div>
      <div className="text-xs text-text-secondary">{formatTime(event.timestamp)}</div>
    </div>
  );
};
