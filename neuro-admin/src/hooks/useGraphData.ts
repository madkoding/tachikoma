import { useMemo } from 'react';
import { filterNodes, createRealLinks, buildVirtualLinks } from '../utils/graphTransform';
import { GRAPH_CONFIG } from '../constants/graphConfig';
import type { GraphNode, GraphLink } from '../types/graph';
import type { Memory } from '../api/client';

interface GraphDataResponse {
  nodes: Memory[];
  edges: { source: string; target: string; relation: string; weight: number }[];
}

interface UseGraphDataProps {
  graphData: GraphDataResponse | undefined;
  filterType: string;
  searchQuery: string;
}

/**
 * Genera un hash numérico determinístico a partir de un string (ID del nodo)
 * Usa el algoritmo djb2 para producir valores consistentes
 */
function hashString(str: string): number {
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash) ^ (str.codePointAt(i) ?? 0);
  }
  return hash >>> 0; // Convertir a unsigned 32-bit
}

/**
 * Genera un número pseudo-aleatorio determinístico entre 0 y 1
 * basado en una semilla (seed)
 */
function seededRandom(seed: number): number {
  const x = Math.sin(seed) * 10000;
  return x - Math.floor(x);
}

/**
 * Genera posición 3D determinística basada en el ID del nodo
 * Cada nodo siempre estará en la misma posición
 */
function generateNodePosition(nodeId: string) {
  const { min, max } = GRAPH_CONFIG.nodes.radius;
  const hash = hashString(nodeId);
  
  // Usar diferentes partes del hash para cada eje
  const seed1 = seededRandom(hash);
  const seed2 = seededRandom(hash * 2);
  const seed3 = seededRandom(hash * 3);
  
  // Distribución esférica usando coordenadas esféricas
  const theta = seed1 * Math.PI * 2;        // Ángulo horizontal (0 a 2π)
  const phi = Math.acos(2 * seed2 - 1);     // Ángulo vertical (distribución uniforme en esfera)
  const radius = min + seed3 * (max - min); // Radio dentro del rango

  return {
    x: radius * Math.sin(phi) * Math.cos(theta),
    y: radius * Math.sin(phi) * Math.sin(theta),
    z: radius * Math.cos(phi),
  };
}

export function useGraphData({ graphData, filterType, searchQuery }: UseGraphDataProps) {
  const { nodes, links, currentHighlightedIds } = useMemo(() => {
    if (!graphData) {
      return { nodes: [], links: [], currentHighlightedIds: new Set<string>() };
    }

    const { filteredNodes, highlightIds } = filterNodes(
      graphData.nodes,
      filterType,
      searchQuery
    );

    const nodeIds = new Set(filteredNodes.map(n => n.id));
    const realLinks = createRealLinks(graphData.edges, nodeIds);
    const virtualLinks = buildVirtualLinks(filteredNodes, realLinks);

    // Asignar posiciones iniciales dispersas en un volumen 3D esférico
    // Cada nodo siempre tendrá la misma posición basada en su ID
    const nodesWithPositions = filteredNodes.map((n) => {
      const pos = generateNodePosition(n.id);

      return {
        ...n,
        ...pos,
        fx: pos.x,
        fy: pos.y,
        fz: pos.z,
        __highlighted: highlightIds.has(n.id),
      };
    });

    return {
      nodes: nodesWithPositions as GraphNode[],
      links: [...realLinks, ...virtualLinks] as GraphLink[],
      currentHighlightedIds: highlightIds,
    };
  }, [graphData, filterType, searchQuery]);

  const memoryTypes = useMemo(() => {
    if (!graphData) return [];
    const types = new Set(graphData.nodes.map(n => n.memory_type));
    return Array.from(types);
  }, [graphData]);

  return {
    nodes,
    links,
    currentHighlightedIds,
    memoryTypes,
  };
}
