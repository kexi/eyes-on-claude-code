import type { SessionInfo } from '@/types';
import { getStatusEmoji, getStatusClass, getStatusText, formatTime } from '@/lib/utils';
import { removeSession } from '@/lib/tauri';

interface SessionCardProps {
  session: SessionInfo;
  isMiniView: boolean;
}

export const SessionCard = ({ session, isMiniView }: SessionCardProps) => {
  const statusClass = getStatusClass(session.status);

  const handleRemove = async () => {
    try {
      await removeSession(session.project_dir);
    } catch (error) {
      console.error('Failed to remove session:', error);
    }
  };

  const borderColor = {
    waiting: 'border-l-4 border-warning',
    completed: 'border-l-4 border-info',
    active: 'border-l-4 border-success',
  }[statusClass];

  const statusTextColor = {
    waiting: 'text-warning',
    completed: 'text-info',
    active: 'text-success',
  }[statusClass];

  return (
    <div
      className={`bg-bg-secondary rounded-xl flex items-center transition-all hover:-translate-y-0.5 hover:shadow-lg hover:shadow-black/30 ${borderColor} ${
        isMiniView ? 'p-2 gap-2 flex-wrap' : 'p-4 px-5 gap-4'
      }`}
    >
      <div className={`text-center ${isMiniView ? 'text-base w-6 shrink-0' : 'text-2xl w-10'}`}>
        {getStatusEmoji(session.status)}
      </div>

      <div className={`flex-1 min-w-0 overflow-hidden`}>
        <div className={`font-semibold truncate ${isMiniView ? 'text-xs' : 'mb-1'}`}>
          {session.project_name}
        </div>
        <div
          className={`font-mono text-text-secondary truncate ${
            isMiniView ? 'text-[0.5rem]' : 'text-xs'
          }`}
        >
          {session.project_dir}
        </div>
        {session.waiting_for && (
          <div
            className={`text-warning bg-warning/10 rounded inline-block mt-1 truncate max-w-full ${
              isMiniView ? 'text-[0.5rem] py-0.5 px-1' : 'text-xs py-1 px-2'
            }`}
          >
            ⏸ {session.waiting_for}
          </div>
        )}
      </div>

      {!isMiniView && (
        <div className="flex items-center">
          <div className="text-right">
            <div className={`text-sm font-medium ${statusTextColor}`}>
              {getStatusText(session.status)}
            </div>
            <div className="text-xs text-text-secondary mt-0.5">
              {formatTime(session.last_event)}
            </div>
          </div>
          <button
            onClick={handleRemove}
            className="ml-3 w-7 h-7 rounded-md border border-text-secondary text-text-secondary flex items-center justify-center transition-all hover:bg-accent hover:border-accent hover:text-white"
            title="Remove session"
          >
            ×
          </button>
        </div>
      )}
    </div>
  );
};
