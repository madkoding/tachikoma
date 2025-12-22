import { useRef, useCallback, useState, useMemo, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import ForceGraph3D from 'react-force-graph-3d';
import { graphApi, type Memory, type GraphEdge } from '../api/client';

interface GraphNode extends Memory {
  x?: number;
  y?: number;
  z?: number;
  vx?: number;
  vy?: number;
  vz?: number;
}

interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  relation: string;
  weight: number;
}

const NODE_COLORS: Record<string, string> = {
  fact: '#00f5ff',      // cyber-cyan
  preference: '#00ff88', // cyber-green
  context: '#f59e0b',
  conversation: '#ff00ff', // cyber-magenta
  task: '#ef4444',
  entity: '#ec4899',
  default: '#6b7280',
};

const RELATION_COLORS: Record<string, string> = {
  RelatedTo: '#94a3b8',
  Causes: '#ef4444',
  PartOf: '#22c55e',
  HasProperty: '#f59e0b',
  UsedFor: '#3b82f6',
  CapableOf: '#8b5cf6',
  AtLocation: '#ec4899',
  CreatedBy: '#14b8a6',
  DerivedFrom: '#f97316',
  SimilarTo: '#6366f1',
  ContradictsWith: '#dc2626',
};

export default function GraphPage() {
  const { t } = useTranslation();
  const graphRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [dimensions, setDimensions] = useState({ width: 800, height: 600 });

  // Update dimensions on resize
  useEffect(() => {
    const updateDimensions = () => {
      if (containerRef.current) {
        setDimensions({
          width: containerRef.current.clientWidth,
          height: containerRef.current.clientHeight,
        });
      }
    };
    updateDimensions();
    globalThis.addEventListener('resize', updateDimensions);
    return () => globalThis.removeEventListener('resize', updateDimensions);
  }, []);

  const { data: graphData, isLoading } = useQuery({
    queryKey: ['graph-data'],
    queryFn: () => graphApi.getGraph(500),
    refetchInterval: 60000,
  });

  // Transform data for force graph
  const { nodes, links } = useMemo(() => {
    if (!graphData) return { nodes: [], links: [] };

    let filteredNodes = graphData.nodes;
    
    // Filter by type
    if (filterType !== 'all') {
      filteredNodes = filteredNodes.filter(n => n.memory_type === filterType);
    }
    
    // Filter by search
    if (searchQuery) {
      const query = searchQuery.toLowerCase();
      filteredNodes = filteredNodes.filter(n => 
        n.content.toLowerCase().includes(query)
      );
    }

    const nodeIds = new Set(filteredNodes.map(n => n.id));
    
    const filteredEdges = graphData.edges.filter(
      e => nodeIds.has(e.source) && nodeIds.has(e.target)
    );

    return {
      nodes: filteredNodes as GraphNode[],
      links: filteredEdges.map((e: GraphEdge) => ({
        source: e.source,
        target: e.target,
        relation: e.relation,
        weight: e.weight,
      })) as GraphLink[],
    };
  }, [graphData, filterType, searchQuery]);

  const memoryTypes = useMemo(() => {
    if (!graphData) return [];
    const types = new Set(graphData.nodes.map(n => n.memory_type));
    return Array.from(types);
  }, [graphData]);

  const handleNodeClick = useCallback((node: GraphNode) => {
    setSelectedNode(node);
    if (graphRef.current) {
      // Focus on node with smooth camera transition
      const distance = 100;
      const distRatio = 1 + distance / Math.hypot(node.x || 0, node.y || 0, node.z || 0);
      graphRef.current.cameraPosition(
        { x: (node.x || 0) * distRatio, y: (node.y || 0) * distRatio, z: (node.z || 0) * distRatio },
        node,
        2000
      );
    }
  }, []);

  const handleResetView = () => {
    if (graphRef.current) {
      graphRef.current.cameraPosition({ x: 0, y: 0, z: 300 }, { x: 0, y: 0, z: 0 }, 1000);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-8rem)]">
        <div className="cyber-spinner"></div>
      </div>
    );
  }

  return (
    <div className="space-y-4 h-[calc(100vh-8rem)]">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-2xl font-bold neon-cyan font-cyber tracking-wider">
            {t('graph.title')}
          </h1>
          <p className="text-cyber-cyan/60 font-mono text-sm mt-1">
            {t('graph.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-sm text-cyber-cyan/70 font-mono">
            {t('graph.nodeCount')}: <span className="neon-green">{nodes.length}</span> | {t('graph.edgeCount')}: <span className="neon-magenta">{links.length}</span>
          </span>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-wrap items-center gap-4 cyber-card p-4">
        {/* Search */}
        <div className="flex-1 min-w-[200px]">
          <input
            type="text"
            placeholder={t('graph.search')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="cyber-input w-full"
          />
        </div>

        {/* Filter */}
        <div>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="cyber-input"
          >
            <option value="all">{t('graph.allTypes')}</option>
            {memoryTypes.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
        </div>

        {/* Reset View */}
        <button
          onClick={handleResetView}
          className="cyber-button px-4 py-2"
          title={t('graph.resetView')}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
          </svg>
        </button>
      </div>

      {/* Graph Container */}
      <div className="flex gap-4 h-[calc(100%-12rem)]">
        {/* Graph */}
        <div 
          ref={containerRef}
          className="flex-1 cyber-card overflow-hidden relative"
          style={{ background: 'radial-gradient(ellipse at center, #0a1628 0%, #050d18 100%)' }}
        >
          {/* Grid overlay effect */}
          <div className="absolute inset-0 opacity-10 pointer-events-none"
               style={{ 
                 backgroundImage: 'linear-gradient(rgba(0,245,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(0,245,255,0.1) 1px, transparent 1px)',
                 backgroundSize: '50px 50px'
               }} 
          />
          
          <ForceGraph3D
            ref={graphRef}
            width={dimensions.width}
            height={dimensions.height}
            graphData={{ nodes, links }}
            nodeId="id"
            nodeLabel={(node: GraphNode) => `${node.memory_type}: ${node.content.substring(0, 50)}...`}
            nodeColor={(node: GraphNode) => NODE_COLORS[node.memory_type] || NODE_COLORS.default}
            nodeRelSize={4}
            nodeVal={(node: GraphNode) => (node.importance_score || 0.5) * 3}
            nodeOpacity={0.9}
            linkColor={(link: GraphLink) => RELATION_COLORS[link.relation] || '#94a3b8'}
            linkWidth={(link: GraphLink) => link.weight * 1.5}
            linkOpacity={0.6}
            linkDirectionalArrowLength={3}
            linkDirectionalArrowRelPos={1}
            onNodeClick={handleNodeClick}
            backgroundColor="rgba(0,0,0,0)"
            showNavInfo={false}
            enableNodeDrag={true}
            enableNavigationControls={true}
            controlType="orbit"
          />
        </div>

        {/* Selected Node Panel */}
        {selectedNode && (
          <div className="w-80 cyber-card p-4 overflow-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold neon-cyan font-mono">{t('graph.nodeDetails')}</h3>
              <button
                onClick={() => setSelectedNode(null)}
                className="p-1 rounded hover:bg-cyber-cyan/10 text-cyber-cyan"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="space-y-3">
              <div>
                <span className="text-xs text-cyber-cyan/50 font-mono">{t('memories.type')}</span>
                <p className="text-sm font-medium text-cyber-cyan">
                  <span
                    className="inline-block w-3 h-3 rounded-full mr-2 shadow-[0_0_10px_currentColor]"
                    style={{ backgroundColor: NODE_COLORS[selectedNode.memory_type] || NODE_COLORS.default }}
                  />
                  {selectedNode.memory_type}
                </p>
              </div>
              
              <div>
                <span className="text-xs text-cyber-cyan/50 font-mono">{t('graph.content')}</span>
                <p className="text-sm text-cyber-cyan/80">{selectedNode.content}</p>
              </div>
              
              <div>
                <span className="text-xs text-cyber-cyan/50 font-mono">{t('graph.importance')}</span>
                <p className="text-sm neon-green">{(selectedNode.importance_score || 0).toFixed(2)}</p>
              </div>
              
              <div>
                <span className="text-xs text-cyber-cyan/50 font-mono">{t('graph.created')}</span>
                <p className="text-sm text-cyber-cyan/70">
                  {new Date(selectedNode.created_at).toLocaleString()}
                </p>
              </div>

              <div>
                <span className="text-xs text-cyber-cyan/50 font-mono">ID</span>
                <p className="text-xs text-cyber-cyan/40 font-mono break-all">
                  {selectedNode.id}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 cyber-card p-4">
        <div className="flex flex-wrap gap-3">
          <span className="text-sm font-medium text-cyber-cyan font-mono">{t('graph.nodeTypes')}:</span>
          {Object.entries(NODE_COLORS).filter(([k]) => k !== 'default').map(([type, color]) => (
            <span key={type} className="flex items-center gap-1 text-sm text-cyber-cyan/70">
              <span 
                className="w-3 h-3 rounded-full shadow-[0_0_8px_currentColor]" 
                style={{ backgroundColor: color, boxShadow: `0 0 8px ${color}` }} 
              />
              {type}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
