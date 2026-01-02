import { StatCard } from './StatCard';
import type { SessionInfo, EventInfo } from '@/types';

interface StatsGridProps {
  sessions: SessionInfo[];
  events: EventInfo[];
}

export const StatsGrid = ({ sessions, events }: StatsGridProps) => {
  const totalSessions = sessions.length;
  const waitingSessions = sessions.filter(
    (s) => s.status === 'WaitingPermission' || s.status === 'WaitingInput'
  ).length;
  const totalEvents = events.length;

  return (
    <div className="grid grid-cols-3 gap-4 mb-6">
      <StatCard value={totalSessions} label="Active Sessions" />
      <StatCard value={waitingSessions} label="Waiting" />
      <StatCard value={totalEvents} label="Events Today" />
    </div>
  );
};
