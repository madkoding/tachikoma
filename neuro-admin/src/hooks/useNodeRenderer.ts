import { useCallback } from 'react';
import * as THREE from 'three';
import { NODE_COLORS } from '../constants/graph';
import type { GraphNode } from '../types/graph';

export function useNodeRenderer(highlightedIds: Set<string>) {
  const nodeThreeObject = useCallback(
    (node: GraphNode) => {
      const color = NODE_COLORS[node.memory_type] || NODE_COLORS.default;
      const size = 4 + (node.importance_score || 0.5) * 4;
      const isHighlighted = highlightedIds.has(node.id);

      const group = new THREE.Group();

      // Core sphere
      const geometry = new THREE.SphereGeometry(size * 0.6, 16, 16);
      const material = new THREE.MeshBasicMaterial({
        color: '#ffffff',
        transparent: true,
        opacity: isHighlighted ? 1 : 0.95,
      });
      const sphere = new THREE.Mesh(geometry, material);
      group.add(sphere);

      // Inner glow
      const innerGlowGeometry = new THREE.SphereGeometry(size, 16, 16);
      const innerGlowMaterial = new THREE.MeshBasicMaterial({
        color: color,
        transparent: true,
        opacity: isHighlighted ? 0.8 : 0.6,
      });
      const innerGlow = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
      group.add(innerGlow);

      // Outer glow
      const glowGeometry = new THREE.SphereGeometry(size * 2, 16, 16);
      const glowMaterial = new THREE.MeshBasicMaterial({
        color: color,
        transparent: true,
        opacity: isHighlighted ? 0.3 : 0.15,
      });
      const glow = new THREE.Mesh(glowGeometry, glowMaterial);
      group.add(glow);

      // Star rays
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

      // Outer ring for highlighted
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

      // Twinkle animation
      const twinkle = () => {
        const time = Date.now() * 0.003;
        const twinkleValue = 0.7 + Math.sin(time + (node.id.codePointAt(0) || 0)) * 0.3;
        material.opacity = twinkleValue;
        innerGlowMaterial.opacity = (isHighlighted ? 0.8 : 0.6) * twinkleValue;
        requestAnimationFrame(twinkle);
      };
      twinkle();

      return group;
    },
    [highlightedIds]
  );

  return nodeThreeObject;
}
