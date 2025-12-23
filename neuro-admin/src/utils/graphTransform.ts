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

/** Group nodes by their memory type */
function groupNodesByType(nodes: Memory[]): Record<string, Memory[]> {
  const nodesByType: Record<string, Memory[]> = {};
  for (const node of nodes) {
    if (!nodesByType[node.memory_type]) {
      nodesByType[node.memory_type] = [];
    }
    nodesByType[node.memory_type].push(node);
  }
  return nodesByType;
}

/** Create a connection key for tracking existing connections */
function getConnectionKey(idA: string, idB: string): string {
  return `${idA}-${idB}`;
}

/** Add a link if it doesn't already exist */
function tryAddLink(
  links: GraphLink[],
  existingConnections: Set<string>,
  sourceId: string,
  targetId: string,
  relation: string,
  weight: number
): boolean {
  const forwardKey = getConnectionKey(sourceId, targetId);
  const reverseKey = getConnectionKey(targetId, sourceId);
  
  if (!existingConnections.has(forwardKey) && !existingConnections.has(reverseKey)) {
    links.push({ source: sourceId, target: targetId, relation, weight });
    existingConnections.add(forwardKey);
    return true;
  }
  return false;
}

/** Create chain connections between nodes of the same type */
function createTypeChainLinks(
  nodesByType: Record<string, Memory[]>,
  existingConnections: Set<string>
): GraphLink[] {
  const links: GraphLink[] = [];
  
  for (const nodesOfType of Object.values(nodesByType)) {
    for (let i = 0; i < nodesOfType.length - 1; i++) {
      tryAddLink(
        links,
        existingConnections,
        nodesOfType[i].id,
        nodesOfType[i + 1].id,
        'SimilarTo',
        0.3
      );
    }
  }
  
  return links;
}

/** Create hub connections between different type groups */
function createHubLinks(
  nodesByType: Record<string, Memory[]>,
  existingConnections: Set<string>
): GraphLink[] {
  const links: GraphLink[] = [];
  const typeKeys = Object.keys(nodesByType);
  
  for (let i = 0; i < typeKeys.length; i++) {
    for (let j = i + 1; j < typeKeys.length; j++) {
      const nodesA = nodesByType[typeKeys[i]];
      const nodesB = nodesByType[typeKeys[j]];
      
      if (nodesA.length > 0 && nodesB.length > 0) {
        tryAddLink(
          links,
          existingConnections,
          nodesA[0].id,
          nodesB[0].id,
          'RelatedTo',
          0.2
        );
      }
    }
  }
  
  return links;
}

/** Create random connections for organic appearance */
function createRandomLinks(
  nodes: Memory[],
  existingConnections: Set<string>,
  maxConnections: number = 50
): GraphLink[] {
  const links: GraphLink[] = [];
  const targetCount = Math.min(nodes.length * 2, maxConnections);
  let created = 0;
  
  for (let attempts = 0; attempts < targetCount * 3 && created < targetCount; attempts++) {
    const i = Math.floor(Math.random() * nodes.length);
    const j = Math.floor(Math.random() * nodes.length);
    
    if (i !== j) {
      const added = tryAddLink(
        links,
        existingConnections,
        nodes[i].id,
        nodes[j].id,
        'RelatedTo',
        0.15
      );
      if (added) created++;
    }
  }
  
  return links;
}

/** Build the complete set of virtual links for the graph */
export function buildVirtualLinks(
  nodes: Memory[],
  realLinks: GraphLink[]
): GraphLink[] {
  // Track existing connections
  const existingConnections = new Set(
    realLinks.map(l => 
      getConnectionKey(
        typeof l.source === 'string' ? l.source : l.source.id,
        typeof l.target === 'string' ? l.target : l.target.id
      )
    )
  );
  
  const nodesByType = groupNodesByType(nodes);
  
  // Build virtual links in layers
  const chainLinks = createTypeChainLinks(nodesByType, existingConnections);
  const hubLinks = createHubLinks(nodesByType, existingConnections);
  const randomLinks = createRandomLinks(nodes, existingConnections);
  
  return [...chainLinks, ...hubLinks, ...randomLinks];
}
