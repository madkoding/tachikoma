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
  const typeKeys = Object.keys(nodesByType).sort((a, b) => a.localeCompare(b)); // Ordenar para determinismo
  
  for (let i = 0; i < typeKeys.length; i++) {
    for (let j = i + 1; j < typeKeys.length; j++) {
      const nodesA = nodesByType[typeKeys[i]];
      const nodesB = nodesByType[typeKeys[j]];
      
      if (nodesA.length > 0 && nodesB.length > 0) {
        // Usar el primer nodo ordenado por ID para determinismo
        const sortedA = [...nodesA].sort((a, b) => a.id.localeCompare(b.id));
        const sortedB = [...nodesB].sort((a, b) => a.id.localeCompare(b.id));
        
        tryAddLink(
          links,
          existingConnections,
          sortedA[0].id,
          sortedB[0].id,
          'RelatedTo',
          0.2
        );
      }
    }
  }
  
  return links;
}

/**
 * Hash string para generar un número determinístico
 */
function hashString(str: string): number {
  let hash = 5381;
  for (let i = 0; i < str.length; i++) {
    hash = ((hash << 5) + hash) ^ (str.codePointAt(i) ?? 0);
  }
  return hash >>> 0;
}

/**
 * Extrae palabras clave del contenido (palabras de 4+ caracteres)
 */
function extractKeywords(content: string): Set<string> {
  const words = content.toLowerCase()
    .replaceAll(/[^a-záéíóúüñ\s]/gi, ' ')
    .split(/\s+/)
    .filter(w => w.length >= 4);
  return new Set(words);
}

/**
 * Calcula similitud de Jaccard entre dos conjuntos de palabras
 */
function jaccardSimilarity(setA: Set<string>, setB: Set<string>): number {
  if (setA.size === 0 || setB.size === 0) return 0;
  
  let intersection = 0;
  for (const word of setA) {
    if (setB.has(word)) intersection++;
  }
  
  const union = setA.size + setB.size - intersection;
  return union > 0 ? intersection / union : 0;
}

/**
 * Crea conexiones basadas en similitud de contenido
 * Determinístico: mismos nodos = mismas conexiones
 */
function createContentBasedLinks(
  nodes: Memory[],
  existingConnections: Set<string>,
  maxConnections: number = 50
): GraphLink[] {
  const links: GraphLink[] = [];
  
  if (nodes.length < 2) return links;
  
  // Pre-calcular keywords para cada nodo
  const nodeKeywords = new Map<string, Set<string>>();
  for (const node of nodes) {
    nodeKeywords.set(node.id, extractKeywords(node.content));
  }
  
  // Calcular todas las similitudes entre pares
  const similarities: Array<{ i: number; j: number; score: number }> = [];
  
  // Ordenar nodos por ID para determinismo
  const sortedNodes = [...nodes].sort((a, b) => a.id.localeCompare(b.id));
  
  for (let i = 0; i < sortedNodes.length; i++) {
    for (let j = i + 1; j < sortedNodes.length; j++) {
      const keywordsA = nodeKeywords.get(sortedNodes[i].id)!;
      const keywordsB = nodeKeywords.get(sortedNodes[j].id)!;
      
      const similarity = jaccardSimilarity(keywordsA, keywordsB);
      
      // Solo considerar pares con alguna similitud
      if (similarity > 0.05) {
        similarities.push({ i, j, score: similarity });
      }
    }
  }
  
  // Ordenar por similitud descendente (determinístico por score + IDs)
  similarities.sort((a, b) => {
    if (b.score !== a.score) return b.score - a.score;
    // Desempate por hash de IDs combinados
    const hashA = hashString(sortedNodes[a.i].id + sortedNodes[a.j].id);
    const hashB = hashString(sortedNodes[b.i].id + sortedNodes[b.j].id);
    return hashA - hashB;
  });
  
  // Tomar las mejores conexiones hasta el límite
  let created = 0;
  for (const { i, j, score } of similarities) {
    if (created >= maxConnections) break;
    
    const added = tryAddLink(
      links,
      existingConnections,
      sortedNodes[i].id,
      sortedNodes[j].id,
      'SimilarContent',
      Math.min(0.5, score * 2) // Peso basado en similitud
    );
    if (added) created++;
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
  
  // Build virtual links in layers (todos determinísticos)
  const chainLinks = createTypeChainLinks(nodesByType, existingConnections);
  const hubLinks = createHubLinks(nodesByType, existingConnections);
  const contentLinks = createContentBasedLinks(nodes, existingConnections);
  
  return [...chainLinks, ...hubLinks, ...contentLinks];
}
