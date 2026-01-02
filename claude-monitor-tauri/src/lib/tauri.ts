import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import { getCurrentWindow, getAllWindows } from '@tauri-apps/api/window';
import type { DashboardData, DiffType, GitInfo, Settings } from '@/types';

// Commands
export const getDashboardData = () => invoke<DashboardData>('get_dashboard_data');
export const removeSession = (projectDir: string) => invoke('remove_session', { projectDir });
export const clearAllSessions = () => invoke('clear_all_sessions');
export const getSettings = () => invoke<Settings>('get_settings');
export const getRepoGitInfo = (projectDir: string) =>
  invoke<GitInfo>('get_repo_git_info', { projectDir });

export type { DiffType };

export const openDiff = (projectDir: string, diffType: DiffType, baseBranch?: string) =>
  invoke('open_diff', { projectDir, diffType, baseBranch });

// Window operations
export const getAppWindow = () => {
  try {
    return getCurrentWindow();
  } catch {
    return null;
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

// Bring all diff windows to front
export const bringDiffWindowsToFront = async (): Promise<void> => {
  const windows = await getAllWindows();
  const diffWindows = windows.filter((w) => w.label.startsWith('difit-'));

  for (const window of diffWindows) {
    await window.show();
    await window.unminimize();
    await window.setFocus();
  }

  // Re-focus dashboard to keep it on top
  const dashboard = getCurrentWindow();
  await dashboard.setFocus();
};
