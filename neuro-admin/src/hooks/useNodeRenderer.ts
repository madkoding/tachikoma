import { useCallback, useRef, useEffect } from 'react';
import * as THREE from 'three';
import { NODE_COLORS } from '../constants/graph';
import type { GraphNode } from '../types/graph';

// Animation duration for birth effect (ms)
const BIRTH_ANIMATION_DURATION = 6000;

// Cache for halo canvases to avoid recreating them
const haloCanvasCache = new Map<string, HTMLCanvasElement>();

// Create a circular halo texture using canvas with proper transparency
function createHaloCanvas(color: string, size: number): HTMLCanvasElement {
  const cacheKey = `${color}-${size}`;
  if (haloCanvasCache.has(cacheKey)) {
    return haloCanvasCache.get(cacheKey)!;
  }

  const canvas = document.createElement('canvas');
  const resolution = 256; // Higher resolution for smoother edges
  canvas.width = resolution;
  canvas.height = resolution;
  const ctx = canvas.getContext('2d', { alpha: true })!;
  
  const centerX = resolution / 2;
  const centerY = resolution / 2;
  const outerRadius = resolution / 2 - 8;
  const innerRadius = outerRadius * 0.7;

  // Ensure canvas is fully transparent
  ctx.clearRect(0, 0, resolution, resolution);

  // Draw soft outer glow
  const glowGradient = ctx.createRadialGradient(
    centerX, centerY, innerRadius * 0.8,
    centerX, centerY, outerRadius * 1.2
  );
  glowGradient.addColorStop(0, 'rgba(255,255,255,0)');
  glowGradient.addColorStop(0.4, color + '30');
  glowGradient.addColorStop(0.6, color + '60');
  glowGradient.addColorStop(0.8, color + '30');
  glowGradient.addColorStop(1, 'rgba(255,255,255,0)');

  ctx.beginPath();
  ctx.arc(centerX, centerY, outerRadius * 1.2, 0, Math.PI * 2);
  ctx.fillStyle = glowGradient;
  ctx.fill();

  // Draw main ring with gradient stroke
  ctx.beginPath();
  ctx.arc(centerX, centerY, (innerRadius + outerRadius) / 2, 0, Math.PI * 2);
  ctx.strokeStyle = color;
  ctx.lineWidth = (outerRadius - innerRadius) * 0.5;
  ctx.lineCap = 'round';
  ctx.stroke();

  // Inner bright highlight
  ctx.beginPath();
  ctx.arc(centerX, centerY, (innerRadius + outerRadius) / 2, 0, Math.PI * 2);
  ctx.strokeStyle = 'rgba(255,255,255,0.8)';
  ctx.lineWidth = (outerRadius - innerRadius) * 0.15;
  ctx.stroke();

  haloCanvasCache.set(cacheKey, canvas);
  return canvas;
}

// Store animation state outside React to persist across renders and remounts
const nodeAnimationState = new Map<string, {
  group: THREE.Group;
  birthTime: number;
  initialized: boolean;
  size: number;
}>();

// Store groups outside React to persist across renders and remounts
const globalGroupsMap = new Map<string, THREE.Group>();

interface UseNodeRendererProps {
  hoveredNodeId: string | null;
}

export function useNodeRenderer({ hoveredNodeId }: UseNodeRendererProps) {
  const animationFrameRef = useRef<number | null>(null);

  // Global animation loop for all nodes
  useEffect(() => {
    const animate = () => {
      const now = Date.now();
      
      globalGroupsMap.forEach((group, nodeId) => {
        const state = nodeAnimationState.get(nodeId);
        if (!state) return;

        const elapsed = now - state.birthTime;
        const isNewborn = elapsed < BIRTH_ANIMATION_DURATION;
        const isHovered = hoveredNodeId === nodeId;

        // ===== HOVER HALO - Sprite always facing camera =====
        const hoverHalo = group.getObjectByName('hoverHalo') as THREE.Sprite;
        if (hoverHalo) {
          const material = hoverHalo.material as THREE.SpriteMaterial;
          if (isHovered) {
            // Fade in and gentle pulse
            const targetOpacity = 0.6 + Math.sin(now * 0.004) * 0.15;
            material.opacity += (targetOpacity - material.opacity) * 0.15;
            // Pulse scale slightly
            const pulse = 1 + Math.sin(now * 0.003) * 0.05;
            hoverHalo.scale.setScalar(state.size * 8 * pulse);
          } else {
            // Fade out
            material.opacity *= 0.85;
            if (material.opacity < 0.01) {
              material.opacity = 0;
            }
          }
        }

        // ===== BIRTH ANIMATION =====
        if (isNewborn) {
          const progress = elapsed / BIRTH_ANIMATION_DURATION;
          const easeOut = 1 - Math.pow(1 - progress, 3);
          
          // Starburst rays rotation and expansion - larger and more dramatic
          const supernovaRing = group.getObjectByName('supernovaRing') as THREE.Sprite;
          if (supernovaRing) {
            const baseSize = state.size * 4;
            const expandScale = baseSize * (1 + easeOut * 10);
            supernovaRing.scale.setScalar(expandScale);
            // Fade slower for more visibility
            supernovaRing.material.opacity = Math.pow(1 - progress, 1.5) * 1.0;
            supernovaRing.material.rotation += 0.008; // Slower rotation
          }

          // Shockwave rings - sequential expansion, much larger
          for (let i = 0; i < 3; i++) {
            const shockwave = group.getObjectByName(`shockwave${i}`) as THREE.Sprite;
            if (shockwave) {
              const delay = i * 0.25; // More delay between waves
              const shockProgress = Math.max(0, (progress - delay) / (1 - delay));
              const easeShock = 1 - Math.pow(1 - shockProgress, 2);
              const baseSize = state.size * 1;
              const shockScale = baseSize + easeShock * state.size * (15 + i * 8);
              shockwave.scale.setScalar(Math.max(0.1, shockScale));
              // Slower fade
              shockwave.material.opacity = Math.max(0, Math.pow(1 - shockProgress, 1.5)) * 0.85;
            }
          }

          // Particle burst - stardust expanding outward
          const particles = group.getObjectByName('birthParticles') as THREE.Points;
          if (particles) {
            const positions = particles.geometry.attributes.position.array as Float32Array;
            const velocities = (particles.userData as { velocities: Float32Array }).velocities;
            
            for (let i = 0; i < positions.length; i += 3) {
              // Particles slow down as they travel - travel farther
              const decay = Math.pow(1 - progress, 0.25);
              positions[i] += velocities[i] * decay * 0.5;
              positions[i + 1] += velocities[i + 1] * decay * 0.5;
              positions[i + 2] += velocities[i + 2] * decay * 0.5;
            }
            particles.geometry.attributes.position.needsUpdate = true;
            // Particles fade out slower
            (particles.material as THREE.PointsMaterial).opacity = Math.pow(1 - progress, 1.2);
            // Particles shrink as they fade
            (particles.material as THREE.PointsMaterial).size = state.size * 0.5 * (1 - progress * 0.5);
          }

          // Core flash - bright initial flash then slower fade
          const coreFlash = group.getObjectByName('coreFlash') as THREE.Sprite;
          if (coreFlash) {
            const flashPhase = progress < 0.1 
              ? progress / 0.1  // Quick initial flash ramp up
              : Math.pow(1 - (progress - 0.1) / 0.9, 2);  // Slower fade out
            const baseSize = state.size * 12;
            coreFlash.scale.setScalar(baseSize * (0.6 + flashPhase * 2));
            coreFlash.material.opacity = flashPhase * 1.0;
          }
        } else {
          // Clean up birth elements after animation
          cleanupBirthElements(group);
        }
      });

      animationFrameRef.current = requestAnimationFrame(animate);
    };

    animationFrameRef.current = requestAnimationFrame(animate);

    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, [hoveredNodeId]);

  const nodeThreeObject = useCallback(
    (node: GraphNode) => {
      const color = NODE_COLORS[node.memory_type] || NODE_COLORS.default;
      const size = 4 + (node.importance_score || 0.5) * 4;
      // Use nullish coalescing to accept 0 as valid birthTime (initial load)
      const birthTime = node.__birthTime ?? Date.now();
      const elapsed = Date.now() - birthTime;
      const isNewborn = elapsed < BIRTH_ANIMATION_DURATION;

      // Reuse existing group if possible from global map
      let group = globalGroupsMap.get(node.id);
      if (group) {
        return group;
      }

      group = new THREE.Group();
      globalGroupsMap.set(node.id, group);
      nodeAnimationState.set(node.id, { group, birthTime, initialized: true, size });

      // ===== CORE STAR =====
      const coreGeometry = new THREE.SphereGeometry(size * 0.6, 32, 32);
      const coreMaterial = new THREE.MeshBasicMaterial({
        color: '#ffffff',
        transparent: true,
        opacity: 0.95,
      });
      const core = new THREE.Mesh(coreGeometry, coreMaterial);
      core.name = 'core';
      group.add(core);

      // Inner colored glow
      const innerGlowGeometry = new THREE.SphereGeometry(size, 32, 32);
      const innerGlowMaterial = new THREE.MeshBasicMaterial({
        color: color,
        transparent: true,
        opacity: 0.6,
      });
      const innerGlow = new THREE.Mesh(innerGlowGeometry, innerGlowMaterial);
      innerGlow.name = 'innerGlow';
      group.add(innerGlow);

      // Outer atmospheric glow
      const outerGlowGeometry = new THREE.SphereGeometry(size * 2, 32, 32);
      const outerGlowMaterial = new THREE.MeshBasicMaterial({
        color: color,
        transparent: true,
        opacity: 0.15,
      });
      const outerGlow = new THREE.Mesh(outerGlowGeometry, outerGlowMaterial);
      outerGlow.name = 'outerGlow';
      group.add(outerGlow);

      // Star rays (cross pattern)
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

      // ===== HOVER HALO - Sprite that always faces camera =====
      const haloCanvas = createHaloCanvas(color, size * 4);
      const haloTexture = new THREE.CanvasTexture(haloCanvas);
      haloTexture.needsUpdate = true;
      const haloMaterial = new THREE.SpriteMaterial({
        map: haloTexture,
        transparent: true,
        opacity: 0,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
        depthTest: true,
      });
      const hoverHalo = new THREE.Sprite(haloMaterial);
      hoverHalo.name = 'hoverHalo';
      hoverHalo.scale.setScalar(size * 8);
      hoverHalo.renderOrder = 999; // Render on top
      group.add(hoverHalo);

      // ===== BIRTH ANIMATION ELEMENTS =====
      if (isNewborn) {
        createBirthElements(group, size, color);
      }

      // Ambient twinkle animation
      const twinkle = () => {
        if (!globalGroupsMap.has(node.id)) return;
        
        const time = Date.now() * 0.003;
        const twinkleValue = 0.7 + Math.sin(time + (node.id.codePointAt(0) || 0)) * 0.3;
        coreMaterial.opacity = twinkleValue * 0.95;
        innerGlowMaterial.opacity = 0.6 * twinkleValue;
        
        requestAnimationFrame(twinkle);
      };
      twinkle();

      return group;
    },
    // Empty dependencies - callback is stable, groups are reused from global map
    []
  );

  // Cleanup on unmount - DON'T clear global maps, they persist across remounts
  useEffect(() => {
    return () => {
      // Only cancel animation frame, don't clear the maps
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  return nodeThreeObject;
}

// Cache for birth effect textures
const birthTextureCache = new Map<string, THREE.Texture>();

// Create a radial glow texture for the core flash (always faces camera)
function createGlowTexture(color: string): THREE.Texture {
  const cacheKey = `glow-${color}`;
  if (birthTextureCache.has(cacheKey)) {
    return birthTextureCache.get(cacheKey)!;
  }

  const canvas = document.createElement('canvas');
  const resolution = 512; // Higher resolution for larger scales
  canvas.width = resolution;
  canvas.height = resolution;
  const ctx = canvas.getContext('2d', { alpha: true })!;
  
  const centerX = resolution / 2;
  const centerY = resolution / 2;
  const radius = resolution / 2 - 20; // More margin from edges

  ctx.clearRect(0, 0, resolution, resolution);

  // Create intense core glow with smoother falloff
  const gradient = ctx.createRadialGradient(centerX, centerY, 0, centerX, centerY, radius);
  gradient.addColorStop(0, '#ffffff');
  gradient.addColorStop(0.08, '#ffffff');
  gradient.addColorStop(0.2, color + 'FF');
  gradient.addColorStop(0.35, color + 'CC');
  gradient.addColorStop(0.5, color + '88');
  gradient.addColorStop(0.65, color + '44');
  gradient.addColorStop(0.8, color + '18');
  gradient.addColorStop(0.9, color + '08');
  gradient.addColorStop(1, 'rgba(0,0,0,0)');

  ctx.beginPath();
  ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
  ctx.fillStyle = gradient;
  ctx.fill();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  birthTextureCache.set(cacheKey, texture);
  return texture;
}

// Create expanding shockwave ring texture
function createShockwaveTexture(color: string): THREE.Texture {
  const cacheKey = `shockwave-${color}`;
  if (birthTextureCache.has(cacheKey)) {
    return birthTextureCache.get(cacheKey)!;
  }

  const canvas = document.createElement('canvas');
  const resolution = 512; // Higher resolution
  canvas.width = resolution;
  canvas.height = resolution;
  const ctx = canvas.getContext('2d', { alpha: true })!;
  
  const centerX = resolution / 2;
  const centerY = resolution / 2;
  const outerRadius = resolution / 2 - 40; // More margin
  const innerRadius = outerRadius * 0.75;

  ctx.clearRect(0, 0, resolution, resolution);

  // Soft outer glow with smoother falloff
  const glowGradient = ctx.createRadialGradient(
    centerX, centerY, innerRadius * 0.6,
    centerX, centerY, outerRadius
  );
  glowGradient.addColorStop(0, 'rgba(0,0,0,0)');
  glowGradient.addColorStop(0.4, 'rgba(0,0,0,0)');
  glowGradient.addColorStop(0.55, color + '15');
  glowGradient.addColorStop(0.7, color + '50');
  glowGradient.addColorStop(0.8, color + '40');
  glowGradient.addColorStop(0.9, color + '15');
  glowGradient.addColorStop(1, 'rgba(0,0,0,0)');

  ctx.beginPath();
  ctx.arc(centerX, centerY, outerRadius, 0, Math.PI * 2);
  ctx.fillStyle = glowGradient;
  ctx.fill();

  // Main ring with soft edges
  const ringRadius = (innerRadius + outerRadius) / 2;
  const ringWidth = (outerRadius - innerRadius) * 0.35;
  
  ctx.beginPath();
  ctx.arc(centerX, centerY, ringRadius, 0, Math.PI * 2);
  ctx.strokeStyle = color + 'AA';
  ctx.lineWidth = ringWidth;
  ctx.stroke();

  // Inner bright edge (softer)
  ctx.beginPath();
  ctx.arc(centerX, centerY, innerRadius + ringWidth * 0.3, 0, Math.PI * 2);
  ctx.strokeStyle = '#ffffffCC';
  ctx.lineWidth = 4;
  ctx.stroke();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  birthTextureCache.set(cacheKey, texture);
  return texture;
}

// Create star burst texture with rays
function createStarburstTexture(color: string): THREE.Texture {
  const cacheKey = `starburst-${color}`;
  if (birthTextureCache.has(cacheKey)) {
    return birthTextureCache.get(cacheKey)!;
  }

  const canvas = document.createElement('canvas');
  const resolution = 512; // Higher resolution
  canvas.width = resolution;
  canvas.height = resolution;
  const ctx = canvas.getContext('2d', { alpha: true })!;
  
  const centerX = resolution / 2;
  const centerY = resolution / 2;

  ctx.clearRect(0, 0, resolution, resolution);

  // Draw star rays - wider and with softer edges
  const rayCount = 8;
  for (let i = 0; i < rayCount; i++) {
    const angle = (i / rayCount) * Math.PI * 2;
    const rayLength = resolution / 2 - 50; // More margin
    
    const gradient = ctx.createLinearGradient(
      centerX, centerY,
      centerX + Math.cos(angle) * rayLength,
      centerY + Math.sin(angle) * rayLength
    );
    gradient.addColorStop(0, '#ffffffEE');
    gradient.addColorStop(0.15, color + 'DD');
    gradient.addColorStop(0.4, color + '88');
    gradient.addColorStop(0.7, color + '33');
    gradient.addColorStop(0.9, color + '0A');
    gradient.addColorStop(1, 'rgba(0,0,0,0)');

    ctx.beginPath();
    ctx.moveTo(centerX, centerY);
    ctx.lineTo(
      centerX + Math.cos(angle - 0.12) * rayLength,
      centerY + Math.sin(angle - 0.12) * rayLength
    );
    ctx.lineTo(
      centerX + Math.cos(angle + 0.12) * rayLength,
      centerY + Math.sin(angle + 0.12) * rayLength
    );
    ctx.closePath();
    ctx.fillStyle = gradient;
    ctx.fill();
  }

  // Secondary shorter rays
  for (let i = 0; i < rayCount; i++) {
    const angle = (i / rayCount) * Math.PI * 2 + Math.PI / rayCount;
    const rayLength = resolution / 3 - 20;
    
    const gradient = ctx.createLinearGradient(
      centerX, centerY,
      centerX + Math.cos(angle) * rayLength,
      centerY + Math.sin(angle) * rayLength
    );
    gradient.addColorStop(0, color + 'BB');
    gradient.addColorStop(0.4, color + '55');
    gradient.addColorStop(0.8, color + '15');
    gradient.addColorStop(1, 'rgba(0,0,0,0)');

    ctx.beginPath();
    ctx.moveTo(centerX, centerY);
    ctx.lineTo(
      centerX + Math.cos(angle - 0.08) * rayLength,
      centerY + Math.sin(angle - 0.08) * rayLength
    );
    ctx.lineTo(
      centerX + Math.cos(angle + 0.08) * rayLength,
      centerY + Math.sin(angle + 0.08) * rayLength
    );
    ctx.closePath();
    ctx.fillStyle = gradient;
    ctx.fill();
  }

  // Central glow - larger and softer
  const coreGradient = ctx.createRadialGradient(centerX, centerY, 0, centerX, centerY, 60);
  coreGradient.addColorStop(0, '#ffffff');
  coreGradient.addColorStop(0.3, color + 'CC');
  coreGradient.addColorStop(0.6, color + '55');
  coreGradient.addColorStop(1, 'rgba(0,0,0,0)');
  ctx.beginPath();
  ctx.arc(centerX, centerY, 60, 0, Math.PI * 2);
  ctx.fillStyle = coreGradient;
  ctx.fill();

  const texture = new THREE.CanvasTexture(canvas);
  texture.needsUpdate = true;
  birthTextureCache.set(cacheKey, texture);
  return texture;
}

// Helper function to create birth animation elements
function createBirthElements(group: THREE.Group, size: number, color: string) {
  // Core flash - intense white/colored glow (Sprite - always faces camera)
  const glowTexture = createGlowTexture(color);
  const coreFlashMaterial = new THREE.SpriteMaterial({
    map: glowTexture,
    transparent: true,
    opacity: 1,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const coreFlash = new THREE.Sprite(coreFlashMaterial);
  coreFlash.name = 'coreFlash';
  coreFlash.scale.setScalar(size * 12); // Larger initial size
  group.add(coreFlash);

  // Starburst rays (Sprite - always faces camera)
  const starburstTexture = createStarburstTexture(color);
  const starburstMaterial = new THREE.SpriteMaterial({
    map: starburstTexture,
    transparent: true,
    opacity: 1,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
  });
  const starburst = new THREE.Sprite(starburstMaterial);
  starburst.name = 'supernovaRing'; // Reuse the name for animation
  starburst.scale.setScalar(size * 4); // Larger initial rays
  group.add(starburst);

  // Shockwave rings (Sprites - always face camera)
  for (let i = 0; i < 3; i++) {
    const shockwaveTexture = createShockwaveTexture(color);
    const shockwaveMaterial = new THREE.SpriteMaterial({
      map: shockwaveTexture,
      transparent: true,
      opacity: 0.85,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    const shockwave = new THREE.Sprite(shockwaveMaterial);
    shockwave.name = `shockwave${i}`;
    shockwave.scale.setScalar(size * 0.5); // Larger starting size
    group.add(shockwave);
  }

  // Particle burst - stardust explosion with colors
  const particleCount = 150;
  const particleGeometry = new THREE.BufferGeometry();
  const positions = new Float32Array(particleCount * 3);
  const colors = new Float32Array(particleCount * 3);
  const sizes = new Float32Array(particleCount);
  const velocities = new Float32Array(particleCount * 3);
  
  // Parse the color to RGB
  const tempColor = new THREE.Color(color);
  const whiteColor = new THREE.Color('#ffffff');
  
  for (let i = 0; i < particleCount; i++) {
    positions[i * 3] = 0;
    positions[i * 3 + 1] = 0;
    positions[i * 3 + 2] = 0;
    
    // Random spherical direction
    const theta = Math.random() * Math.PI * 2;
    const phi = Math.acos(2 * Math.random() - 1);
    const speed = 0.3 + Math.random() * 2;
    
    velocities[i * 3] = Math.sin(phi) * Math.cos(theta) * speed;
    velocities[i * 3 + 1] = Math.sin(phi) * Math.sin(theta) * speed;
    velocities[i * 3 + 2] = Math.cos(phi) * speed;
    
    // Mix between white and the node color
    const mixFactor = Math.random();
    const particleColor = whiteColor.clone().lerp(tempColor, mixFactor);
    colors[i * 3] = particleColor.r;
    colors[i * 3 + 1] = particleColor.g;
    colors[i * 3 + 2] = particleColor.b;
    
    // Random sizes
    sizes[i] = size * (0.2 + Math.random() * 0.4);
  }
  
  particleGeometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
  particleGeometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
  particleGeometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));
  
  const particleMaterial = new THREE.PointsMaterial({
    size: size * 0.4,
    transparent: true,
    opacity: 1,
    vertexColors: true,
    blending: THREE.AdditiveBlending,
    depthWrite: false,
    sizeAttenuation: true,
  });
  
  const particles = new THREE.Points(particleGeometry, particleMaterial);
  particles.name = 'birthParticles';
  particles.userData = { velocities };
  group.add(particles);
}

// Helper function to cleanup birth elements
function cleanupBirthElements(group: THREE.Group) {
  const birthElementNames = [
    'supernovaRing', 'shockwave0', 'shockwave1', 'shockwave2', 
    'birthParticles', 'coreFlash'
  ];
  
  birthElementNames.forEach(name => {
    const obj = group.getObjectByName(name);
    if (obj) {
      group.remove(obj);
      disposeObject(obj);
    }
  });
}

// Helper function to dispose a single object
function disposeObject(obj: THREE.Object3D) {
  if (obj instanceof THREE.Mesh) {
    obj.geometry.dispose();
    if (Array.isArray(obj.material)) {
      obj.material.forEach(m => m.dispose());
    } else {
      obj.material.dispose();
    }
  } else if (obj instanceof THREE.Points) {
    obj.geometry.dispose();
    (obj.material as THREE.Material).dispose();
  } else if (obj instanceof THREE.Group) {
    obj.children.forEach(child => disposeObject(child));
  }
}

// Helper function to dispose entire group
function disposeGroup(group: THREE.Group) {
  group.traverse((child) => {
    if (child instanceof THREE.Mesh) {
      child.geometry.dispose();
      if (Array.isArray(child.material)) {
        child.material.forEach(m => m.dispose());
      } else {
        child.material.dispose();
      }
    } else if (child instanceof THREE.Points) {
      child.geometry.dispose();
      (child.material as THREE.Material).dispose();
    }
  });
}
