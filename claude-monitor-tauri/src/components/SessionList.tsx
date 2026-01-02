import type { SessionInfo } from '@/types';
import { SessionCard } from './SessionCard';
import { EmptyState } from './EmptyState';

interface SessionListProps {
  sessions: SessionInfo[];
  isMiniView: boolean;
  onRefresh: () => void;
}

export const SessionList = ({ sessions, isMiniView, onRefresh }: SessionListProps) => {
  return (
    <div className={isMiniView ? 'mb-2.5' : 'mb-6'}>
      <div className={`flex justify-between items-center ${isMiniView ? 'mb-1.5' : 'mb-4'}`}>
        <h2 className={`font-semibold ${isMiniView ? 'text-xs' : 'text-lg'}`}>Sessions</h2>
        <button
          onClick={onRefresh}
          className={`bg-bg-card border-none text-text-primary rounded-lg cursor-pointer transition-colors hover:bg-accent ${
            isMiniView ? 'py-0.5 px-2 text-[0.625rem]' : 'py-2 px-4 text-sm'
          }`}
        >
          Refresh
        </button>
      </div>
      <div className={`flex flex-col ${isMiniView ? 'gap-2' : 'gap-3'}`}>
        {sessions.length === 0 ? (
          <EmptyState icon="📭" message="No active sessions" isMiniView={isMiniView} />
        ) : (
          sessions.map((session) => (
            <SessionCard
              key={session.project_dir || session.project_name}
              session={session}
              isMiniView={isMiniView}
            />
          ))
        )}
      </div>
    </div>
  );
};
