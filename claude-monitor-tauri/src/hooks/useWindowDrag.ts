import { useEffect, type RefObject } from 'react';
import { startDragging } from '@/lib/tauri';

export const useWindowDrag = (containerRef: RefObject<HTMLElement | null>, isMiniView: boolean) => {
  useEffect(() => {
    const container = containerRef.current;
    if (!container || !isMiniView) return;

    const handleMouseDown = async (e: MouseEvent) => {
      const target = e.target as HTMLElement;

      // Don't drag when clicking interactive elements
      if (target.tagName === 'BUTTON' || target.closest('button')) {
        return;
      }

      if (e.buttons === 1) {
        try {
          await startDragging();
        } catch (error) {
          console.error('Failed to start dragging:', error);
        }
      }
    };

    container.addEventListener('mousedown', handleMouseDown);
    return () => container.removeEventListener('mousedown', handleMouseDown);
  }, [containerRef, isMiniView]);
};
