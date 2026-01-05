import { useEffect, useState } from 'react';
import { AppProvider } from '@/context/AppContext';
import { useAppContext } from '@/context/useAppContext';
import { useWindowOpacity } from '@/hooks/useWindowOpacity';
import { useWindowDrag } from '@/hooks/useWindowDrag';
import { Header } from '@/components/Header';
import { SessionList } from '@/components/SessionList';
import { SetupModal } from '@/components/SetupModal';
import {
  onWindowFocus,
  bringDiffWindowsToFront,
  getSetupStatus,
  setWindowSizeForSetup,
} from '@/lib/tauri';
import { allHooksConfigured } from '@/lib/utils';
import type { SetupStatus } from '@/types';

const Dashboard = () => {
  const { dashboardData, settings, isLoading, refreshData } = useAppContext();

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
      <SessionList sessions={dashboardData.sessions} />
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
          // Enlarge window for setup modal
          setWindowSizeForSetup(true).catch(console.error);
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
    // Restore miniview size
    setWindowSizeForSetup(false).catch(console.error);
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
