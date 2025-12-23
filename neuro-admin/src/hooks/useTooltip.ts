import { useState, useEffect, useRef, useCallback } from 'react';

interface TooltipState {
  text: string;
  x: number;
  y: number;
  alignment: 'left' | 'center' | 'right';
}

interface UseTooltipProps {
  graphRef: React.RefObject<any>;
  containerRef: React.RefObject<HTMLDivElement>;
}

export function useTooltip({ graphRef, containerRef }: UseTooltipProps) {
  const [tooltip, setTooltip] = useState<TooltipState | null>(null);
  const [displayedText, setDisplayedText] = useState('');
  const [tooltipFading, setTooltipFading] = useState(false);
  const tooltipTimeoutRef = useRef<ReturnType<typeof setTimeout>>();

  // Efecto typewriter
  useEffect(() => {
    if (!tooltip) {
      setDisplayedText('');
      return;
    }

    setDisplayedText('');
    let index = 0;
    const text = tooltip.text;

    const typeInterval = setInterval(() => {
      if (index < text.length) {
        setDisplayedText(text.substring(0, index + 1));
        index++;
      } else {
        clearInterval(typeInterval);
      }
    }, 15);

    return () => clearInterval(typeInterval);
  }, [tooltip]);

  const calculatePosition = useCallback(
    (node: { x?: number; y?: number; z?: number; content: string }) => {
      if (!graphRef.current || !containerRef.current) return null;

      const screenCoords = graphRef.current.graph2ScreenCoords(
        node.x || 0,
        node.y || 0,
        node.z || 0
      );
      const containerRect = containerRef.current.getBoundingClientRect();
      const tooltipWidth = 280;
      const tooltipHeight = 40;
      const padding = 15;

      let adjustedX = screenCoords.x;
      let alignment: 'left' | 'center' | 'right' = 'center';

      if (screenCoords.x < tooltipWidth / 2 + padding) {
        adjustedX = padding;
        alignment = 'left';
      } else if (screenCoords.x > containerRect.width - tooltipWidth / 2 - padding) {
        adjustedX = containerRect.width - padding;
        alignment = 'right';
      }

      let adjustedY = screenCoords.y - 30;
      if (adjustedY - tooltipHeight < padding) {
        adjustedY = screenCoords.y + 50;
      }
      if (adjustedY > containerRect.height - padding) {
        adjustedY = containerRect.height - padding;
      }

      return {
        text: node.content.substring(0, 100) + (node.content.length > 100 ? '...' : ''),
        x: adjustedX,
        y: adjustedY,
        alignment,
      };
    },
    [graphRef, containerRef]
  );

  const showTooltip = useCallback(
    (node: { x?: number; y?: number; z?: number; content: string } | null) => {
      if (tooltipTimeoutRef.current) {
        clearTimeout(tooltipTimeoutRef.current);
      }

      if (!node || !containerRef.current) {
        tooltipTimeoutRef.current = setTimeout(() => {
          setTooltipFading(true);
          setTimeout(() => {
            setTooltip(null);
            setTooltipFading(false);
          }, 1000);
        }, 3000);
        return;
      }

      const newText = node.content.substring(0, 100) + (node.content.length > 100 ? '...' : '');
      const isNewNode = tooltip?.text !== newText;

      if (isNewNode && tooltip) {
        setTooltipFading(true);
        setTimeout(() => {
          const newPosition = calculatePosition(node);
          if (newPosition) {
            setTooltip(newPosition);
            setTooltipFading(false);
          }
        }, 300);
      } else if (!tooltip) {
        const newPosition = calculatePosition(node);
        if (newPosition) {
          setTooltip(newPosition);
          setTooltipFading(false);
        }
      }
    },
    [tooltip, calculatePosition, containerRef]
  );

  const hideTooltip = useCallback(() => {
    if (tooltipTimeoutRef.current) {
      clearTimeout(tooltipTimeoutRef.current);
    }
    setTooltip(null);
    setTooltipFading(false);
  }, []);

  useEffect(() => {
    return () => {
      if (tooltipTimeoutRef.current) {
        clearTimeout(tooltipTimeoutRef.current);
      }
    };
  }, []);

  return {
    tooltip,
    displayedText,
    tooltipFading,
    showTooltip,
    hideTooltip,
  };
}
