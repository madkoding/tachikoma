import { useEffect, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { WifiOff, RefreshCw, Server, Database } from 'lucide-react';

interface NoConnectionScreenProps {
  error?: string | null;
  onRetry?: () => void;
  isChecking?: boolean;
}

export function NoConnectionScreen({ error, onRetry, isChecking }: NoConnectionScreenProps) {
  const { t } = useTranslation();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number>();

  // TV static noise effect
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const resize = () => {
      canvas.width = window.innerWidth;
      canvas.height = window.innerHeight;
    };
    
    resize();
    window.addEventListener('resize', resize);

    const renderNoise = () => {
      const imageData = ctx.createImageData(canvas.width, canvas.height);
      const data = imageData.data;
      
      for (let i = 0; i < data.length; i += 4) {
        // Generate random grayscale value with some color tint
        const noise = Math.random() * 255;
        const scanlineEffect = Math.sin(((i / 4) / canvas.width) * Math.PI * 2 + Date.now() * 0.01) * 10;
        
        // Grayscale with slight blue tint (like old CRT)
        data[i] = noise * 0.9 + scanlineEffect;     // R
        data[i + 1] = noise * 0.95 + scanlineEffect; // G
        data[i + 2] = noise + scanlineEffect;        // B
        data[i + 3] = 180; // Alpha - semi-transparent
      }
      
      ctx.putImageData(imageData, 0, 0);
      
      // Add scanlines
      ctx.fillStyle = 'rgba(0, 0, 0, 0.03)';
      for (let y = 0; y < canvas.height; y += 2) {
        ctx.fillRect(0, y, canvas.width, 1);
      }
      
      // Add occasional horizontal glitch lines
      if (Math.random() > 0.97) {
        const glitchY = Math.random() * canvas.height;
        const glitchHeight = Math.random() * 10 + 2;
        ctx.fillStyle = 'rgba(255, 255, 255, 0.3)';
        ctx.fillRect(0, glitchY, canvas.width, glitchHeight);
      }
      
      animationRef.current = requestAnimationFrame(renderNoise);
    };

    renderNoise();

    return () => {
      window.removeEventListener('resize', resize);
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

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
