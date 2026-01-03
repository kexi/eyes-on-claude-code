import { useEffect } from 'react';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';

export const useWindowOpacity = (activeOpacity: number, inactiveOpacity: number) => {
  useEffect(() => {
    const applyOpacity = (active: boolean) => {
      const opacity = active ? activeOpacity : inactiveOpacity;
      document.body.style.setProperty('opacity', String(opacity), 'important');
    };

    let unlisten: UnlistenFn | undefined;

    // Listen for custom dashboard-active event from Rust backend
    // This event is emitted when:
    // - Dashboard gains/loses focus
    // - A difit window gains focus (dashboard becomes inactive)
    listen<boolean>('dashboard-active', (event) => {
      applyOpacity(event.payload);
    }).then((u) => {
      unlisten = u;
    });

    // Set initial opacity based on current focus state
    getCurrentWindow()
      .isFocused()
      .then((focused) => applyOpacity(focused))
      .catch(console.error);

    return () => {
      unlisten?.();
    };
  }, [activeOpacity, inactiveOpacity]);
};
