import { useState, useCallback, useRef, useEffect } from 'react';
import type { SetupStatus, HookStatus } from '@/types';
import { checkClaudeSettings } from '@/lib/tauri';

interface SetupModalProps {
  setupStatus: SetupStatus;
  onComplete: () => void;
}

// Hook display names
const HOOK_LABELS: Record<keyof HookStatus, string> = {
  session_start: 'SessionStart',
  session_end: 'SessionEnd',
  stop: 'Stop',
  post_tool_use: 'PostToolUse',
  notification_permission: 'Notification (permission_prompt)',
  notification_idle: 'Notification (idle_prompt)',
};

// Check if all hooks are configured
const allHooksConfigured = (hooks: HookStatus): boolean => {
  return (
    hooks.session_start &&
    hooks.session_end &&
    hooks.stop &&
    hooks.post_tool_use &&
    hooks.notification_permission &&
    hooks.notification_idle
  );
};

export const SetupModal = ({ setupStatus: initialStatus, onComplete }: SetupModalProps) => {
  const [status, setStatus] = useState<SetupStatus>(initialStatus);
  const [isChecking, setIsChecking] = useState(false);
  const [copied, setCopied] = useState(false);

  const copyTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const completeTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Cleanup timeouts on unmount
  useEffect(() => {
    return () => {
      if (copyTimeoutRef.current) clearTimeout(copyTimeoutRef.current);
      if (completeTimeoutRef.current) clearTimeout(completeTimeoutRef.current);
    };
  }, []);

  const handleCopy = useCallback(async () => {
    try {
      await navigator.clipboard.writeText(status.merged_settings);
      setCopied(true);
      if (copyTimeoutRef.current) clearTimeout(copyTimeoutRef.current);
      copyTimeoutRef.current = setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [status.merged_settings]);

  const handleCheckAgain = useCallback(async () => {
    setIsChecking(true);
    try {
      const newStatus = await checkClaudeSettings();
      setStatus(newStatus);
      if (allHooksConfigured(newStatus.hooks) && !newStatus.init_error) {
        if (completeTimeoutRef.current) clearTimeout(completeTimeoutRef.current);
        completeTimeoutRef.current = setTimeout(onComplete, 1500);
      }
    } catch (err) {
      console.error('Failed to check settings:', err);
    } finally {
      setIsChecking(false);
    }
  }, [onComplete]);

  const isAllConfigured = allHooksConfigured(status.hooks) && !status.init_error;

  if (isAllConfigured) {
    return (
      <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
        <div className="bg-bg-primary rounded-xl p-6 max-w-md w-full mx-4 shadow-2xl">
          <div className="flex flex-col items-center gap-4">
            <div className="w-16 h-16 rounded-full bg-success/20 flex items-center justify-center">
              <svg className="w-8 h-8 text-success" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
            </div>
            <h2 className="text-xl font-semibold text-text-primary">Setup Complete</h2>
            <p className="text-text-secondary text-center">
              Claude Code hooks are configured correctly.
            </p>
          </div>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-bg-primary rounded-xl p-6 max-w-2xl w-full mx-4 shadow-2xl max-h-[90vh] overflow-y-auto">
        <h2 className="text-xl font-semibold text-text-primary mb-4">Setup Required</h2>

        <div className="space-y-4">
          {status.init_error && (
            <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-4">
              <p className="text-red-400 text-sm font-medium mb-1">Initialization Error</p>
              <p className="text-text-primary text-sm break-words">{status.init_error}</p>
            </div>
          )}

          {/* Hook Status List */}
          <div className="bg-bg-card rounded-lg p-4">
            <p className="text-text-primary text-sm font-medium mb-3">Hook Configuration Status:</p>
            <div className="space-y-2">
              {(Object.keys(HOOK_LABELS) as Array<keyof HookStatus>).map((key) => (
                <div key={key} className="flex items-center justify-between text-sm">
                  <span className="text-text-secondary">{HOOK_LABELS[key]}</span>
                  {status.hooks[key] ? (
                    <span className="text-success font-medium">OK</span>
                  ) : (
                    <span className="text-red-400 font-medium">NG</span>
                  )}
                </div>
              ))}
            </div>
          </div>

          <div className="bg-warning/10 border border-warning/30 rounded-lg p-4">
            <p className="text-text-primary text-sm">
              Some hooks are not configured. Please copy the settings below and replace your{' '}
              <code className="bg-bg-card px-1.5 py-0.5 rounded text-xs">~/.claude/settings.json</code>{' '}
              file:
            </p>
          </div>

          <div className="relative">
            <pre className="bg-bg-card rounded-lg p-4 text-xs text-text-secondary overflow-x-auto max-h-64">
              {status.merged_settings}
            </pre>
            <button
              onClick={handleCopy}
              className={`absolute top-2 right-2 px-3 py-1.5 rounded text-xs font-medium transition-colors ${
                copied
                  ? 'bg-success text-white'
                  : 'bg-bg-primary hover:bg-bg-primary/80 text-text-primary'
              }`}
            >
              {copied ? 'Copied!' : 'Copy'}
            </button>
          </div>

          <div className="text-text-secondary text-sm space-y-2">
            <p>Steps:</p>
            <ol className="list-decimal list-inside space-y-1 ml-2">
              <li>Copy the settings above</li>
              <li>
                Open{' '}
                <code className="bg-bg-card px-1.5 py-0.5 rounded text-xs">~/.claude/settings.json</code>
              </li>
              <li>Replace the contents with the copied settings</li>
              <li>Save the file</li>
              <li>Click "Check Again" below</li>
            </ol>
          </div>

          {status.hook_path && (
            <div className="text-text-secondary text-xs">
              <span className="opacity-70">Hook script location: </span>
              <code className="bg-bg-card px-1.5 py-0.5 rounded">{status.hook_path}</code>
            </div>
          )}

          <div className="flex justify-end gap-3 pt-4 border-t border-bg-card">
            <button
              onClick={handleCheckAgain}
              disabled={isChecking}
              className="px-4 py-2 bg-accent hover:bg-accent/80 disabled:opacity-50 disabled:cursor-not-allowed text-white rounded-lg text-sm font-medium transition-colors flex items-center gap-2"
            >
              {isChecking && (
                <svg className="w-4 h-4 animate-spin" viewBox="0 0 24 24">
                  <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4" fill="none" />
                  <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                </svg>
              )}
              Check Again
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
