import { useEffect, useState, useMemo } from 'react';
import { AppProvider } from '@/context/AppContext';
import { useAppContext } from '@/context/useAppContext';
import { useWindowOpacity } from '@/hooks/useWindowOpacity';
import { useWindowDrag } from '@/hooks/useWindowDrag';
import { Header } from '@/components/Header';
import { SessionList } from '@/components/SessionList';
import { SetupModal } from '@/components/SetupModal';
import { TmuxPanel } from '@/components/TmuxPanel';
import { TmuxViewer } from '@/components/TmuxViewer';
import {
  onWindowFocus,
  bringDiffWindowsToFront,
  getSetupStatus,
  setWindowSizeForSetup,
} from '@/lib/tauri';
import { allHooksConfigured } from '@/lib/utils';
import type { SetupStatus } from '@/types';

type TabType = 'sessions' | 'tmux';

const Dashboard = () => {
  const { dashboardData, settings, isLoading, refreshData } = useAppContext();
  const [activeTab, setActiveTab] = useState<TabType>('sessions');

  // Always apply mini-view class to body
  useEffect(() => {
    document.body.classList.add('mini-view');
    return () => {
      document.body.classList.remove('mini-view');
    };
  }, []);

  // Handle window opacity based on focus
  useWindowOpacity(settings.opacity_active, settings.opacity_inactive);

  // Handle window drag
  useWindowDrag();

  // Bring diff windows to front when dashboard is focused (via Cmd+Tab etc.)
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    onWindowFocus(() => {
      bringDiffWindowsToFront().catch(console.error);
    }).then((u) => {
      unlisten = u;
    });
    return () => unlisten?.();
  }, []);

  if (isLoading) {
    return (
      <div className="container bg-bg-primary h-screen rounded-xl max-w-[900px] mx-auto p-2.5 flex items-center justify-center">
        <div className="text-text-secondary">Loading...</div>
      </div>
    );
  }

  return (
    <div className="container bg-bg-primary h-screen rounded-xl max-w-[900px] mx-auto flex flex-col p-2.5">
      <Header sessions={dashboardData.sessions} onRefresh={refreshData} />

      {/* Tab navigation */}
      <div className="flex gap-1 mb-2 border-b border-white/10 pb-1">
        <button
          onClick={() => setActiveTab('sessions')}
          className={`px-3 py-1 text-sm rounded-t ${
            activeTab === 'sessions'
              ? 'bg-bg-card text-text-primary'
              : 'text-text-secondary hover:text-text-primary'
          }`}
        >
          Sessions
        </button>
        <button
          onClick={() => setActiveTab('tmux')}
          className={`px-3 py-1 text-sm rounded-t ${
            activeTab === 'tmux'
              ? 'bg-bg-card text-text-primary'
              : 'text-text-secondary hover:text-text-primary'
          }`}
        >
          tmux
        </button>
      </div>

      {/* Tab content */}
      {activeTab === 'sessions' ? (
        <SessionList sessions={dashboardData.sessions} />
      ) : (
        <TmuxPanel />
      )}
    </div>
  );
};

function App() {
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null);
  const [showSetupModal, setShowSetupModal] = useState(false);
  const [setupChecked, setSetupChecked] = useState(false);

  // Parse URL parameters to check if this is a tmux viewer window
  const tmuxPaneId = useMemo(() => {
    const params = new URLSearchParams(window.location.search);
    return params.get('tmux_pane');
  }, []);

  // Check setup status on mount (skip for tmux viewer windows)
  useEffect(() => {
    if (tmuxPaneId) {
      setSetupChecked(true);
      return;
    }

    getSetupStatus()
      .then((status) => {
        setSetupStatus(status);
        // Show modal if any hook is missing or there's an init error
        if (!allHooksConfigured(status.hooks) || status.init_error) {
          setShowSetupModal(true);
          // Enlarge window for setup modal
          setWindowSizeForSetup(true).catch(console.error);
        }
        setSetupChecked(true);
      })
      .catch((err) => {
        console.error('Failed to get setup status:', err);
        setSetupChecked(true);
      });
  }, [tmuxPaneId]);

  const handleSetupComplete = () => {
    setShowSetupModal(false);
    // Restore miniview size
    setWindowSizeForSetup(false).catch(console.error);
  };

  // Render tmux viewer if pane_id is in URL
  if (tmuxPaneId) {
    return <TmuxViewer paneId={tmuxPaneId} />;
  }

  // Wait for setup check before showing anything
  if (!setupChecked) {
    return (
      <div className="bg-bg-primary h-screen flex items-center justify-center">
        <div className="text-text-secondary">Checking setup...</div>
      </div>
    );
  }

  return (
    <AppProvider>
      <Dashboard />
      {showSetupModal && setupStatus && (
        <SetupModal setupStatus={setupStatus} onComplete={handleSetupComplete} />
      )}
    </AppProvider>
  );
}

export default App;
