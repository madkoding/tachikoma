import type { Memory, GraphEdge } from '../api/client';
import type { GraphLink } from '../types/graph';

// ============================================================================
// Graph Data Transformation Utilities
// ============================================================================

/** Filter nodes by type and search query */
export function filterNodes(
  nodes: Memory[],
  filterType: string,
  searchQuery: string
): { filteredNodes: Memory[]; highlightIds: Set<string> } {
  let filteredNodes = nodes;
  
  if (filterType !== 'all') {
    filteredNodes = filteredNodes.filter(n => n.memory_type === filterType);
  }
  
  let highlightIds = new Set<string>();
  if (searchQuery) {
    const query = searchQuery.toLowerCase();
    filteredNodes = filteredNodes.filter(n => n.content.toLowerCase().includes(query));
    highlightIds = new Set(filteredNodes.map(n => n.id));
  }
  
  return { filteredNodes, highlightIds };
}

/** Convert edges to links and filter by available nodes */
export function createRealLinks(edges: GraphEdge[], nodeIds: Set<string>): GraphLink[] {
  return edges
    .filter(e => nodeIds.has(e.source) && nodeIds.has(e.target))
    .map(e => ({
      source: e.source,
      target: e.target,
      relation: e.relation,
      weight: e.weight,
    }));
}