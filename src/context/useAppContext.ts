import { useContext } from 'react';
import { AppContext } from './appContextStore';
import type { AppContextValue } from './appContextStore';

export const useAppContext = (): AppContextValue => {
  const context = useContext(AppContext);
  if (!context) throw new Error('useAppContext must be used within AppProvider');
  return context;
};
