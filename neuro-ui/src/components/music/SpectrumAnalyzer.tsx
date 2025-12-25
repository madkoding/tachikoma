import React from 'react';
import { useMusicStore } from '../../stores/musicStore';

interface SpectrumAnalyzerProps {
  barCount?: number;
  className?: string;
  showReflection?: boolean;
  ledStyle?: boolean; // LED cuadrados style
  compact?: boolean;  // Versión compacta para miniplayer
}

export const SpectrumAnalyzer: React.FC<SpectrumAnalyzerProps> = ({ 
  barCount = 32, 
  className = '',
  showReflection = true,
  ledStyle = true,
  compact = false,
}) => {
  const { spectrumData, player } = useMusicStore();

  // Use only the number of bars we need
  const bars = spectrumData.slice(0, barCount);
  
  // Number of LED segments per bar (3x más segmentos)
  const ledCount = compact ? 16 : 36;

  // Cyberpunk color gradient based on frequency and level
  const getLedColor = (barIndex: number, ledIndex: number, isLit: boolean, value: number) => {
    if (!isLit) return 'rgba(30, 30, 40, 0.5)';
    
    const hue = 180 + (barIndex / barCount) * 60; // Cyan to purple
    const levelRatio = ledIndex / ledCount;
    
    // Higher LEDs get more intense colors
    if (levelRatio > 0.8) {
      // Top LEDs - magenta/red for peaks
      return `hsla(320, 100%, ${50 + value * 20}%, ${0.8 + value * 0.2})`;
    } else if (levelRatio > 0.6) {
      // Upper mid - purple
      return `hsla(280, 90%, ${45 + value * 20}%, ${0.8 + value * 0.2})`;
    } else {
      // Lower LEDs - cyan
      return `hsla(${hue}, 100%, ${40 + value * 25}%, ${0.8 + value * 0.2})`;
    }
  };

  const getGlowColor = (barIndex: number, ledIndex: number) => {
    const levelRatio = ledIndex / ledCount;
    if (levelRatio > 0.8) {
      return 'rgba(255, 0, 128, 0.6)';
    } else if (levelRatio > 0.6) {
      return 'rgba(180, 0, 255, 0.5)';
    }
    return 'rgba(0, 255, 255, 0.4)';
  };

  if (ledStyle) {
    return (
      <div className={`relative ${className}`} style={{ minHeight: compact ? '60px' : '100px' }}>
        {/* LED Grid */}
        <div 
          className="flex items-end justify-center gap-[4px]" 
          style={{ height: '100%' }}
        >
          {bars.map((value, barIndex) => {
            // Scale value for visual effect
            const scaledValue = Math.pow(value, 0.6) * 1.3;
            const litLeds = Math.floor(scaledValue * ledCount);
            const isActive = player.isPlaying && value > 0.05;
            
            return (
              <div
                key={`bar-${barIndex}`}
                className="flex-1 flex flex-col-reverse gap-[2px]"
                style={{ maxWidth: compact ? '6px' : '10px', height: '100%' }}
              >
                {Array.from({ length: ledCount }).map((_, ledIndex) => {
                  const isLit = ledIndex < litLeds;
                  const isPeak = ledIndex === litLeds - 1 && isLit;
                  
                  return (
                    <div
                      key={`led-${barIndex}-${ledIndex}`}
                      className="w-full transition-all duration-50"
                      style={{
                        height: `${100 / ledCount}%`,
                        minHeight: compact ? '2px' : '2px',
                        background: getLedColor(barIndex, ledIndex, isLit, value),
                        boxShadow: isLit && isActive
                          ? `0 0 ${isPeak ? '6px' : '2px'} ${getGlowColor(barIndex, ledIndex)}`
                          : 'none',
                      }}
                    />
                  );
                })}
              </div>
            );
          })}
        </div>

        {/* Reflection effect for LED style */}
        {showReflection && !compact && (
          <div 
            className="absolute bottom-0 left-0 right-0 flex items-start justify-center gap-[2px] opacity-20 transform scale-y-[-1]"
            style={{
              height: '20%',
              maskImage: 'linear-gradient(to bottom, rgba(0,0,0,0.3), transparent)',
              WebkitMaskImage: 'linear-gradient(to bottom, rgba(0,0,0,0.3), transparent)',
            }}
          >
            {bars.map((value, barIndex) => {
              const scaledValue = Math.pow(value, 0.6) * 1.3;
              const litLeds = Math.min(3, Math.floor(scaledValue * ledCount));
              
              return (
                <div
                  key={`reflection-${barIndex}`}
                  className="flex-1 flex flex-col-reverse gap-[1px]"
                  style={{ maxWidth: '12px', height: '100%' }}
                >
                  {Array.from({ length: 3 }).map((_, ledIndex) => (
                    <div
                      key={`ref-led-${barIndex}-${ledIndex}`}
                      className="w-full rounded-sm"
                      style={{
                        height: '33%',
                        background: ledIndex < litLeds 
                          ? getLedColor(barIndex, ledIndex, true, value * 0.5)
                          : 'transparent',
                      }}
                    />
                  ))}
                </div>
              );
            })}
          </div>
        )}

        {/* Scanline effect */}
        {!compact && (
          <div 
            className="absolute inset-0 pointer-events-none opacity-5"
            style={{
              background: 'repeating-linear-gradient(0deg, transparent, transparent 2px, rgba(0,255,255,0.05) 2px, rgba(0,255,255,0.05) 4px)',
            }}
          />
        )}
      </div>
    );
  }

  // Original bar style (fallback)
  return (
    <div className={`relative ${className}`} style={{ minHeight: '100px' }}>
      {/* Main spectrum */}
      <div className="flex items-end justify-center gap-[2px]" style={{ height: '100%' }}>
        {bars.map((value, index) => {
          const scaledValue = Math.pow(value, 0.7) * 1.5;
          const heightPercent = Math.min(100, Math.max(5, scaledValue * 100));
          const isActive = player.isPlaying && value > 0.05;
          const hue = 180 + (index / barCount) * 60;
          
          return (
            <div
              key={`bar-${index}`}
              className="flex-1 flex items-end"
              style={{ maxWidth: '12px', height: '100%' }}
            >
              <div
                className="w-full rounded-t"
                style={{
                  height: `${heightPercent}%`,
                  minHeight: '4px',
                  transition: 'height 50ms ease-out',
                  background: `linear-gradient(to top, hsl(${hue}, 80%, 50%), hsl(${hue}, 60%, 35%))`,
                  boxShadow: isActive 
                    ? `0 0 10px hsla(${hue}, 100%, 60%, 0.5)`
                    : 'none',
                }}
              />
            </div>
          );
        })}
      </div>
    </div>
  );
};

export default SpectrumAnalyzer;
