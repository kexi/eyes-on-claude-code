import type { EventInfo } from '@/types';
import { EventItem } from './EventItem';
import { EmptyState } from './EmptyState';

interface EventListProps {
  events: EventInfo[];
}

export const EventList = ({ events }: EventListProps) => {
  const displayEvents = [...events].reverse().slice(0, 10);

  return (
    <div className="mb-6">
      <div className="flex justify-between items-center mb-4">
        <h2 className="font-semibold text-lg">Recent Events</h2>
      </div>
      <div className="flex flex-col gap-2">
        {displayEvents.length === 0 ? (
          <EmptyState icon="📋" message="No events yet" />
        ) : (
          displayEvents.map((event, idx) => <EventItem key={`${event.timestamp}-${idx}`} event={event} />)
        )}
      </div>
    </div>
  );
};
