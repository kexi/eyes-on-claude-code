import type { SessionInfo } from '@/types';

interface HeaderProps {
  sessions: SessionInfo[];
  isMiniView: boolean;
}

export const Header = ({ sessions, isMiniView }: HeaderProps) => {
  const waitingCount = sessions.filter(
    (s) => s.status === 'WaitingPermission' || s.status === 'WaitingInput'
  ).length;

  const isWaiting = waitingCount > 0;

  return (
    <header
      className={`flex justify-between items-center border-b border-bg-card ${
        isMiniView ? 'py-1.5 mb-2.5 flex-nowrap' : 'py-5 mb-6'
      }`}
    >
      <div className="flex items-center gap-3">
        <h1 className={`font-semibold ${isMiniView ? 'text-sm whitespace-nowrap' : 'text-2xl'}`}>
          Eyes on Claude Code
        </h1>
      </div>
      <div
        className={`flex items-center gap-2 bg-bg-card rounded-full ${
          isMiniView ? 'py-0.5 px-2 text-[0.625rem] whitespace-nowrap shrink-0' : 'py-2 px-4 text-sm'
        }`}
      >
        <div
          className={`rounded-full bg-success ${isWaiting ? 'bg-warning animate-pulse-slow' : ''} ${
            isMiniView ? 'w-2 h-2' : 'w-2.5 h-2.5'
          }`}
        />
        <span>{isWaiting ? `${waitingCount} waiting` : 'Monitoring'}</span>
      </div>
    </header>
  );
};
