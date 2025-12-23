import { useState, useEffect, useCallback } from 'react';

interface UseDimensionsProps {
  containerRef: React.RefObject<HTMLDivElement>;
}

export function useDimensions({ containerRef }: UseDimensionsProps) {
  const [dimensions, setDimensions] = useState<{ width: number; height: number } | null>(null);

  const shouldUpdateDimensions = useCallback(
    (prev: { width: number; height: number } | null, newDims: { width: number; height: number }) => {
      if (!prev) return newDims;
      const hasChanged =
        Math.abs(prev.width - newDims.width) > 1 || Math.abs(prev.height - newDims.height) > 1;
      return hasChanged ? newDims : prev;
    },
    []
  );

  useEffect(() => {
    let rafId: number;
    const timeoutIds: ReturnType<typeof setTimeout>[] = [];
    let mounted = true;

    const calculateDimensions = () => {
      if (!containerRef.current) return null;
      const width = containerRef.current.offsetWidth;
      const height = containerRef.current.offsetHeight;
      if (width < 100 || height < 100) return null;
      return { width, height };
    };

    const applyDimensions = (newDims: { width: number; height: number }) => {
      setDimensions(prev => shouldUpdateDimensions(prev, newDims));
    };

    const updateDimensions = () => {
      if (!mounted) return;
      if (rafId) cancelAnimationFrame(rafId);

      rafId = requestAnimationFrame(() => {
        if (!mounted) return;
        const newDims = calculateDimensions();
        if (newDims) applyDimensions(newDims);
      });
    };

    const scheduleUpdates = () => {
      updateDimensions();
      [50, 100, 200, 300, 500, 750, 1000, 1500, 2000].forEach(delay => {
        timeoutIds.push(setTimeout(updateDimensions, delay));
      });
    };

    if (document.readyState === 'complete') {
      scheduleUpdates();
    } else {
      window.addEventListener('load', scheduleUpdates, { once: true });
    }

    const resizeObserver = new ResizeObserver(updateDimensions);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    window.addEventListener('resize', updateDimensions);

    return () => {
      mounted = false;
      if (rafId) cancelAnimationFrame(rafId);
      timeoutIds.forEach(id => clearTimeout(id));
      resizeObserver.disconnect();
      window.removeEventListener('resize', updateDimensions);
    };
  }, [containerRef, shouldUpdateDimensions]);

  return dimensions;
}
