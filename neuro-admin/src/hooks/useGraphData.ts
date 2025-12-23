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
 * Genera un hash del contenido de un nodo para detectar cambios
 */
function hashNodeContent(node: { content: string; memory_type: string; importance_score: number }): string {
  return `${node.content}|${node.memory_type}|${node.importance_score}`;
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

// Store birth times outside of React to persist across renders
const nodeBirthTimes = new Map<string, number>();

// Store update times for update animation effects
const nodeUpdateTimes = new Map<string, number>();

// Store content hashes to detect real changes
const nodeContentHashes = new Map<string, string>();

// Track if we've completed the initial load (to skip birth animation)
let hasCompletedInitialLoad = false;

// Cache node objects to preserve references between updates
const nodeObjectCache = new Map<string, GraphNode>();

// STABLE ARRAY REFERENCES - mutate these instead of creating new arrays
// This prevents ForceGraph3D from re-creating Three.js objects
const stableNodesArray: GraphNode[] = [];
const stableLinksArray: GraphLink[] = [];

export function useGraphData({ graphData, filterType, searchQuery }: UseGraphDataProps) {
  const { nodes, links, currentHighlightedIds } = useMemo(() => {
    if (!graphData) {
      // Clear arrays but keep references
      stableNodesArray.length = 0;
      stableLinksArray.length = 0;
      return { nodes: stableNodesArray, links: stableLinksArray, currentHighlightedIds: new Set<string>() };
    }

    const { filteredNodes, highlightIds } = filterNodes(
      graphData.nodes,
      filterType,
      searchQuery
    );

    const nodeIds = new Set(filteredNodes.map(n => n.id));
    const realLinks = createRealLinks(graphData.edges, nodeIds);
    const virtualLinks = buildVirtualLinks(filteredNodes, realLinks);

    const now = Date.now();

    // On first real data load, mark ALL nodes from graphData (not just filtered)
    // so they don't get birth animation on page refresh
    if (!hasCompletedInitialLoad && graphData.nodes.length > 0) {
      graphData.nodes.forEach(n => {
        // Set birth time far in the past so animation doesn't trigger
        nodeBirthTimes.set(n.id, 0);
      });
      hasCompletedInitialLoad = true;
    }

    // Build a set of current node IDs for quick lookup
    const currentNodeIds = new Set(filteredNodes.map(n => n.id));
    
    // Remove nodes that no longer exist from the stable array
    for (let i = stableNodesArray.length - 1; i >= 0; i--) {
      if (!currentNodeIds.has(stableNodesArray[i].id)) {
        stableNodesArray.splice(i, 1);
      }
    }
    
    // Build set of existing IDs in stable array
    const existingIds = new Set(stableNodesArray.map(n => n.id));

    // Update existing nodes and add new ones
    filteredNodes.forEach((n) => {
      const pos = generateNodePosition(n.id);

      // Track birth time for new nodes (for birth animation)
      // Only set current time for truly NEW nodes (not from initial load)
      if (!nodeBirthTimes.has(n.id)) {
        nodeBirthTimes.set(n.id, now);
      }
      const birthTime = nodeBirthTimes.get(n.id)!;
      
      // Always ensure we have a content hash for the node
      const currentHash = hashNodeContent(n);
      if (!nodeContentHashes.has(n.id)) {
        // First time seeing this node's content - store hash without triggering animation
        nodeContentHashes.set(n.id, currentHash);
      }

      // Check if we have a cached node object
      const cachedNode = nodeObjectCache.get(n.id);
      if (cachedNode) {
        // Check if content has actually changed (for update animation)
        const previousHash = nodeContentHashes.get(n.id);
        
        console.log('[useGraphData] Checking node:', n.id.substring(0, 8), 
          'prevHash:', previousHash?.substring(0, 30), 
          'currHash:', currentHash.substring(0, 30),
          'changed:', previousHash !== currentHash);
        
        if (previousHash && currentHash !== previousHash) {
          // Content changed! Set update time for animation
          const updateTime = now;
          nodeUpdateTimes.set(n.id, updateTime);
          nodeContentHashes.set(n.id, currentHash);
          // Update the cached node's update time
          cachedNode.__updateTime = updateTime;
          console.log('[useGraphData] 🎆 Node updated, triggering animation:', n.id, 'updateTime:', updateTime);
        }
        
        // Update mutable properties on the existing object
        cachedNode.__highlighted = highlightIds.has(n.id);
        // Update data properties that might have changed
        cachedNode.content = n.content;
        cachedNode.memory_type = n.memory_type;
        cachedNode.importance_score = n.importance_score;
        cachedNode.access_count = n.access_count;
        cachedNode.metadata = n.metadata;
        cachedNode.created_at = n.created_at;
        cachedNode.updated_at = n.updated_at;
        
        // If not already in stable array, add it
        if (!existingIds.has(n.id)) {
          stableNodesArray.push(cachedNode);
        }
        return;
      }

      // Create new node object and cache it
      const newNode = {
        ...n,
        ...pos,
        fx: pos.x,
        fy: pos.y,
        fz: pos.z,
        __highlighted: highlightIds.has(n.id),
        __birthTime: birthTime,
      } as GraphNode;
      
      nodeObjectCache.set(n.id, newNode);
      stableNodesArray.push(newNode);
    });

    // Update links intelligently - preserve ForceGraph3D's internal node references
    const allNewLinks = [...realLinks, ...virtualLinks];
    
    // Create a key for each link to identify it
    const getLinkKey = (link: GraphLink): string => {
      const srcId = typeof link.source === 'string' ? link.source : (link.source as any).id;
      const tgtId = typeof link.target === 'string' ? link.target : (link.target as any).id;
      return `${srcId}::${tgtId}::${link.relation}`;
    };
    
    // Build set of new link keys
    const newLinkKeys = new Set(allNewLinks.map(getLinkKey));
    
    // Remove links that no longer exist
    for (let i = stableLinksArray.length - 1; i >= 0; i--) {
      const key = getLinkKey(stableLinksArray[i]);
      if (!newLinkKeys.has(key)) {
        stableLinksArray.splice(i, 1);
      }
    }
    
    // Build set of existing link keys
    const existingLinkKeys = new Set(stableLinksArray.map(getLinkKey));
    
    // Add new links that don't exist yet, using node object references
    for (const link of allNewLinks) {
      const key = getLinkKey(link);
      if (!existingLinkKeys.has(key)) {
        // Get the source and target IDs
        const srcId = typeof link.source === 'string' ? link.source : (link.source as any).id;
        const tgtId = typeof link.target === 'string' ? link.target : (link.target as any).id;
        
        // Get node object references from cache
        const srcNode = nodeObjectCache.get(srcId);
        const tgtNode = nodeObjectCache.get(tgtId);
        
        // Only add link if both nodes exist in cache
        if (srcNode && tgtNode) {
          stableLinksArray.push({
            source: srcNode,
            target: tgtNode,
            relation: link.relation,
            weight: link.weight,
          } as GraphLink);
        }
      }
    }

    return {
      nodes: stableNodesArray,
      links: stableLinksArray,
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
