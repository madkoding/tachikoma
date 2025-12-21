import { useRef, useCallback, useState, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import ForceGraph2D from 'react-force-graph-2d';
import { graphApi, type Memory, type GraphEdge } from '../api/client';

interface GraphNode extends Memory {
  x?: number;
  y?: number;
  vx?: number;
  vy?: number;
}

interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  relation: string;
  weight: number;
}

const NODE_COLORS: Record<string, string> = {
  fact: '#0ea5e9',
  preference: '#22c55e',
  context: '#f59e0b',
  conversation: '#8b5cf6',
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
  const [selectedNode, setSelectedNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');

  const { data: graphData, isLoading } = useQuery({
    queryKey: ['graph-data'],
    queryFn: () => graphApi.getGraph(500),
    refetchInterval: 60000,
  });

  const { data: stats } = useQuery({
    queryKey: ['graph-stats'],
    queryFn: graphApi.getStats,
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
      graphRef.current.centerAt(node.x, node.y, 1000);
      graphRef.current.zoom(2, 1000);
    }
  }, []);

  const handleZoomIn = () => {
    if (graphRef.current) {
      graphRef.current.zoom(graphRef.current.zoom() * 1.5, 300);
    }
  };

  const handleZoomOut = () => {
    if (graphRef.current) {
      graphRef.current.zoom(graphRef.current.zoom() / 1.5, 300);
    }
  };

  const handleResetView = () => {
    if (graphRef.current) {
      graphRef.current.zoomToFit(500);
    }
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-[calc(100vh-8rem)]">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-neuro-500"></div>
      </div>
    );
  }

  return (
    <div className="space-y-4 h-[calc(100vh-8rem)]">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            {t('graph.title')}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            {t('graph.subtitle')}
          </p>
        </div>
        <div className="flex items-center gap-4">
          <span className="text-sm text-gray-600 dark:text-gray-400">
            {t('graph.nodeCount')}: {nodes.length} | {t('graph.edgeCount')}: {links.length}
          </span>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-wrap items-center gap-4 bg-white dark:bg-gray-800 p-4 rounded-xl shadow-sm">
        {/* Search */}
        <div className="flex-1 min-w-[200px]">
          <input
            type="text"
            placeholder={t('graph.search')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-neuro-500"
          />
        </div>

        {/* Filter */}
        <div>
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-neuro-500"
          >
            <option value="all">{t('graph.allTypes')}</option>
            {memoryTypes.map((type) => (
              <option key={type} value={type}>
                {type}
              </option>
            ))}
          </select>
        </div>

        {/* Zoom Controls */}
        <div className="flex items-center gap-2">
          <button
            onClick={handleZoomIn}
            className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600"
            title={t('graph.zoomIn')}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM10 7v3m0 0v3m0-3h3m-3 0H7" />
            </svg>
          </button>
          <button
            onClick={handleZoomOut}
            className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600"
            title={t('graph.zoomOut')}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0zM13 10H7" />
            </svg>
          </button>
          <button
            onClick={handleResetView}
            className="p-2 rounded-lg bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600"
            title={t('graph.resetView')}
          >
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          </button>
        </div>
      </div>

      {/* Graph Container */}
      <div className="flex gap-4 h-[calc(100%-12rem)]">
        {/* Graph */}
        <div className="flex-1 bg-white dark:bg-gray-800 rounded-xl shadow-sm overflow-hidden">
          <ForceGraph2D
            ref={graphRef}
            graphData={{ nodes, links }}
            nodeId="id"
            nodeLabel={(node: GraphNode) => `${node.memory_type}: ${node.content.substring(0, 50)}...`}
            nodeColor={(node: GraphNode) => NODE_COLORS[node.memory_type] || NODE_COLORS.default}
            nodeRelSize={6}
            nodeVal={(node: GraphNode) => node.importance * 2}
            linkColor={(link: GraphLink) => RELATION_COLORS[link.relation] || '#94a3b8'}
            linkWidth={(link: GraphLink) => link.weight * 2}
            linkDirectionalArrowLength={3}
            linkDirectionalArrowRelPos={1}
            onNodeClick={handleNodeClick}
            backgroundColor="transparent"
            cooldownTicks={100}
            onEngineStop={() => graphRef.current?.zoomToFit(400)}
          />
        </div>

        {/* Selected Node Panel */}
        {selectedNode && (
          <div className="w-80 bg-white dark:bg-gray-800 rounded-xl shadow-sm p-4 overflow-auto">
            <div className="flex items-center justify-between mb-4">
              <h3 className="font-semibold text-gray-900 dark:text-white">Node Details</h3>
              <button
                onClick={() => setSelectedNode(null)}
                className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
              >
                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                  <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                </svg>
              </button>
            </div>
            
            <div className="space-y-3">
              <div>
                <span className="text-xs text-gray-500 dark:text-gray-400">Type</span>
                <p className="text-sm font-medium text-gray-900 dark:text-white">
                  <span
                    className="inline-block w-3 h-3 rounded-full mr-2"
                    style={{ backgroundColor: NODE_COLORS[selectedNode.memory_type] || NODE_COLORS.default }}
                  />
                  {selectedNode.memory_type}
                </p>
              </div>
              
              <div>
                <span className="text-xs text-gray-500 dark:text-gray-400">Content</span>
                <p className="text-sm text-gray-900 dark:text-white">{selectedNode.content}</p>
              </div>
              
              <div>
                <span className="text-xs text-gray-500 dark:text-gray-400">Importance</span>
                <p className="text-sm text-gray-900 dark:text-white">{selectedNode.importance.toFixed(2)}</p>
              </div>
              
              <div>
                <span className="text-xs text-gray-500 dark:text-gray-400">Created</span>
                <p className="text-sm text-gray-900 dark:text-white">
                  {new Date(selectedNode.created_at).toLocaleString()}
                </p>
              </div>

              <div>
                <span className="text-xs text-gray-500 dark:text-gray-400">ID</span>
                <p className="text-xs text-gray-500 dark:text-gray-400 font-mono break-all">
                  {selectedNode.id}
                </p>
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Legend */}
      <div className="flex flex-wrap gap-4 bg-white dark:bg-gray-800 p-4 rounded-xl shadow-sm">
        <div className="flex flex-wrap gap-3">
          <span className="text-sm font-medium text-gray-700 dark:text-gray-300">Node Types:</span>
          {Object.entries(NODE_COLORS).filter(([k]) => k !== 'default').map(([type, color]) => (
            <span key={type} className="flex items-center gap-1 text-sm text-gray-600 dark:text-gray-400">
              <span className="w-3 h-3 rounded-full" style={{ backgroundColor: color }} />
              {type}
            </span>
          ))}
        </div>
      </div>
    </div>
  );
}
