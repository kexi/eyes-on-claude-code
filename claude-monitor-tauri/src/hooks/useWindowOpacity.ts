import { useEffect } from 'react';
import { onWindowFocus, onWindowBlur, isFocused } from '@/lib/tauri';

export const useWindowOpacity = (activeOpacity: number, inactiveOpacity: number) => {
  useEffect(() => {
    const applyOpacity = (focused: boolean) => {
      const opacity = focused ? activeOpacity : inactiveOpacity;
      document.body.style.setProperty('opacity', String(opacity), 'important');
    };

    const unlisteners: Array<() => void> = [];

    onWindowFocus(() => applyOpacity(true)).then((u) => unlisteners.push(u));
    onWindowBlur(() => applyOpacity(false)).then((u) => unlisteners.push(u));

    // Initial state
    isFocused().then((focused) => applyOpacity(focused));

    return () => {
      unlisteners.forEach((u) => u());
    };
  }, [activeOpacity, inactiveOpacity]);
};
