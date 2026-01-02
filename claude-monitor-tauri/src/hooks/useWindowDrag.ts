import { useEffect, useRef, type RefObject } from 'react';
import { startDragging } from '@/lib/tauri';

export const useWindowDrag = (containerRef: RefObject<HTMLElement | null>, isMiniView: boolean) => {
  // Use ref to track isMiniView without triggering effect re-runs
  const isMiniViewRef = useRef(isMiniView);
  isMiniViewRef.current = isMiniView;

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const handleMouseDown = async (e: MouseEvent) => {
      // Only enable dragging in mini view mode (check current value via ref)
      if (!isMiniViewRef.current) return;

      const target = e.target as HTMLElement;

      // Don't drag when clicking on buttons or interactive elements
      if (
        target.tagName === 'BUTTON' ||
        target.classList.contains('remove-btn') ||
        target.classList.contains('refresh-btn') ||
        target.closest('button') ||
        target.closest('.remove-btn')
      ) {
        return;
      }

      // Only left mouse button
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
  }, [containerRef]);
};
