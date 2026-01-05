import type { SessionInfo } from '@/types';
import { SessionCard } from './SessionCard';
import { EmptyState } from './EmptyState';

interface SessionListProps {
  sessions: SessionInfo[];
}

export const SessionList = ({ sessions }: SessionListProps) => {
  return (
    <div className="flex-1 overflow-y-scroll min-h-0">
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
