import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow } from '@tauri-apps/api/window';
import type { DashboardData, Settings } from '@/types';

// Commands
export const getDashboardData = () => invoke<DashboardData>('get_dashboard_data');
export const removeSession = (projectDir: string) => invoke('remove_session', { projectDir });
export const clearAllSessions = () => invoke('clear_all_sessions');
export const getSettings = () => invoke<Settings>('get_settings');

// Window operations
export const getAppWindow = () => {
  try {
    return getCurrentWindow();
  } catch {
    return null;
  }
};

export const startDragging = async () => {
  const window = getAppWindow();
  if (window) {
    await window.startDragging();
  }
};

export const isFocused = async (): Promise<boolean> => {
  const window = getAppWindow();
  if (window) {
    return window.isFocused();
  }
  return document.hasFocus();
};

// Event listeners
export const onStateUpdated = (callback: (data: DashboardData) => void): Promise<UnlistenFn> => {
  return listen<DashboardData>('state-updated', (event) => callback(event.payload));
};

export const onSettingsUpdated = (callback: (settings: Settings) => void): Promise<UnlistenFn> => {
  return listen<Settings>('settings-updated', (event) => callback(event.payload));
};

export const onWindowFocus = (callback: () => void): Promise<UnlistenFn> => {
  return listen('tauri://focus', callback);
};

export const onWindowBlur = (callback: () => void): Promise<UnlistenFn> => {
  return listen('tauri://blur', callback);
};
