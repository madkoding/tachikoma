import { useState, useCallback } from 'react';

export function useHoverState() {
  const [hoveredNodeId, setHoveredNodeId] = useState<string | null>(null);

  const handleNodeHover = useCallback((node: { id: string } | null) => {
    setHoveredNodeId(node?.id ?? null);
  }, []);

  return {
    hoveredNodeId,
    handleNodeHover,
  };
}
