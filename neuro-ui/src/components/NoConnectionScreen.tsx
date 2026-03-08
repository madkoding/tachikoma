import { useEffect, useRef, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { WifiOff, RefreshCw, Server, Database } from 'lucide-react';

interface NoConnectionScreenProps {
  error?: string | null;
  onRetry?: () => void;
  isChecking?: boolean;
}

// Pre-generated noise tile size (small for memory efficiency)
const NOISE_TILE_SIZE = 128;
// Target FPS for noise animation (24 FPS for smooth film-like effect)
const TARGET_FPS = 24;
const FRAME_INTERVAL = 1000 / TARGET_FPS;

export function NoConnectionScreen({ error, onRetry, isChecking }: NoConnectionScreenProps) {
  const { t } = useTranslation();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();
  const lastFrameTimeRef = useRef<number>(0);
  // Pre-generated noise patterns for tiling
  const noisePatternsRef = useRef<ImageData[]>([]);
  const currentPatternRef = useRef<number>(0);
  const glitchTimeoutRef = useRef<number>();

  // Generate multiple noise tiles at init (done once)
  const generateNoisePatterns = useCallback((ctx: CanvasRenderingContext2D, count: number = 4) => {
    const patterns: ImageData[] = [];
    for (let p = 0; p < count; p++) {
      const imageData = ctx.createImageData(NOISE_TILE_SIZE, NOISE_TILE_SIZE);
      const data = imageData.data;
      
      for (let i = 0; i < data.length; i += 4) {
        const noise = Math.random() * 255;
        // Grayscale with slight blue tint (like old CRT)
        data[i] = noise * 0.9;       // R
        data[i + 1] = noise * 0.95;  // G
        data[i + 2] = noise;         // B
        data[i + 3] = 180;           // Alpha
      }
      patterns.push(imageData);
    }
    return patterns;
  }, []);

  // Low-resource TV static noise effect using tiled patterns
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d', { 
      alpha: false, // Disable alpha for better performance
      desynchronized: true // Hint for lower latency
    });
    if (!ctx) return;

    // Create offscreen canvas for noise tile
    const tileCanvas = document.createElement('canvas');
    tileCanvas.width = NOISE_TILE_SIZE;
    tileCanvas.height = NOISE_TILE_SIZE;
    const tileCtx = tileCanvas.getContext('2d');
    if (!tileCtx) return;

    // Create offscreen canvas for scanlines pattern (generated once)
    const scanlineCanvas = document.createElement('canvas');
    scanlineCanvas.width = 1;
    scanlineCanvas.height = 4; // 2px line + 2px gap
    const scanlineCtx = scanlineCanvas.getContext('2d');
    if (!scanlineCtx) return;
    
    // Draw scanline pattern (dark line every 4 pixels)
    scanlineCtx.fillStyle = 'rgba(0, 0, 0, 0.35)';
    scanlineCtx.fillRect(0, 0, 1, 2);
    scanlineCtx.fillStyle = 'transparent';
    scanlineCtx.clearRect(0, 2, 1, 2);
    
    const scanlinePattern = ctx.createPattern(scanlineCanvas, 'repeat');

    // Generate noise patterns once
    noisePatternsRef.current = generateNoisePatterns(tileCtx);

    const resize = () => {
      // Use device pixel ratio for crisp rendering but cap it for performance
      const dpr = Math.min(window.devicePixelRatio || 1, 1.5);
      canvas.width = Math.floor(window.innerWidth * dpr);
      canvas.height = Math.floor(window.innerHeight * dpr);
      canvas.style.width = `${window.innerWidth}px`;
      canvas.style.height = `${window.innerHeight}px`;
      ctx.scale(dpr, dpr);
    };
    
    resize();
    window.addEventListener('resize', resize);

    // Glitch effect - runs independently at random intervals
    const scheduleGlitch = () => {
      const delay = 2000 + Math.random() * 4000; // 2-6 seconds
      glitchTimeoutRef.current = window.setTimeout(() => {
        if (canvas && ctx) {
          // Draw glitch line
          const glitchY = Math.random() * window.innerHeight;
          const glitchHeight = Math.random() * 8 + 2;
          ctx.fillStyle = 'rgba(255, 255, 255, 0.25)';
          ctx.fillRect(0, glitchY, window.innerWidth, glitchHeight);
        }
        scheduleGlitch();
      }, delay);
    };
    scheduleGlitch();

    // Offset for pattern movement
    let offsetX = 0;
    let offsetY = 0;

    const renderNoise = (timestamp: number) => {
      // Throttle to target FPS
      if (timestamp - lastFrameTimeRef.current < FRAME_INTERVAL) {
        animationRef.current = requestAnimationFrame(renderNoise);
        return;
      }
      lastFrameTimeRef.current = timestamp;

      // Cycle through pre-generated patterns
      currentPatternRef.current = (currentPatternRef.current + 1) % noisePatternsRef.current.length;
      const currentPattern = noisePatternsRef.current[currentPatternRef.current];
      
      // Draw noise tile to offscreen canvas
      tileCtx.putImageData(currentPattern, 0, 0);
      
      // Create tiled pattern from the small noise image
      const pattern = ctx.createPattern(tileCanvas, 'repeat');
      if (pattern) {
        // Move the pattern over time for visual movement
        offsetX = (offsetX + 2) % NOISE_TILE_SIZE;
        offsetY = (offsetY + 1) % NOISE_TILE_SIZE;
        
        ctx.save();
        ctx.translate(offsetX, offsetY);
        ctx.fillStyle = pattern;
        ctx.fillRect(-NOISE_TILE_SIZE, -NOISE_TILE_SIZE, window.innerWidth + NOISE_TILE_SIZE * 2, window.innerHeight + NOISE_TILE_SIZE * 2);
        ctx.restore();
      }
      
      // Draw CRT scanlines using pre-generated pattern (much faster than loop)
      if (scanlinePattern) {
        ctx.fillStyle = scanlinePattern;
        ctx.fillRect(0, 0, window.innerWidth, window.innerHeight);
      }
      
      animationRef.current = requestAnimationFrame(renderNoise);
    };

    animationRef.current = requestAnimationFrame(renderNoise);

    return () => {
      window.removeEventListener('resize', resize);
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
      if (glitchTimeoutRef.current) {
        clearTimeout(glitchTimeoutRef.current);
      }
      noisePatternsRef.current = [];
    };
  }, [generateNoisePatterns]);

  return (
    <div className="fixed inset-0 z-[9999] overflow-hidden">
      {/* TV Static Background */}
      <canvas
        ref={canvasRef}
        className="absolute inset-0 w-full h-full"
        style={{ imageRendering: 'pixelated' }}
      />
      
      {/* Vignette overlay */}
      <div 
        className="absolute inset-0 pointer-events-none"
        style={{
          background: 'radial-gradient(ellipse at center, transparent 0%, rgba(0,0,0,0.5) 100%)',
        }}
      />
      
      {/* CRT screen curvature effect */}
      <div 
        className="absolute inset-0 pointer-events-none"
        style={{
          boxShadow: 'inset 0 0 150px rgba(0,0,0,0.7)',
          borderRadius: '10px',
        }}
      />

      {/* Connection Error Modal */}
      <div className="absolute inset-0 flex items-center justify-center p-4">
        <div 
          className="bg-gray-900/95 backdrop-blur-sm border-2 border-red-500/50 rounded-lg shadow-2xl max-w-md w-full overflow-hidden animate-pulse-slow"
          style={{
            boxShadow: '0 0 60px rgba(239, 68, 68, 0.3), inset 0 0 60px rgba(0,0,0,0.5)',
          }}
        >
          {/* Header with glitch effect */}
          <div className="bg-red-600/90 px-6 py-4 flex items-center gap-3">
            <div className="relative">
              <WifiOff className="w-8 h-8 text-white animate-pulse" />
              <div className="absolute inset-0 bg-red-400 rounded-full animate-ping opacity-30" />
            </div>
            <div>
              <h2 className="text-xl font-bold text-white tracking-wide">
                {t('connection.noSignal', 'SIN SEÑAL')}
              </h2>
              <p className="text-red-200 text-sm">
                {t('connection.checkConnection', 'Verificando conexión...')}
              </p>
            </div>
          </div>
          
          {/* Content */}
          <div className="p-6 space-y-6">
            {/* Status indicators */}
            <div className="space-y-3">
              <div className="flex items-center gap-3 text-gray-300">
                <Server className="w-5 h-5 text-red-400" />
                <span>{t('connection.serverUnavailable', 'Servidor no disponible')}</span>
              </div>
              <div className="flex items-center gap-3 text-gray-300">
                <Database className="w-5 h-5 text-red-400" />
                <span>{t('connection.dbDisconnected', 'Base de datos desconectada')}</span>
              </div>
            </div>
            
            {/* Error message */}
            {error && (
              <div className="bg-red-900/30 border border-red-500/30 rounded-lg p-3">
                <p className="text-red-300 text-sm font-mono">
                  {error}
                </p>
              </div>
            )}
            
            {/* Info text */}
            <p className="text-gray-400 text-sm">
              {t('connection.tryingToReconnect', 'El sistema está intentando reconectarse automáticamente. Si el problema persiste, verifica que los servicios estén ejecutándose.')}
            </p>
            
            {/* Retry button */}
            <button
              onClick={onRetry}
              disabled={isChecking}
              className="w-full flex items-center justify-center gap-2 px-4 py-3 bg-red-600 hover:bg-red-500 disabled:bg-red-800 disabled:cursor-not-allowed text-white rounded-lg transition-all duration-200 font-medium"
            >
              <RefreshCw className={`w-5 h-5 ${isChecking ? 'animate-spin' : ''}`} />
              {isChecking 
                ? t('connection.checking', 'Verificando...')
                : t('connection.retry', 'Reintentar conexión')
              }
            </button>
          </div>
          
          {/* Footer with animated bar */}
          <div className="h-1 bg-gray-800 overflow-hidden">
            <div 
              className="h-full bg-gradient-to-r from-red-600 via-red-400 to-red-600 animate-loading-bar"
              style={{
                width: '30%',
                animation: 'loadingBar 1.5s ease-in-out infinite',
              }}
            />
          </div>
        </div>
      </div>

      {/* Scanline overlay for entire screen */}
      <div 
        className="absolute inset-0 pointer-events-none opacity-10"
        style={{
          backgroundImage: 'repeating-linear-gradient(0deg, transparent, transparent 1px, rgba(0,0,0,0.3) 1px, rgba(0,0,0,0.3) 2px)',
          backgroundSize: '100% 2px',
        }}
      />

      {/* Flicker effect */}
      <style>{`
        @keyframes loadingBar {
          0% { transform: translateX(-100%); }
          100% { transform: translateX(400%); }
        }
        
        @keyframes pulse-slow {
          0%, 100% { opacity: 1; }
          50% { opacity: 0.95; }
        }
        
        .animate-pulse-slow {
          animation: pulse-slow 2s ease-in-out infinite;
        }
      `}</style>
    </div>
  );
}
