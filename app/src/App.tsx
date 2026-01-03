import { useEffect, useState } from 'react';
import { AppProvider, useAppContext } from '@/context/AppContext';
import { useWindowOpacity } from '@/hooks/useWindowOpacity';
import { useWindowDrag } from '@/hooks/useWindowDrag';
import { Header } from '@/components/Header';
import { StatsGrid } from '@/components/StatsGrid';
import { SessionList } from '@/components/SessionList';
import { EventList } from '@/components/EventList';
import { SetupModal } from '@/components/SetupModal';
import { onWindowFocus, bringDiffWindowsToFront, getSetupStatus } from '@/lib/tauri';
import type { SetupStatus, HookStatus } from '@/types';

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

const Dashboard = () => {
  const { dashboardData, settings, isLoading, refreshData } = useAppContext();
  const isMiniView = settings.mini_view;

  // Apply mini-view class to body
  useEffect(() => {
    if (isMiniView) {
      document.body.classList.add('mini-view');
    } else {
      document.body.classList.remove('mini-view');
    }
    return () => {
      document.body.classList.remove('mini-view');
    };
  }, [isMiniView]);

  // Handle window opacity based on focus
  useWindowOpacity(settings.opacity_active, settings.opacity_inactive);

  // Handle window drag in mini-view mode
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
      <div className="container bg-bg-primary h-screen rounded-xl max-w-[900px] mx-auto p-5 flex items-center justify-center">
        <div className="text-text-secondary">Loading...</div>
      </div>
    );
  }

  return (
    <div
      className={`container bg-bg-primary h-screen rounded-xl max-w-[900px] mx-auto overflow-y-auto ${
        isMiniView ? 'p-2.5' : 'p-5'
      }`}
    >
      <Header sessions={dashboardData.sessions} isMiniView={isMiniView} />

      {!isMiniView && (
        <StatsGrid sessions={dashboardData.sessions} events={dashboardData.events} />
      )}

      <SessionList
        sessions={dashboardData.sessions}
        isMiniView={isMiniView}
        onRefresh={refreshData}
      />

      {!isMiniView && <EventList events={dashboardData.events} />}
    </div>
  );
};

function App() {
  const [setupStatus, setSetupStatus] = useState<SetupStatus | null>(null);
  const [showSetupModal, setShowSetupModal] = useState(false);
  const [setupChecked, setSetupChecked] = useState(false);

  // Check setup status on mount
  useEffect(() => {
    getSetupStatus()
      .then((status) => {
        setSetupStatus(status);
        // Show modal if any hook is missing or there's an init error
        if (!allHooksConfigured(status.hooks) || status.init_error) {
          setShowSetupModal(true);
        }
        setSetupChecked(true);
      })
      .catch((err) => {
        console.error('Failed to get setup status:', err);
        setSetupChecked(true);
      });
  }, []);

  const handleSetupComplete = () => {
    setShowSetupModal(false);
  };

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
