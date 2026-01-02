import { useRef, useEffect } from 'react';
import { AppProvider, useAppContext } from '@/context/AppContext';
import { useWindowOpacity } from '@/hooks/useWindowOpacity';
import { useWindowDrag } from '@/hooks/useWindowDrag';
import { Header } from '@/components/Header';
import { StatsGrid } from '@/components/StatsGrid';
import { SessionList } from '@/components/SessionList';
import { EventList } from '@/components/EventList';

const Dashboard = () => {
  const { dashboardData, settings, isLoading, refreshData } = useAppContext();
  const containerRef = useRef<HTMLDivElement>(null);
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
  useWindowDrag(containerRef, isMiniView);

  if (isLoading) {
    return (
      <div className="container bg-bg-primary h-screen rounded-xl max-w-[900px] mx-auto p-5 flex items-center justify-center">
        <div className="text-text-secondary">Loading...</div>
      </div>
    );
  }

  return (
    <div
      ref={containerRef}
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
  return (
    <AppProvider>
      <Dashboard />
    </AppProvider>
  );
}

export default App;
