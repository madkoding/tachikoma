import { useRef, useCallback, useState, useEffect, useMemo } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import ForceGraph3D from 'react-force-graph-3d';

import { graphApi, memoryApi, Memory } from '../api/client';
import { RELATION_COLORS } from '../constants/graph';
import { GRAPH_CONFIG } from '../constants/graphConfig';
import type { GraphNode, GraphLink } from '../types/graph';
import MemoryModal from '../components/MemoryModal';

import { useGraphForces } from '../hooks/useGraphForces';
import { useGraphData } from '../hooks/useGraphData';
import { useTooltip } from '../hooks/useTooltip';
import { useDimensions } from '../hooks/useDimensions';
import { useNodeRenderer } from '../hooks/useNodeRenderer';
import { useHoverState } from '../hooks/useHoverState';
import { useGraphEvents, MemoryEventData, RelationEventData } from '../hooks/useGraphEvents';

import {
  GraphHeader,
  GraphControls,
  GraphLegend,
  GraphBackground,
  GraphTooltip,
} from '../components/graph';

export default function GraphPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const graphRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const savedCameraRef = useRef<{ position: {x: number, y: number, z: number}, rotation: number } | null>(null);

  const [modalNode, setModalNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [isAutoRotating, setIsAutoRotating] = useState(false);

  const { data: graphData, isLoading } = useQuery({
    queryKey: ['graph-data'],
    queryFn: () => graphApi.getGraph(500),
    // With SSE, we can reduce polling frequency significantly
    refetchInterval: 300000, // 5 minutes instead of 1 minute
  });

  // Subscribe to real-time graph events via SSE
  const { status: sseStatus } = useGraphEvents({
    onMemoryCreated: useCallback((eventData: MemoryEventData) => {
      console.log('[GraphPage] SSE: Memory created', eventData);
      // Add new memory to the cache optimistically
      queryClient.setQueryData<{ nodes: Memory[]; edges: any[] }>(['graph-data'], (oldData) => {
        if (!oldData) return oldData;
        
        // Check if node already exists
        const exists = oldData.nodes.some(n => n.id === eventData.id);
        if (exists) {
          console.log('[GraphPage] Node already exists, skipping');
          return oldData;
        }
        
        // Create a new memory from event data
        const newMemory: Memory = {
          id: eventData.id,
          content: eventData.content,
          memory_type: eventData.memory_type,
          importance_score: 0.5, // Default
          access_count: 0,
          metadata: {},
          created_at: eventData.created_at,
          updated_at: eventData.created_at,
        };
        
        console.log('[GraphPage] Adding new node to graph:', newMemory);
        return {
          ...oldData,
          nodes: [...oldData.nodes, newMemory],
        };
      });
    }, [queryClient]),
    
    onMemoryUpdated: useCallback((eventData: MemoryEventData) => {
      console.log('[GraphPage] SSE: Memory updated', eventData);
      // Instead of manually updating the cache (which can break ForceGraph3D's node references),
      // we invalidate the query to trigger a refetch. The useGraphData hook will then
      // properly detect the content change and trigger the update animation.
      queryClient.invalidateQueries({ queryKey: ['graph-data'] });
    }, [queryClient]),
    
    onMemoryDeleted: useCallback((id: string) => {
      console.log('[GraphPage] SSE: Memory deleted', id);
      // Remove memory from cache
      queryClient.setQueryData<{ nodes: Memory[]; edges: any[] }>(['graph-data'], (oldData) => {
        if (!oldData) return oldData;
        
        return {
          ...oldData,
          nodes: oldData.nodes.filter(n => n.id !== id),
          edges: oldData.edges.filter(e => e.source !== id && e.target !== id),
        };
      });
    }, [queryClient]),
    
    onRelationCreated: useCallback((relationData: RelationEventData) => {
      console.log('[GraphPage] SSE: Relation created', relationData);
      // Add new relation to cache
      queryClient.setQueryData<{ nodes: Memory[]; edges: any[] }>(['graph-data'], (oldData) => {
        if (!oldData) return oldData;
        
        return {
          ...oldData,
          edges: [...oldData.edges, relationData],
        };
      });
    }, [queryClient]),
  });

  const dimensions = useDimensions({ containerRef });

  const { nodes, links, memoryTypes } = useGraphData({
    graphData,
    filterType,
    searchQuery,
  });

  useGraphForces(graphRef, graphData);

  const { tooltip, displayedText, tooltipFading, showTooltip, hideTooltip } = useTooltip({
    graphRef,
    containerRef,
  });

  const { hoveredNodeId, handleNodeHover } = useHoverState();

  const nodeThreeObject = useNodeRenderer({
    hoveredNodeId,
    nodes, // Pass nodes to detect updates in animation loop
  });

  // Combined hover handler for tooltip and visual effects
  const onNodeHover = useCallback((node: GraphNode | null) => {
    showTooltip(node);
    handleNodeHover(node);
  }, [showTooltip, handleNodeHover]);

  // Calcular el centro de todos los nodos (bounding box center)
  const nodesCenter = useMemo(() => {
    if (!nodes || nodes.length === 0) {
      return { x: 0, y: 0, z: 0 };
    }

    let minX = Infinity, maxX = -Infinity;
    let minY = Infinity, maxY = -Infinity;
    let minZ = Infinity, maxZ = -Infinity;

    for (const node of nodes) {
      const x = node.x ?? 0;
      const y = node.y ?? 0;
      const z = node.z ?? 0;
      
      minX = Math.min(minX, x);
      maxX = Math.max(maxX, x);
      minY = Math.min(minY, y);
      maxY = Math.max(maxY, y);
      minZ = Math.min(minZ, z);
      maxZ = Math.max(maxZ, z);
    }

    return {
      x: (minX + maxX) / 2,
      y: (minY + maxY) / 2,
      z: (minZ + maxZ) / 2,
    };
  }, [nodes]);

  // Rotación automática basada en tiempo real
  // El ángulo se calcula desde el timestamp actual, así al recargar
  // la página el grafo continúa desde la posición correcta
  useEffect(() => {
    if (!isAutoRotating || !graphRef.current) return;

    const { initialDistance, rotationSpeed } = GRAPH_CONFIG.camera;
    let frameId: number;

    const rotate = () => {
      if (graphRef.current) {
        // Calcular ángulo basado en tiempo real
        const now = Date.now();
        const radiansPerMs = rotationSpeed * 60 / 1000;
        const angle = (now * radiansPerMs) % (Math.PI * 2);
        
        // Orbitar alrededor del centro de los nodos
        const x = nodesCenter.x + initialDistance * Math.sin(angle);
        const z = nodesCenter.z + initialDistance * Math.cos(angle);
        graphRef.current.cameraPosition(
          { x, y: nodesCenter.y, z },
          nodesCenter,
          0
        );
      }
      frameId = requestAnimationFrame(rotate);
    };

    frameId = requestAnimationFrame(rotate);
    return () => cancelAnimationFrame(frameId);
  }, [isAutoRotating, nodesCenter]);

  // Inicializar cámara cuando el grafo esté listo
  // Posiciona inmediatamente usando la misma fórmula de tiempo real para evitar cortes
  useEffect(() => {
    if (dimensions && graphData && graphRef.current && nodes.length > 0) {
      const { initialDistance, rotationSpeed } = GRAPH_CONFIG.camera;
      
      // Calcular posición inicial basada en tiempo real (misma fórmula que rotación)
      const now = Date.now();
      const radiansPerMs = rotationSpeed * 60 / 1000;
      const angle = (now * radiansPerMs) % (Math.PI * 2);
      
      const x = nodesCenter.x + initialDistance * Math.sin(angle);
      const z = nodesCenter.z + initialDistance * Math.cos(angle);
      
      graphRef.current.cameraPosition(
        { x, y: nodesCenter.y, z },
        nodesCenter,
        0
      );
      setIsAutoRotating(true);
    }
  }, [dimensions, graphData, nodesCenter, nodes.length]);

  // Pausar rotación en interacción
  const pauseRotation = useCallback(() => {
    setIsAutoRotating(false);
    setTimeout(() => {
      if (!modalNode) {
        setIsAutoRotating(true);
      }
    }, GRAPH_CONFIG.camera.resumeDelay);
  }, [modalNode]);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    container.addEventListener('wheel', pauseRotation);
    container.addEventListener('mousedown', pauseRotation);
    container.addEventListener('touchstart', pauseRotation);

    return () => {
      container.removeEventListener('wheel', pauseRotation);
      container.removeEventListener('mousedown', pauseRotation);
      container.removeEventListener('touchstart', pauseRotation);
    };
  }, [pauseRotation]);

  // Mutations
  const updateMutation = useMutation({
    mutationFn: async ({
      id,
      content,
      memoryType,
      importance,
    }: {
      id: string;
      content: string;
      memoryType: string;
      importance: number;
    }) => {
      return memoryApi.update(id, {
        content,
        memory_type: memoryType,
        importance_score: importance,
      });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph-data'] });
      setModalNode(null);
    },
  });

  const deleteMutation = useMutation({
    mutationFn: (id: string) => memoryApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph-data'] });
      setModalNode(null);
    },
  });

  const handleNodeClick = useCallback((node: GraphNode) => {
    setIsAutoRotating(false);
    hideTooltip();

    if (graphRef.current) {
      // Guardar posición actual de la cámara
      const currentPos = graphRef.current.cameraPosition();
      savedCameraRef.current = {
        position: { x: currentPos.x, y: currentPos.y, z: currentPos.z },
        rotation: 0 // Ya no necesitamos guardar rotación, usa tiempo real
      };

      const distance = 80;
      const nodePos = { x: node.x || 0, y: node.y || 0, z: node.z || 0 };
      const distRatio = 1 + distance / Math.hypot(nodePos.x, nodePos.y, nodePos.z);

      graphRef.current.cameraPosition(
        { x: nodePos.x * distRatio, y: nodePos.y * distRatio, z: nodePos.z * distRatio },
        nodePos,
        1500
      );

      setTimeout(() => setModalNode(node), 1600);
    }
  }, [hideTooltip]);

  const handleCloseModal = useCallback(() => {
    setModalNode(null);
    
    if (graphRef.current && savedCameraRef.current) {
      // Restaurar posición guardada
      const { position } = savedCameraRef.current;
      graphRef.current.cameraPosition(
        position,
        { x: 0, y: 0, z: 0 },
        1500
      );
      
      setTimeout(() => {
        savedCameraRef.current = null;
        setIsAutoRotating(true);
      }, 1600);
    } else {
      setIsAutoRotating(true);
    }
  }, []);

  const handleResetView = useCallback(() => {
    const { initialDistance, initialY } = GRAPH_CONFIG.camera;
    if (graphRef.current) {
      graphRef.current.cameraPosition(
        { x: 0, y: initialY, z: initialDistance },
        { x: 0, y: 0, z: 0 },
        1500
      );
      setIsAutoRotating(true);
    }
  }, []);

  const handleBackgroundClick = useCallback(() => {
    hideTooltip();
    pauseRotation();
  }, [hideTooltip, pauseRotation]);

  const handleSaveMemory = (id: string, content: string, memoryType: string, importance: number) => {
    updateMutation.mutate({ id, content, memoryType, importance });
  };

  const handleDeleteMemory = (id: string) => {
    deleteMutation.mutate(id);
  };

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="cyber-spinner"></div>
      </div>
    );
  }

  const { simulation } = GRAPH_CONFIG;

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <GraphHeader nodeCount={nodes.length} linkCount={links.length} connectionStatus={sseStatus} />

      <GraphControls
        searchQuery={searchQuery}
        onSearchChange={setSearchQuery}
        filterType={filterType}
        onFilterChange={setFilterType}
        memoryTypes={memoryTypes}
        onResetView={handleResetView}
      />

      <div className="flex-1 min-h-0 p-2 md:p-4 pt-2 flex flex-col">
        <div
          ref={containerRef}
          className="flex-1 cyber-card overflow-hidden relative nebula-background"
          style={{ minHeight: '300px' }}
        >
          <GraphBackground />

          {dimensions && (
            <ForceGraph3D
              ref={graphRef}
              width={dimensions.width}
              height={dimensions.height}
              graphData={{ nodes, links }}
              nodeId="id"
              nodeLabel=""
              nodeThreeObject={nodeThreeObject}
              nodeThreeObjectExtend={false}
              nodeRelSize={20}
              linkColor={(link: GraphLink) => RELATION_COLORS[link.relation] || '#00f5ff'}
              linkWidth={1}
              linkOpacity={0.25}
              linkDirectionalArrowLength={0}
              linkDirectionalParticles={2}
              linkDirectionalParticleWidth={1.5}
              linkDirectionalParticleSpeed={0.006}
              linkDirectionalParticleColor={() => '#00f5ff'}
              d3AlphaDecay={simulation.alphaDecay}
              d3VelocityDecay={simulation.velocityDecay}
              warmupTicks={simulation.warmupTicks}
              cooldownTicks={simulation.cooldownTicks}
              cooldownTime={simulation.cooldownTime}
              onNodeClick={handleNodeClick}
              onNodeHover={onNodeHover}
              onBackgroundClick={handleBackgroundClick}
              backgroundColor="rgba(0,0,0,0)"
              showNavInfo={false}
              enableNodeDrag={true}
              enableNavigationControls={true}
              controlType="trackball"
            />
          )}

          <GraphTooltip tooltip={tooltip} displayedText={displayedText} fading={tooltipFading} />

          <div className="absolute bottom-2 left-2 text-xs text-cyber-cyan/40 font-mono hidden md:block">
            {t('graph.scrollZoom')} • {t('graph.dragRotate')}
          </div>
          <div className="absolute bottom-2 left-2 text-xs text-cyber-cyan/40 font-mono md:hidden">
            {t('graph.tapHint')}
          </div>
        </div>
      </div>

      <GraphLegend />

      {modalNode && (
        <MemoryModal
          node={modalNode}
          onClose={handleCloseModal}
          onSave={handleSaveMemory}
          onDelete={handleDeleteMemory}
          isLoading={updateMutation.isPending || deleteMutation.isPending}
        />
      )}
    </div>
  );
}
