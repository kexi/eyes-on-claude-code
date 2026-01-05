import { useState, useEffect, useRef, useCallback, useMemo } from 'react';
import { getCurrentWindow } from '@tauri-apps/api/window';
import { AnsiUp } from 'ansi_up';
import { tmuxCapturePane, tmuxSendKeys } from '@/lib/tauri';

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

  const ansiUp = useMemo(() => {
    const instance = new AnsiUp();
    instance.use_classes = true;
    return instance;
  }, []);

  const htmlContent = useMemo(() => {
    return ansiUp.ansi_to_html(content);
  }, [ansiUp, content]);

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

  const handleClose = async () => {
    try {
      await getCurrentWindow().close();
    } catch (err) {
      console.error('Failed to close window:', err);
    }
  };

  const convertKeyToTmux = (e: KeyboardEvent): string | null => {
    // Ignore modifier-only keys
    if (['Control', 'Alt', 'Shift', 'Meta'].includes(e.key)) {
      return null;
    }

    // Handle Ctrl+key combinations
    if (e.ctrlKey && e.key.length === 1) {
      return `C-${e.key.toLowerCase()}`;
    }

    // Handle special keys
    const keyMap: Record<string, string> = {
      Enter: 'Enter',
      Escape: 'Escape',
      Backspace: 'BSpace',
      Tab: 'Tab',
      ArrowUp: 'Up',
      ArrowDown: 'Down',
      ArrowLeft: 'Left',
      ArrowRight: 'Right',
      Home: 'Home',
      End: 'End',
      PageUp: 'PageUp',
      PageDown: 'PageDown',
      Delete: 'DC',
      Insert: 'IC',
      F1: 'F1',
      F2: 'F2',
      F3: 'F3',
      F4: 'F4',
      F5: 'F5',
      F6: 'F6',
      F7: 'F7',
      F8: 'F8',
      F9: 'F9',
      F10: 'F10',
      F11: 'F11',
      F12: 'F12',
    };

    if (keyMap[e.key]) {
      return keyMap[e.key];
    }

    // Regular character
    if (e.key.length === 1) {
      return e.key;
    }

    return null;
  };

  const handleKeyDown = useCallback(
    async (e: KeyboardEvent) => {
      const tmuxKey = convertKeyToTmux(e);
      if (tmuxKey) {
        e.preventDefault();
        try {
          await tmuxSendKeys(paneId, tmuxKey);
        } catch (err) {
          console.error('Failed to send key:', err);
        }
      }
    },
    [paneId]
  );

  useEffect(() => {
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleKeyDown]);

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
      className={`relative flex h-screen flex-col bg-bg-primary transition-opacity duration-300 ${
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
            className="ansi-content h-full overflow-y-auto overflow-x-hidden whitespace-pre-wrap break-all rounded bg-black/50 p-3 font-mono text-sm text-text-primary"
            dangerouslySetInnerHTML={{ __html: htmlContent || '(empty)' }}
          />
        )}
      </div>

      <button
        onClick={handleClose}
        className="absolute bottom-4 left-1/2 -translate-x-1/2 rounded-lg bg-bg-card px-8 py-2 text-base text-text-secondary hover:bg-white/20 hover:text-text-primary transition-colors"
      >
        Close
      </button>
    </div>
  );
};
