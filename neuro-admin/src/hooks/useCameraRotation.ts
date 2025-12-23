import { useEffect, useRef, useCallback } from 'react';
import { GRAPH_CONFIG } from '../constants/graphConfig';

interface UseCameraRotationProps {
  graphRef: React.RefObject<any>;
  isAutoRotating: boolean;
  setIsAutoRotating: (value: boolean) => void;
  modalOpen: boolean;
}

export function useCameraRotation({
  graphRef,
  isAutoRotating,
  setIsAutoRotating,
  modalOpen,
}: UseCameraRotationProps) {
  const rotationRef = useRef<number>(0);
  const animationRef = useRef<number>();
  const resumeTimeoutRef = useRef<ReturnType<typeof setTimeout>>();
  const savedCameraPositionRef = useRef<{ x: number; y: number; z: number } | null>(null);

  // Establecer posición inicial de la cámara
  const initializeCamera = useCallback(() => {
    if (!graphRef.current) return;
    
    const { initialDistance, initialY } = GRAPH_CONFIG.camera;
    graphRef.current.cameraPosition(
      { x: 0, y: initialY, z: initialDistance },
      { x: 0, y: 0, z: 0 },
      0
    );
    rotationRef.current = Math.PI / 2; // Iniciar desde z positivo
  }, [graphRef]);

  // Efecto de rotación automática
  useEffect(() => {
    if (!isAutoRotating || !graphRef.current) return;

    const { initialDistance, initialY, rotationSpeed } = GRAPH_CONFIG.camera;

    const rotate = () => {
      if (graphRef.current) {
        rotationRef.current += rotationSpeed;
        const x = initialDistance * Math.sin(rotationRef.current);
        const z = initialDistance * Math.cos(rotationRef.current);
        graphRef.current.cameraPosition({ x, y: initialY, z }, { x: 0, y: 0, z: 0 }, 0);
        animationRef.current = requestAnimationFrame(rotate);
      }
    };

    animationRef.current = requestAnimationFrame(rotate);

    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [isAutoRotating, graphRef]);

  // Pausar rotación y programar reanudación
  const pauseRotation = useCallback(() => {
    setIsAutoRotating(false);

    if (resumeTimeoutRef.current) {
      clearTimeout(resumeTimeoutRef.current);
    }

    resumeTimeoutRef.current = setTimeout(() => {
      if (!modalOpen && graphRef.current) {
        const currentPos = graphRef.current.cameraPosition();
        rotationRef.current = Math.atan2(currentPos.x, currentPos.z);
      }
      if (!modalOpen) {
        setIsAutoRotating(true);
      }
    }, GRAPH_CONFIG.camera.resumeDelay);
  }, [graphRef, modalOpen, setIsAutoRotating]);

  // Guardar posición de cámara antes de acercar a un nodo
  const saveCameraPosition = useCallback(() => {
    if (graphRef.current) {
      const pos = graphRef.current.cameraPosition();
      savedCameraPositionRef.current = { x: pos.x, y: pos.y, z: pos.z };
    }
  }, [graphRef]);

  // Restaurar posición de cámara
  const restoreCameraPosition = useCallback(() => {
    if (graphRef.current && savedCameraPositionRef.current) {
      graphRef.current.cameraPosition(
        savedCameraPositionRef.current,
        { x: 0, y: 0, z: 0 },
        1500
      );

      setTimeout(() => {
        if (savedCameraPositionRef.current) {
          rotationRef.current = Math.atan2(
            savedCameraPositionRef.current.x,
            savedCameraPositionRef.current.z
          );
        }
        savedCameraPositionRef.current = null;
        setIsAutoRotating(true);
      }, 1600);
    } else {
      setIsAutoRotating(true);
    }
  }, [graphRef, setIsAutoRotating]);

  // Enfocar cámara en un nodo
  const focusOnNode = useCallback((node: { x?: number; y?: number; z?: number }) => {
    if (!graphRef.current) return;

    const distance = 80;
    const nodePos = { x: node.x || 0, y: node.y || 0, z: node.z || 0 };
    const distRatio = 1 + distance / Math.hypot(nodePos.x, nodePos.y, nodePos.z);

    graphRef.current.cameraPosition(
      { x: nodePos.x * distRatio, y: nodePos.y * distRatio, z: nodePos.z * distRatio },
      nodePos,
      1500
    );
  }, [graphRef]);

  // Limpiar al desmontar
  useEffect(() => {
    return () => {
      if (resumeTimeoutRef.current) {
        clearTimeout(resumeTimeoutRef.current);
      }
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  return {
    initializeCamera,
    pauseRotation,
    saveCameraPosition,
    restoreCameraPosition,
    focusOnNode,
    resumeTimeoutRef,
  };
}
