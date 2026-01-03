import type { SessionInfo } from '@/types';
import { SessionCard } from './SessionCard';
import { EmptyState } from './EmptyState';

interface SessionListProps {
  sessions: SessionInfo[];
  onRefresh: () => void;
}

export const SessionList = ({ sessions, onRefresh }: SessionListProps) => {
  return (
    <div className="mb-2.5">
      <div className="flex justify-between items-center mb-1.5">
        <h2 className="font-semibold text-xs">Sessions</h2>
        <button
          onClick={onRefresh}
          className="bg-bg-card border-none text-text-primary rounded-lg cursor-pointer transition-colors hover:bg-accent py-0.5 px-2 text-[0.625rem]"
        >
          Refresh
        </button>
      </div>
      <div className="flex flex-col gap-2">
        {sessions.length === 0 ? (
          <EmptyState icon="📭" message="No active sessions" />
        ) : (
          sessions.map((session) => (
            <SessionCard key={session.project_dir || session.project_name} session={session} />
          ))
        )}
      </div>
    </div>
  );
};
