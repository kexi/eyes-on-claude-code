import { useState, useEffect, useRef, useCallback } from 'react';
import { tmuxCapturePane } from '@/lib/tauri';

const POLLING_INTERVAL = 500;

interface TmuxViewerProps {
  paneId: string;
}

export const TmuxViewer = ({ paneId }: TmuxViewerProps) => {
  const [content, setContent] = useState<string>('');
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [isFadedIn, setIsFadedIn] = useState(false);
  const contentRef = useRef<HTMLPreElement>(null);
  const prevContentRef = useRef<string>('');

  const loadContent = useCallback(async () => {
    try {
      const newContent = await tmuxCapturePane(paneId);
      if (newContent !== prevContentRef.current) {
        setContent(newContent);
        prevContentRef.current = newContent;
      }
      setError(null);
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setIsLoading(false);
    }
  }, [paneId]);

  useEffect(() => {
    loadContent();
    const timer = setTimeout(() => setIsFadedIn(true), 50);
    return () => clearTimeout(timer);
  }, [loadContent]);

  useEffect(() => {
    const intervalId = setInterval(loadContent, POLLING_INTERVAL);
    return () => clearInterval(intervalId);
  }, [loadContent]);

  useEffect(() => {
    if (contentRef.current) {
      contentRef.current.scrollTop = contentRef.current.scrollHeight;
    }
  }, [content]);

  return (
    <div
      className={`flex h-screen flex-col bg-bg-primary transition-opacity duration-300 ${
        isFadedIn ? 'opacity-100' : 'opacity-0'
      }`}
    >
      {error && (
        <div className="mx-2 mt-2 rounded bg-red-900/50 px-3 py-2 text-sm text-red-200">{error}</div>
      )}

      <div className="flex-1 overflow-hidden p-2">
        {isLoading && !content ? (
          <div className="flex h-full items-center justify-center text-text-secondary">
            Loading...
          </div>
        ) : (
          <pre
            ref={contentRef}
            className="h-full overflow-y-auto overflow-x-hidden whitespace-pre-wrap break-all rounded bg-black/50 p-3 font-mono text-sm text-text-primary"
          >
            {content || '(empty)'}
          </pre>
        )}
      </div>
    </div>
  );
};
