import type { SessionInfo } from '@/types';

interface HeaderProps {
  sessions: SessionInfo[];
}

export const Header = ({ sessions }: HeaderProps) => {
  const waitingCount = sessions.filter(
    (s) => s.status === 'WaitingPermission' || s.status === 'WaitingInput'
  ).length;

  const isWaiting = waitingCount > 0;

  return (
    <header className="flex justify-between items-center border-b border-bg-card py-1.5 mb-2.5 flex-nowrap">
      <div className="flex items-center gap-3">
        <h1 className="font-semibold text-sm whitespace-nowrap">Eyes on Claude Code</h1>
      </div>
      <div className="flex items-center gap-2 bg-bg-card rounded-full py-0.5 px-2 text-[0.625rem] whitespace-nowrap shrink-0">
        <div
          className={`w-2 h-2 rounded-full bg-success ${isWaiting ? 'bg-warning animate-pulse-slow' : ''}`}
        />
        <span>{isWaiting ? `${waitingCount} waiting` : 'Monitoring'}</span>
      </div>
    </header>
  );
};
