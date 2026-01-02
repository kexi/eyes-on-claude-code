import { useState, useEffect } from 'react';
import type { SessionInfo, GitInfo } from '@/types';
import { getStatusEmoji, getStatusClass } from '@/lib/utils';
import { removeSession, getRepoGitInfo, openDiff, type DiffType } from '@/lib/tauri';
import { ChevronDownIcon } from './icons';
import { DiffButton } from './DiffButton';

interface SessionCardProps {
  session: SessionInfo;
  isMiniView: boolean;
}

export const SessionCard = ({ session, isMiniView }: SessionCardProps) => {
  const [isExpanded, setIsExpanded] = useState(false);
  const [gitInfo, setGitInfo] = useState<GitInfo | null>(null);
  const [isLoadingGit, setIsLoadingGit] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const statusClass = getStatusClass(session.status);

  // Auto-dismiss error after 5 seconds
  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  const handleRemove = async (e: React.MouseEvent) => {
    e.stopPropagation();
    try {
      await removeSession(session.project_dir);
    } catch (error) {
      console.error('Failed to remove session:', error);
    }
  };

  const handleToggleExpand = () => {
    if (isMiniView) return;
    setIsExpanded(!isExpanded);
  };

  // Reset git info when session event changes (e.g., after commit)
  useEffect(() => {
    setGitInfo(null);
  }, [session.last_event]);

  // Load git info when expanded
  useEffect(() => {
    if (isExpanded && !gitInfo && !isLoadingGit) {
      setIsLoadingGit(true);
      setError(null);
      getRepoGitInfo(session.project_dir)
        .then(setGitInfo)
        .catch((err) => {
          const message = err instanceof Error ? err.message : String(err);
          setError(`Failed to load git info: ${message}`);
          console.error('Failed to load git info:', err);
        })
        .finally(() => setIsLoadingGit(false));
    }
  }, [isExpanded, gitInfo, isLoadingGit, session.project_dir]);

  const handleDiffClick = async (type: DiffType) => {
    try {
      setError(null);
      // For branch diff, use the detected default branch
      const baseBranch = type === 'branch' ? gitInfo?.default_branch : undefined;
      await openDiff(session.project_dir, type, baseBranch);
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error('Failed to open diff:', err);
    }
  };

  const borderColor = {
    waiting: 'border-l-4 border-warning',
    completed: 'border-l-4 border-info',
    active: 'border-l-4 border-success',
  }[statusClass];

  if (isMiniView) {
    return (
      <div
        className={`bg-bg-secondary rounded-xl flex items-center transition-all hover:-translate-y-0.5 hover:shadow-lg hover:shadow-black/30 ${borderColor} p-2 gap-2 flex-wrap`}
      >
        <div className="text-base w-6 shrink-0 text-center">
          {getStatusEmoji(session.status)}
        </div>
        <div className="flex-1 min-w-0 overflow-hidden">
          <div className="font-semibold truncate text-xs">{session.project_name}</div>
          <div className="font-mono text-text-secondary truncate text-[0.5rem]">
            {session.project_dir}
          </div>
          {session.waiting_for && (
            <div className="text-warning bg-warning/10 rounded inline-block mt-1 truncate max-w-full text-[0.5rem] py-0.5 px-1">
              ⏸ {session.waiting_for}
            </div>
          )}
        </div>
      </div>
    );
  }

  return (
    <div
      className={`bg-bg-secondary rounded-xl transition-all hover:shadow-lg hover:shadow-black/30 ${borderColor} overflow-hidden`}
    >
      {/* Header - Always visible */}
      <div
        className="flex items-center p-4 px-5 gap-4 cursor-pointer"
        onClick={handleToggleExpand}
      >
        <div className="text-2xl w-10 text-center">{getStatusEmoji(session.status)}</div>

        <div className="flex-1 min-w-0 overflow-hidden">
          <div className="font-semibold truncate mb-1">{session.project_name}</div>
          <div className="font-mono text-text-secondary truncate text-xs">
            {session.project_dir}
          </div>
          {session.waiting_for && (
            <div className="text-warning bg-warning/10 rounded inline-block mt-1 truncate max-w-full text-xs py-1 px-2">
              ⏸ {session.waiting_for}
            </div>
          )}
        </div>

        <div className="flex items-center gap-3">
          <button
            onClick={handleRemove}
            className="w-7 h-7 rounded-md border border-text-secondary text-text-secondary flex items-center justify-center transition-all hover:bg-accent hover:border-accent hover:text-white remove-btn"
            title="Remove session"
          >
            ×
          </button>
          <div
            className={`w-6 h-6 flex items-center justify-center transition-transform ${
              isExpanded ? 'rotate-180' : ''
            }`}
          >
            <ChevronDownIcon className="text-text-secondary" />
          </div>
        </div>
      </div>

      {/* Expanded content - Git info */}
      {isExpanded && (
        <div className="border-t border-bg-card px-5 py-3 space-y-2">
          {error && (
            <div className="text-red-400 bg-red-400/10 rounded px-3 py-2 text-sm flex items-center justify-between">
              <span>{error}</span>
              <button
                onClick={() => setError(null)}
                className="text-red-400 hover:text-red-300 ml-2"
              >
                ×
              </button>
            </div>
          )}
          {isLoadingGit ? (
            <div className="text-text-secondary text-sm">Loading git info...</div>
          ) : gitInfo?.is_git_repo ? (
            <>
              {/* Unstaged changes */}
              <div className="flex items-center justify-between py-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-text-secondary text-sm">unstaged changes:</span>
                  <span
                    className={`text-sm ${
                      gitInfo.has_unstaged_changes ? 'text-red-400' : 'text-text-secondary'
                    }`}
                  >
                    {gitInfo.has_unstaged_changes ? 'Changed' : 'No changes'}
                  </span>
                </div>
                {gitInfo.has_unstaged_changes && (
                  <DiffButton onClick={() => handleDiffClick('unstaged')} />
                )}
              </div>

              {/* Latest commit */}
              <div className="flex items-center justify-between py-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-text-secondary text-sm">latest commit:</span>
                  <span className="text-info text-sm font-mono">#{gitInfo.latest_commit_hash}</span>
                  <span className="text-text-secondary text-sm">{gitInfo.latest_commit_time}</span>
                </div>
                <DiffButton onClick={() => handleDiffClick('commit')} />
              </div>

              {/* Branch */}
              <div className="flex items-center justify-between py-1.5">
                <div className="flex items-center gap-2">
                  <span className="text-text-secondary text-sm">branch:</span>
                  <span className="text-success text-sm">{gitInfo.branch}</span>
                </div>
                <DiffButton onClick={() => handleDiffClick('branch')} />
              </div>
            </>
          ) : (
            <div className="text-text-secondary text-sm">Not a git repository</div>
          )}
        </div>
      )}
    </div>
  );
};
