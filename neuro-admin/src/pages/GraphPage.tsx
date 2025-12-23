import { useRef, useCallback, useState, useMemo, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import ForceGraph3D from 'react-force-graph-3d';
import * as THREE from 'three';

import { graphApi, memoryApi } from '../api/client';
import { filterNodes, createRealLinks, buildVirtualLinks } from '../utils/graphTransform';
import { NODE_COLORS, RELATION_COLORS } from '../constants/graph';
import type { GraphNode, GraphLink } from '../types/graph';
import MemoryModal from '../components/MemoryModal';

export default function GraphPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const graphRef = useRef<any>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const rotationRef = useRef<number>(0);
  const animationRef = useRef<number>();
  const resumeRotationTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  
  const [modalNode, setModalNode] = useState<GraphNode | null>(null);
  const [filterType, setFilterType] = useState<string>('all');
  const [searchQuery, setSearchQuery] = useState('');
  const [dimensions, setDimensions] = useState<{ width: number; height: number } | null>(null);
  const [isAutoRotating, setIsAutoRotating] = useState(true);
  const [tooltip, setTooltip] = useState<{ text: string; x: number; y: number; alignment?: 'left' | 'center' | 'right' } | null>(null);
  const [displayedText, setDisplayedText] = useState('');
  const [tooltipFading, setTooltipFading] = useState(false);
  const tooltipTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const savedCameraPositionRef = useRef<{ x: number; y: number; z: number } | null>(null);

  // Helper to check if dimensions changed significantly
  const shouldUpdateDimensions = useCallback(
    (prev: { width: number; height: number } | null, newDims: { width: number; height: number }) => {
      if (!prev) return newDims;
      const hasChanged = Math.abs(prev.width - newDims.width) > 1 || 
                        Math.abs(prev.height - newDims.height) > 1;
      return hasChanged ? newDims : prev;
    },
    []
  );

  // Update dimensions on resize - con múltiples intentos para refresh
  useEffect(() => {
    let rafId: number;
    const timeoutIds: ReturnType<typeof setTimeout>[] = [];
    let mounted = true;
    
    const calculateDimensions = () => {
      if (!containerRef.current) return null;
      const width = containerRef.current.offsetWidth;
      const height = containerRef.current.offsetHeight;
      if (width < 100 || height < 100) return null;
      return { width, height };
    };
    
    const applyDimensions = (newDims: { width: number; height: number }) => {
      setDimensions(prev => shouldUpdateDimensions(prev, newDims));
    };
    
    const updateDimensions = () => {
      if (!mounted) return;
      if (rafId) cancelAnimationFrame(rafId);
      
      rafId = requestAnimationFrame(() => {
        if (!mounted) return;
        const newDims = calculateDimensions();
        if (newDims) applyDimensions(newDims);
      });
    };
    
    // Múltiples intentos para capturar el layout correcto después de refresh
    const scheduleUpdates = () => {
      updateDimensions();
      // Intentar varias veces con delays progresivos
      [50, 100, 200, 300, 500, 750, 1000, 1500, 2000].forEach(delay => {
        timeoutIds.push(setTimeout(updateDimensions, delay));
      });
    };
    
    // Ejecutar cuando el documento esté listo
    if (document.readyState === 'complete') {
      scheduleUpdates();
    } else {
      window.addEventListener('load', scheduleUpdates, { once: true });
    }
    
    const resizeObserver = new ResizeObserver(updateDimensions);
    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }
    
    window.addEventListener('resize', updateDimensions);
    
    return () => {
      mounted = false;
      if (rafId) cancelAnimationFrame(rafId);
      timeoutIds.forEach(id => clearTimeout(id));
      resizeObserver.disconnect();
      window.removeEventListener('resize', updateDimensions);
    };
  }, []);

  // Pause auto-rotation on user interaction, resume after 5 seconds
  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;
    
    const pauseAndResumeRotation = () => {
      setIsAutoRotating(false);
      
      if (resumeRotationTimeoutRef.current) {
        clearTimeout(resumeRotationTimeoutRef.current);
      }
      
      // Reanudar rotación después de 10 segundos sin cambiar zoom
      resumeRotationTimeoutRef.current = setTimeout(() => {
        if (graphRef.current && !modalNode) {
          // Obtener posición actual de cámara para mantener distancia
          const currentPos = graphRef.current.cameraPosition();
          // Actualizar rotationRef para continuar desde la posición actual
          rotationRef.current = Math.atan2(currentPos.x, currentPos.z);
        }
        setIsAutoRotating(true);
      }, 10000);
    };
    
    container.addEventListener('wheel', pauseAndResumeRotation);
    container.addEventListener('mousedown', pauseAndResumeRotation);
    container.addEventListener('touchstart', pauseAndResumeRotation);
    
    return () => {
      container.removeEventListener('wheel', pauseAndResumeRotation);
      container.removeEventListener('mousedown', pauseAndResumeRotation);
      container.removeEventListener('touchstart', pauseAndResumeRotation);
      if (resumeRotationTimeoutRef.current) {
        clearTimeout(resumeRotationTimeoutRef.current);
      }
    };
  }, []);

  // Auto rotation effect - mantiene la distancia actual de la cámara
  useEffect(() => {
    if (!isAutoRotating || !graphRef.current) return;
    
    // Obtener distancia actual para mantenerla
    const currentPos = graphRef.current.cameraPosition();
    const currentDistance = Math.hypot(currentPos.x, currentPos.y, currentPos.z) || 1200;
    const currentY = currentPos.y || 150;
    
    const rotate = () => {
      if (graphRef.current && isAutoRotating) {
        rotationRef.current += 0.001;
        const x = currentDistance * Math.sin(rotationRef.current);
        const z = currentDistance * Math.cos(rotationRef.current);
        graphRef.current.cameraPosition({ x, y: currentY, z }, { x: 0, y: 0, z: 0 }, 0);
      }
      animationRef.current = requestAnimationFrame(rotate);
    };
    
    animationRef.current = requestAnimationFrame(rotate);
    
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isAutoRotating]);

  const { data: graphData, isLoading } = useQuery({
    queryKey: ['graph-data'],
    queryFn: () => graphApi.getGraph(500),
    refetchInterval: 60000,
  });

  // Efecto typewriter para el tooltip
  useEffect(() => {
    if (!tooltip) {
      setDisplayedText('');
      return;
    }
    
    setDisplayedText('');
    let index = 0;
    const text = tooltip.text;
    
    const typeInterval = setInterval(() => {
      if (index < text.length) {
        setDisplayedText(text.substring(0, index + 1));
        index++;
      } else {
        clearInterval(typeInterval);
      }
    }, 15); // Velocidad del typewriter
    
    return () => clearInterval(typeInterval);
  }, [tooltip]);

  // Configure d3 forces for spread out layout with weak links
  useEffect(() => {
    const fg = graphRef.current;
    if (!fg) return;
    
    const timer = setTimeout(() => {
      const linkForce = fg.d3Force('link');
      if (linkForce) {
        linkForce.distance(() => 120).strength(() => 0.1);
      }
      
      fg.d3ReheatSimulation();
    }, 500);
    
    return () => clearTimeout(timer);
  }, [graphData]);

  // Mutation for updating memory
  const updateMutation = useMutation({
    mutationFn: async ({ id, content, memoryType, importance }: { id: string; content: string; memoryType: string; importance: number }) => {
      return memoryApi.update(id, { content, memory_type: memoryType, importance_score: importance });
    },
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph-data'] });
      setModalNode(null);
    },
  });

  // Mutation for deleting memory
  const deleteMutation = useMutation({
    mutationFn: (id: string) => memoryApi.delete(id),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['graph-data'] });
      setModalNode(null);
    },
  });

  // Transform data for force graph
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

    return {
      nodes: filteredNodes.map(n => ({ 
        ...n, 
        __highlighted: highlightIds.has(n.id) 
      })) as GraphNode[],
      links: [...realLinks, ...virtualLinks],
      currentHighlightedIds: highlightIds,
    };
  }, [graphData, filterType, searchQuery]);

  // Focus camera on first search result
  useEffect(() => {
    if (searchQuery && nodes.length > 0 && graphRef.current) {
      const firstMatch = nodes[0];
      if (firstMatch.x !== undefined) {
        const distance = 150;
        const distRatio = 1 + distance / Math.hypot(firstMatch.x || 0, firstMatch.y || 0, firstMatch.z || 0);
        graphRef.current.cameraPosition(
          { x: (firstMatch.x || 0) * distRatio, y: (firstMatch.y || 0) * distRatio, z: (firstMatch.z || 0) * distRatio },
          firstMatch,
          1500
        );
        setIsAutoRotating(false);
      }
    }
  }, [searchQuery, nodes]);

  const memoryTypes = useMemo(() => {
    if (!graphData) return [];
    const types = new Set(graphData.nodes.map(n => n.memory_type));
    return Array.from(types);
  }, [graphData]);

  const handleNodeClick = useCallback((node: GraphNode) => {
    setIsAutoRotating(false);
    setTooltip(null);
    
    if (resumeRotationTimeoutRef.current) {
      clearTimeout(resumeRotationTimeoutRef.current);
    }
    
    if (graphRef.current) {
      // Guardar posición actual de la cámara
      const currentPos = graphRef.current.cameraPosition();
      savedCameraPositionRef.current = { x: currentPos.x, y: currentPos.y, z: currentPos.z };
      
      // Acercar la cámara al nodo
      const distance = 80;
      const nodePos = { x: node.x || 0, y: node.y || 0, z: node.z || 0 };
      const distRatio = 1 + distance / Math.hypot(nodePos.x, nodePos.y, nodePos.z);
      
      graphRef.current.cameraPosition(
        { x: nodePos.x * distRatio, y: nodePos.y * distRatio, z: nodePos.z * distRatio },
        nodePos,
        1500
      );
      
      // Abrir modal después de que la cámara termine de acercarse
      setTimeout(() => {
        setModalNode(node);
      }, 1600);
    }
  }, []);

  const handleCloseModal = useCallback(() => {
    setModalNode(null);
    
    if (graphRef.current && savedCameraPositionRef.current) {
      // Alejar la cámara a la posición guardada
      graphRef.current.cameraPosition(
        savedCameraPositionRef.current,
        { x: 0, y: 0, z: 0 },
        1500
      );
      
      // Reanudar rotación después de que la cámara termine de alejarse
      setTimeout(() => {
        if (graphRef.current) {
          const pos = savedCameraPositionRef.current!;
          rotationRef.current = Math.atan2(pos.x, pos.z);
        }
        savedCameraPositionRef.current = null;
        setIsAutoRotating(true);
      }, 1600);
    } else {
      setIsAutoRotating(true);
    }
  }, []);

  const handleResetView = () => {
    if (graphRef.current) {
      graphRef.current.cameraPosition({ x: 0, y: 80, z: 600 }, { x: 0, y: 0, z: 0 }, 1500);
      setIsAutoRotating(true);
    }
  };

  const handleSaveMemory = (id: string, content: string, memoryType: string, importance: number) => {
    updateMutation.mutate({ id, content, memoryType, importance });
  };

  const handleDeleteMemory = (id: string) => {
    deleteMutation.mutate(id);
  };

  // Calcular posición del tooltip dentro del contenedor
  const calculateTooltipPosition = useCallback((node: GraphNode) => {
    if (!graphRef.current || !containerRef.current) return null;
    
    const screenCoords = graphRef.current.graph2ScreenCoords(node.x || 0, node.y || 0, node.z || 0);
    const containerRect = containerRef.current.getBoundingClientRect();
    const tooltipWidth = 280;
    const tooltipHeight = 40;
    const padding = 15;
    
    // Determinar alineación horizontal basada en posición
    let adjustedX = screenCoords.x;
    let alignment: 'left' | 'center' | 'right' = 'center';
    
    if (screenCoords.x < tooltipWidth / 2 + padding) {
      // Muy a la izquierda: alinear a la izquierda
      adjustedX = padding;
      alignment = 'left';
    } else if (screenCoords.x > containerRect.width - tooltipWidth / 2 - padding) {
      // Muy a la derecha: alinear a la derecha
      adjustedX = containerRect.width - padding;
      alignment = 'right';
    }
    
    // Ajustar Y para que no se salga por arriba
    let adjustedY = screenCoords.y - 30;
    if (adjustedY - tooltipHeight < padding) {
      adjustedY = screenCoords.y + 50;
    }
    // Ajustar Y para que no se salga por abajo
    if (adjustedY > containerRect.height - padding) {
      adjustedY = containerRect.height - padding;
    }
    
    return {
      text: node.content.substring(0, 100) + (node.content.length > 100 ? '...' : ''),
      x: adjustedX,
      y: adjustedY,
      alignment
    };
  }, []);

  const handleNodeHover = useCallback((node: GraphNode | null) => {
    if (tooltipTimeoutRef.current) {
      clearTimeout(tooltipTimeoutRef.current);
    }
    
    if (!node || !containerRef.current) {
      // Iniciar desvanecimiento después de 3 segundos
      tooltipTimeoutRef.current = setTimeout(() => {
        setTooltipFading(true);
        setTimeout(() => {
          setTooltip(null);
          setTooltipFading(false);
        }, 1000);
      }, 3000);
      return;
    }
    
    const newText = node.content.substring(0, 100) + (node.content.length > 100 ? '...' : '');
    const isNewNode = tooltip?.text !== newText;
    
    if (isNewNode && tooltip) {
      // Fade out el tooltip actual, luego mostrar el nuevo
      setTooltipFading(true);
      setTimeout(() => {
        const newPosition = calculateTooltipPosition(node);
        if (newPosition) {
          setTooltip(newPosition);
          setTooltipFading(false);
        }
      }, 300);
    } else if (!tooltip) {
      // Mostrar directamente
      const newPosition = calculateTooltipPosition(node);
      if (newPosition) {
        setTooltip(newPosition);
        setTooltipFading(false);
      }
    }
  }, [tooltip, calculateTooltipPosition]);

  // Custom node object with star-like glow effect and twinkle
  const nodeThreeObject = useCallback((node: GraphNode) => {
    const color = NODE_COLORS[node.memory_type] || NODE_COLORS.default;
    const size = 4 + (node.importance_score || 0.5) * 4;
    const isHighlighted = currentHighlightedIds.has(node.id);
    
    const group = new THREE.Group();
    
    // Core sphere (núcleo brillante de la estrella)
    const geometry = new THREE.SphereGeometry(size * 0.6, 16, 16);
    const material = new THREE.MeshBasicMaterial({
      color: '#ffffff',
      transparent: true,
      opacity: isHighlighted ? 1 : 0.95,
    });
    const sphere = new THREE.Mesh(geometry, material);
    group.add(sphere);
    
    // Inner glow (brillo interno coloreado)
    const innerGlowGeometry = new THREE.SphereGeometry(size, 16, 16);
    const innerGlowMaterial = new THREE.MeshBasicMaterial({
      color: color,
      transparent: true,
      opacity: isHighlighted ? 0.8 : 0.6,
    });
    const innerGlow = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
    group.add(innerGlow);
    
    // Outer glow (halo difuso)
    const glowGeometry = new THREE.SphereGeometry(size * 2, 16, 16);
    const glowMaterial = new THREE.MeshBasicMaterial({
      color: color,
      transparent: true,
      opacity: isHighlighted ? 0.3 : 0.15,
    });
    const glow = new THREE.Mesh(glowGeometry, glowMaterial);
    group.add(glow);
    
    // Star rays (rayos de estrella con puntos de luz)
    const rayCount = 4;
    for (let i = 0; i < rayCount; i++) {
      const rayGeometry = new THREE.SphereGeometry(size * 0.15, 8, 8);
      const rayMaterial = new THREE.MeshBasicMaterial({
        color: '#ffffff',
        transparent: true,
        opacity: 0.7,
      });
      const ray = new THREE.Mesh(rayGeometry, rayMaterial);
      const angle = (i / rayCount) * Math.PI * 2;
      const rayDistance = size * 1.8;
      ray.position.x = Math.cos(angle) * rayDistance;
      ray.position.y = Math.sin(angle) * rayDistance;
      group.add(ray);
    }
    
    // Outer ring for highlighted nodes
    if (isHighlighted) {
      const ringGeometry = new THREE.RingGeometry(size * 2.5, size * 2.8, 32);
      const ringMaterial = new THREE.MeshBasicMaterial({
        color: '#ffffff',
        transparent: true,
        opacity: 0.6,
        side: THREE.DoubleSide,
      });
      const ring = new THREE.Mesh(ringGeometry, ringMaterial);
      ring.rotation.x = Math.PI / 2;
      group.add(ring);
    }
    
    // Animación de twinkle (parpadeo de estrella)
    const twinkle = () => {
      const time = Date.now() * 0.003;
      const twinkleValue = 0.7 + Math.sin(time + (node.id.codePointAt(0) || 0)) * 0.3;
      material.opacity = twinkleValue;
      innerGlowMaterial.opacity = (isHighlighted ? 0.8 : 0.6) * twinkleValue;
      requestAnimationFrame(twinkle);
    };
    twinkle();
    
    return group;
  }, [currentHighlightedIds]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="cyber-spinner"></div>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full overflow-hidden">
      {/* Header */}
      <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-2 p-2 md:p-3 shrink-0">
        <div className="flex items-center gap-4">
          <div>
            <h1 className="text-lg md:text-xl font-bold neon-cyan font-cyber tracking-wider">
              {t('graph.title')}
            </h1>
            <p className="text-cyber-cyan/60 font-mono text-xs hidden sm:block">
              {t('graph.subtitle')}
            </p>
          </div>
        </div>
        <div className="flex items-center gap-2 md:gap-4 text-xs md:text-sm">
          <span className="text-cyber-cyan/70 font-mono">
            {t('graph.nodeCount')}: <span className="neon-green">{nodes.length}</span>
          </span>
          <span className="text-cyber-cyan/70 font-mono">
            {t('graph.edgeCount')}: <span className="neon-magenta">{links.length}</span>
          </span>
        </div>
      </div>

      {/* Controls */}
      <div className="flex flex-wrap items-center gap-2 md:gap-4 cyber-card mx-2 md:mx-4 p-2 md:p-4 shrink-0">
        <div className="flex-1 min-w-[150px] md:min-w-[200px]">
          <input
            type="text"
            placeholder={t('graph.search')}
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="cyber-input w-full text-sm"
          />
        </div>

        <div className="w-auto">
          <select
            value={filterType}
            onChange={(e) => setFilterType(e.target.value)}
            className="cyber-input text-sm"
          >
            <option value="all">{t('graph.allTypes')}</option>
            {memoryTypes.map((type) => (
              <option key={type} value={type}>{t(`graph.types.${type}`, type)}</option>
            ))}
          </select>
        </div>

        <button
          onClick={handleResetView}
          className="cyber-button p-2"
          title={t('graph.resetView')}
        >
          <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
          </svg>
        </button>
      </div>

      {/* Main content area */}
      <div className="flex-1 min-h-0 p-2 md:p-4 pt-2 flex flex-col">
        {/* Graph Container */}
        <div 
          ref={containerRef}
          className="flex-1 cyber-card overflow-hidden relative nebula-background"
          style={{ minHeight: '300px' }}
        >
          {/* Nebula layers */}
          <div className="nebula-layer nebula-1"></div>
          <div className="nebula-layer nebula-2"></div>
          <div className="nebula-layer nebula-3"></div>
          
          {/* Stars */}
          <div className="stars-layer"></div>
          
          {/* Grid overlay effect */}
          <div className="absolute inset-0 opacity-5 pointer-events-none"
               style={{ 
                 backgroundImage: 'linear-gradient(rgba(0,245,255,0.1) 1px, transparent 1px), linear-gradient(90deg, rgba(0,245,255,0.1) 1px, transparent 1px)',
                 backgroundSize: '50px 50px'
               }} 
          />
          
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
            linkColor={(link: GraphLink) => RELATION_COLORS[link.relation] || '#00f5ff'}
            linkWidth={1}
            linkOpacity={0.25}
            linkDirectionalArrowLength={0}
            linkDirectionalParticles={2}
            linkDirectionalParticleWidth={1.5}
            linkDirectionalParticleSpeed={0.006}
            linkDirectionalParticleColor={() => '#00f5ff'}
            d3AlphaDecay={0.02}
            d3VelocityDecay={0.3}
            onNodeClick={handleNodeClick}
            onNodeHover={handleNodeHover}
            onBackgroundClick={() => {
              if (tooltipTimeoutRef.current) {
                clearTimeout(tooltipTimeoutRef.current);
              }
              setTooltip(null);
              setTooltipFading(false);
              setIsAutoRotating(false);
              if (resumeRotationTimeoutRef.current) {
                clearTimeout(resumeRotationTimeoutRef.current);
              }
              resumeRotationTimeoutRef.current = setTimeout(() => setIsAutoRotating(true), 10000);
            }}
            backgroundColor="rgba(0,0,0,0)"
            showNavInfo={false}
            enableNodeDrag={true}
            enableNavigationControls={true}
            controlType="trackball"
          />
          )}
          
          {/* Custom Tooltip */}
          {tooltip && displayedText && (
            <div 
              className={`absolute pointer-events-none z-20 transition-opacity duration-300 ${
                tooltipFading ? 'opacity-0' : 'opacity-100'
              }`}
              style={{ 
                left: tooltip.x, 
                top: tooltip.y,
                transform: (() => {
                  if (tooltip.alignment === 'left') return 'translateY(-100%)';
                  if (tooltip.alignment === 'right') return 'translate(-100%, -100%)';
                  return 'translate(-50%, -100%)';
                })(),
                maxWidth: '280px'
              }}
            >
              <span className="text-cyber-cyan font-cyber text-sm font-bold tracking-wide drop-shadow-[0_0_10px_rgba(0,245,255,0.8)]">
                {displayedText}
              </span>
            </div>
          )}
          
          {/* Hints */}
          <div className="absolute bottom-2 left-2 text-xs text-cyber-cyan/40 font-mono hidden md:block">
            {t('graph.scrollZoom')} • {t('graph.dragRotate')}
          </div>
          <div className="absolute bottom-2 left-2 text-xs text-cyber-cyan/40 font-mono md:hidden">
            {t('graph.tapHint')}
          </div>
        </div>
      </div>

      {/* Legend */}
      <div className="flex flex-col sm:flex-row flex-wrap gap-2 md:gap-4 cyber-card mx-2 md:mx-4 mb-2 md:mb-4 p-2 md:p-4 shrink-0">
        {/* Node Types */}
        <div className="flex flex-wrap gap-2 md:gap-3 items-center flex-1">
          <span className="text-xs md:text-sm font-medium text-cyber-cyan font-mono">{t('graph.nodeTypes')}:</span>
          {Object.entries(NODE_COLORS).filter(([k]) => k !== 'default').map(([type, color]) => (
            <span key={type} className="flex items-center gap-1 text-xs text-cyber-cyan/70">
              <span 
                className="w-2 h-2 md:w-3 md:h-3 rounded-full" 
                style={{ backgroundColor: color, boxShadow: `0 0 8px ${color}` }} 
              />
              <span className="hidden sm:inline">{t(`graph.types.${type}`, type)}</span>
            </span>
          ))}
        </div>
        
        {/* Separator - visible only on sm+ */}
        <div className="hidden sm:block w-px bg-cyber-cyan/20 self-stretch" />
        
        {/* Relation Types */}
        <div className="flex flex-wrap gap-2 md:gap-3 items-center flex-1">
          <span className="text-xs md:text-sm font-medium text-cyber-magenta font-mono">{t('graph.relationTypes', 'Relations')}:</span>
          {Object.entries(RELATION_COLORS).map(([relation, color]) => (
            <span key={relation} className="flex items-center gap-1 text-xs text-cyber-cyan/70">
              <span 
                className="w-2 h-2 md:w-3 md:h-3 rounded-full" 
                style={{ backgroundColor: color, boxShadow: `0 0 8px ${color}` }} 
              />
              <span className="hidden sm:inline">{t(`graph.relations.${relation}`, relation)}</span>
            </span>
          ))}
        </div>
      </div>

      {/* Modal */}
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
