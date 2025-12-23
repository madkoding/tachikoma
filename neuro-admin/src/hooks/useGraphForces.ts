import { useEffect, useCallback } from 'react';
import { GRAPH_CONFIG } from '../constants/graphConfig';

export function useGraphForces(graphRef: React.RefObject<any>, graphData: any) {
  const applyForces = useCallback(() => {
    const fg = graphRef.current;
    if (!fg) return;

    const { link, charge, center } = GRAPH_CONFIG.forces;

    const linkForce = fg.d3Force('link');
    if (linkForce) {
      linkForce.distance(() => link.distance).strength(() => link.strength);
    }

    const chargeForce = fg.d3Force('charge');
    if (chargeForce) {
      chargeForce.strength(charge.strength).distanceMax(charge.distanceMax);
    }

    const centerForce = fg.d3Force('center');
    if (centerForce) {
      centerForce.strength(center.strength);
    }

    fg.d3ReheatSimulation();
  }, [graphRef]);

  useEffect(() => {
    if (!graphData) return;

    applyForces();
    const timer = setTimeout(applyForces, 50);

    return () => clearTimeout(timer);
  }, [graphData, applyForces]);

  return { applyForces };
}
