import { useState, useEffect, useCallback } from 'react';
import type { TmuxPane } from '@/types';
import { tmuxIsAvailable, tmuxListPanes, tmuxCapturePane, tmuxSendKeys } from '@/lib/tauri';
import { RefreshIcon } from './icons';

interface TmuxTreeNode {
  session: string;
  windows: {
    index: number;
    name: string;
    panes: TmuxPane[];
  }[];
}

function buildTree(panes: TmuxPane[]): TmuxTreeNode[] {
  const sessionMap = new Map<string, TmuxTreeNode>();

  for (const pane of panes) {
    if (!sessionMap.has(pane.session_name)) {
      sessionMap.set(pane.session_name, {
        session: pane.session_name,
        windows: [],
      });
    }
    const node = sessionMap.get(pane.session_name)!;

    let window = node.windows.find((w) => w.index === pane.window_index);
    if (!window) {
      window = { index: pane.window_index, name: pane.window_name, panes: [] };
      node.windows.push(window);
    }
    window.panes.push(pane);
  }

  return Array.from(sessionMap.values());
}

export const TmuxPanel = () => {
  const [isAvailable, setIsAvailable] = useState<boolean | null>(null);
  const [panes, setPanes] = useState<TmuxPane[]>([]);
  const [selectedPane, setSelectedPane] = useState<TmuxPane | null>(null);
  const [paneContent, setPaneContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const checkAvailability = useCallback(async () => {
    try {
      const available = await tmuxIsAvailable();
      setIsAvailable(available);
      if (available) {
        await refreshPanes();
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setIsAvailable(false);
    }
  }, []);

  const refreshPanes = useCallback(async () => {
    setIsLoading(true);
    setError(null);
    try {
      const result = await tmuxListPanes();
      setPanes(result);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, []);

  const loadPaneContent = useCallback(async (pane: TmuxPane) => {
    setSelectedPane(pane);
    setError(null);
    try {
      const content = await tmuxCapturePane(pane.pane_id);
      setPaneContent(content);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
      setPaneContent('');
    }
  }, []);

  const handleSendKeys = useCallback(
    async (keys: string) => {
      if (!selectedPane) return;
      setError(null);
      try {
        await tmuxSendKeys(selectedPane.pane_id, keys);
        await loadPaneContent(selectedPane);
      } catch (err) {
        setError(err instanceof Error ? err.message : String(err));
      }
    },
    [selectedPane, loadPaneContent]
  );

  useEffect(() => {
    checkAvailability();
  }, [checkAvailability]);

  useEffect(() => {
    if (error) {
      const timer = setTimeout(() => setError(null), 5000);
      return () => clearTimeout(timer);
    }
  }, [error]);

  if (isAvailable === null) {
    return (
      <div className="flex h-full items-center justify-center text-text-secondary">
        Checking tmux availability...
      </div>
    );
  }

  if (!isAvailable) {
    return (
      <div className="flex h-full flex-col items-center justify-center gap-2 text-text-secondary">
        <span className="text-lg">tmux is not available</span>
        <span className="text-sm">Please install tmux and start a session to use this feature.</span>
      </div>
    );
  }

  const tree = buildTree(panes);

  return (
    <div className="flex h-full flex-col">
      {error && (
        <div className="mx-2 mt-2 rounded bg-red-900/50 px-3 py-2 text-sm text-red-200">{error}</div>
      )}

      <div className="flex flex-1 overflow-hidden">
        {/* Left: Tree view */}
        <div className="w-1/3 overflow-y-auto border-r border-white/10 p-2">
          <div className="mb-2 flex items-center justify-between">
            <span className="text-sm font-semibold text-text-secondary">Sessions</span>
            <button
              onClick={refreshPanes}
              disabled={isLoading}
              className="rounded p-1 text-text-secondary hover:bg-white/10 hover:text-text-primary disabled:opacity-50"
              title="Refresh"
            >
              <RefreshIcon className={isLoading ? 'animate-spin' : ''} />
            </button>
          </div>

          {tree.length === 0 ? (
            <div className="text-sm text-text-secondary">No tmux sessions</div>
          ) : (
            tree.map((node) => (
              <div key={node.session} className="mb-2">
                <div className="text-sm font-medium text-text-primary">{node.session}</div>
                {node.windows.map((window) => (
                  <div key={window.index} className="ml-2">
                    <div className="text-xs text-text-secondary">
                      {window.index}: {window.name}
                    </div>
                    {window.panes.map((pane) => (
                      <button
                        key={pane.pane_id}
                        onClick={() => loadPaneContent(pane)}
                        className={`ml-2 block w-full rounded px-2 py-0.5 text-left text-xs ${
                          selectedPane?.pane_id === pane.pane_id
                            ? 'bg-accent text-white'
                            : 'text-text-secondary hover:bg-white/10'
                        }`}
                      >
                        {pane.pane_id} {pane.is_active && '(active)'}
                      </button>
                    ))}
                  </div>
                ))}
              </div>
            ))
          )}
        </div>

        {/* Right: Pane content */}
        <div className="flex flex-1 flex-col overflow-hidden">
          <div className="flex-1 overflow-auto bg-black/30 p-2">
            {selectedPane ? (
              <pre className="whitespace-pre-wrap font-mono text-xs text-text-primary">
                {paneContent || '(empty)'}
              </pre>
            ) : (
              <div className="flex h-full items-center justify-center text-sm text-text-secondary">
                Select a pane to view its content
              </div>
            )}
          </div>

          {/* Bottom: Send keys buttons */}
          {selectedPane && (
            <div className="flex gap-2 border-t border-white/10 p-2">
              <button
                onClick={() => handleSendKeys('Enter')}
                className="rounded bg-bg-card px-3 py-1 text-sm text-text-primary hover:bg-white/20"
              >
                Enter
              </button>
              <button
                onClick={() => handleSendKeys('y')}
                className="rounded bg-bg-card px-3 py-1 text-sm text-text-primary hover:bg-white/20"
              >
                y
              </button>
              <button
                onClick={() => handleSendKeys('n')}
                className="rounded bg-bg-card px-3 py-1 text-sm text-text-primary hover:bg-white/20"
              >
                n
              </button>
              <button
                onClick={() => loadPaneContent(selectedPane)}
                className="ml-auto rounded bg-bg-card px-3 py-1 text-sm text-text-primary hover:bg-white/20"
              >
                Refresh
              </button>
            </div>
          )}
        </div>
      </div>
    </div>
  );
};
