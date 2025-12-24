import { useEffect, useRef } from 'react';

interface Star {
  // 3D position on a sphere
  x: number;
  y: number;
  z: number;
  size: number;
  baseOpacity: number;
  color: string;
  twinkleSpeed: number;
  twinkleOffset: number;
}

interface StarFieldProps {
  readonly starCount?: number;
  readonly graphRef?: React.RefObject<any>;
}

const STAR_COLORS = [
  'rgba(255, 255, 255, 1)',
  'rgba(220, 230, 255, 1)',
  'rgba(255, 250, 240, 1)',
  'rgba(200, 215, 255, 1)',
  'rgba(255, 230, 220, 1)',
];

export default function StarField({ starCount = 5000, graphRef }: StarFieldProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const starsRef = useRef<Star[]>([]);
  const animationRef = useRef<number>();

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    let canvasWidth = 0;
    let canvasHeight = 0;

    const resizeCanvas = () => {
      const dpr = window.devicePixelRatio || 1;
      const rect = canvas.getBoundingClientRect();
      canvas.width = rect.width * dpr;
      canvas.height = rect.height * dpr;
      ctx.setTransform(1, 0, 0, 1, 0, 0);
      ctx.scale(dpr, dpr);
      canvasWidth = rect.width;
      canvasHeight = rect.height;
    };

    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);

    // Generate stars on a unit sphere
    const generateStars = () => {
      const stars: Star[] = [];

      for (let i = 0; i < starCount; i++) {
        // Uniform distribution on sphere
        const theta = Math.random() * Math.PI * 2;
        const phi = Math.acos(2 * Math.random() - 1);
        
        // Convert to Cartesian on unit sphere
        const x = Math.sin(phi) * Math.cos(theta);
        const y = Math.sin(phi) * Math.sin(theta);
        const z = Math.cos(phi);
        
        const sizeRandom = Math.random();
        let size: number;
        if (sizeRandom < 0.75) {
          size = 0.3 + Math.random() * 0.4;
        } else if (sizeRandom < 0.95) {
          size = 0.6 + Math.random() * 0.4;
        } else {
          size = 0.9 + Math.random() * 0.5;
        }

        stars.push({
          x, y, z,
          size,
          baseOpacity: 0.5 + Math.random() * 0.5,
          color: STAR_COLORS[Math.floor(Math.random() * STAR_COLORS.length)],
          twinkleSpeed: 0.3 + Math.random() * 1.5,
          twinkleOffset: Math.random() * Math.PI * 2,
        });
      }

      return stars;
    };

    starsRef.current = generateStars();

    const animate = (time: number) => {
      if (canvasWidth === 0 || canvasHeight === 0) {
        animationRef.current = requestAnimationFrame(animate);
        return;
      }

      const centerX = canvasWidth / 2;
      const centerY = canvasHeight / 2;

      ctx.clearRect(0, 0, canvasWidth, canvasHeight);

      // Get camera rotation from graph - read fresh every frame
      let cameraYaw = 0;   // Horizontal rotation
      let cameraPitch = 0; // Vertical rotation
      let cameraDistance = 1500; // For zoom effect
      
      if (graphRef?.current) {
        try {
          const cameraPos = graphRef.current.cameraPosition();
          if (cameraPos) {
            // Horizontal angle (yaw)
            cameraYaw = Math.atan2(cameraPos.x, cameraPos.z);
            // Vertical angle (pitch)
            const horizontalDist = Math.hypot(cameraPos.x, cameraPos.z);
            cameraPitch = Math.atan2(cameraPos.y, horizontalDist);
            // Distance from center (for zoom)
            cameraDistance = Math.hypot(cameraPos.x, cameraPos.y, cameraPos.z);
          }
        } catch {
          // Keep defaults
        }
      }

      // Precompute rotation matrices
      // Stars are fixed in space - we transform them based on camera view direction
      // Camera looks FROM cameraPos TO origin, so we rotate stars by camera angle
      // This makes stars and graph move together visually when camera orbits
      const cosYaw = Math.cos(cameraYaw);
      const sinYaw = Math.sin(cameraYaw);
      const cosPitch = Math.cos(cameraPitch);
      const sinPitch = Math.sin(cameraPitch);

      // Field of view scale - adjusts with zoom
      const baseFovScale = Math.min(canvasWidth, canvasHeight) * 2;
      const zoomFactor = 1500 / Math.max(cameraDistance, 100);
      const fovScale = baseFovScale * Math.pow(zoomFactor, 0.3);

      // Draw stars
      starsRef.current.forEach((star) => {
        // Apply yaw rotation (around Y axis)
        const x1 = star.x * cosYaw - star.z * sinYaw;
        const y1 = star.y;
        const z1 = star.x * sinYaw + star.z * cosYaw;

        // Apply pitch rotation (around X axis)
        const x2 = x1;
        const y2 = y1 * cosPitch - z1 * sinPitch;
        const z2 = y1 * sinPitch + z1 * cosPitch;

        // Project to screen (stars are on unit sphere, we're at center looking at -Z)
        // Only show stars in front hemisphere (z2 < 0 means in front when looking at -Z)
        // But we want to show all stars, so we use a hemisphere projection
        
        // Use stereographic-like projection for full sky view
        const projectionDivisor = 1 - z2 * 0.5; // Softer projection, shows more of the sphere
        const screenX = centerX + (x2 / projectionDivisor) * fovScale;
        const screenY = centerY - (y2 / projectionDivisor) * fovScale;

        // Skip if outside canvas
        if (screenX < -5 || screenX > canvasWidth + 5 || 
            screenY < -5 || screenY > canvasHeight + 5) {
          return;
        }

        // Calculate distance from screen center for edge fade effect
        const distX = (screenX - centerX) / centerX;
        const distY = (screenY - centerY) / centerY;
        const distFromCenter = Math.hypot(distX, distY);
        
        // Fade: less visible near center, more at edges
        const edgeFade = Math.min(1, Math.pow(distFromCenter * 0.7, 0.8));

        // Twinkle
        const twinkle = Math.sin(time * 0.001 * star.twinkleSpeed + star.twinkleOffset);
        const currentOpacity = star.baseOpacity * (0.85 + twinkle * 0.15) * edgeFade;

        if (currentOpacity < 0.05) return;

        // Draw star
        ctx.beginPath();
        ctx.arc(screenX, screenY, star.size, 0, Math.PI * 2);
        ctx.fillStyle = star.color.replace('1)', `${currentOpacity})`);
        ctx.fill();
      });

      animationRef.current = requestAnimationFrame(animate);
    };

    animationRef.current = requestAnimationFrame(animate);

    return () => {
      window.removeEventListener('resize', resizeCanvas);
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [starCount, graphRef]);

  return (
    <canvas
      ref={canvasRef}
      className="absolute inset-0 w-full h-full pointer-events-none"
      style={{ opacity: 0.9 }}
    />
  );
}
