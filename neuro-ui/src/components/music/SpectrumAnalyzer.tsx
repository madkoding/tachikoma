import React, { useRef, useEffect, useCallback } from 'react';
import { useMusicStore } from '../../stores/musicStore';

interface SpectrumAnalyzerProps {
  barCount?: number;
  className?: string;
  showReflection?: boolean;
  ledStyle?: boolean;
  compact?: boolean;
}

/**
 * Canvas-based Spectrum Analyzer
 * Uses requestAnimationFrame to draw directly on canvas without React re-renders.
 * This eliminates ~1150 DOM elements (32 bars × 36 LEDs) that were causing performance issues.
 */
export const SpectrumAnalyzer: React.FC<SpectrumAnalyzerProps> = ({ 
  barCount = 32, 
  className = '',
  showReflection = true,
  ledStyle = true,
  compact = false,
}) => {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const animationRef = useRef<number | null>(null);
  const lastDrawTimeRef = useRef<number>(0);
  
  // LED count per bar
  const ledCount = compact ? 16 : 36;
  
  // Draw function - called via requestAnimationFrame, not React state
  const draw = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    const ctx = canvas.getContext('2d');
    if (!ctx) return;
    
    // Throttle to ~24fps for performance (every 42ms)
    const now = performance.now();
    if (now - lastDrawTimeRef.current < 42) {
      animationRef.current = requestAnimationFrame(draw);
      return;
    }
    lastDrawTimeRef.current = now;
    
    // Get current state directly (no React subscription)
    const state = useMusicStore.getState();
    const spectrumData = state.spectrumData;
    const isPlaying = state.player.isPlaying;
    
    // Get canvas dimensions
    const width = canvas.width;
    const height = canvas.height;
    
    // Clear canvas
    ctx.clearRect(0, 0, width, height);
    
    // Calculate bar dimensions
    const gap = 2;
    const barWidth = (width - (barCount - 1) * gap) / barCount;
    const ledHeight = (height * (showReflection && !compact ? 0.8 : 1)) / ledCount;
    const ledGap = 1;
    
    // Draw each bar
    for (let barIndex = 0; barIndex < barCount; barIndex++) {
      const value = spectrumData[barIndex] || 0;
      const scaledValue = Math.pow(value, 0.6) * 1.3;
      const litLeds = Math.floor(scaledValue * ledCount);
      const x = barIndex * (barWidth + gap);
      
      // Draw LEDs from bottom to top
      for (let ledIndex = 0; ledIndex < ledCount; ledIndex++) {
        const isLit = ledIndex < litLeds;
        const y = height * (showReflection && !compact ? 0.8 : 1) - (ledIndex + 1) * ledHeight;
        
        if (isLit && isPlaying) {
          // Calculate color based on position
          const levelRatio = ledIndex / ledCount;
          
          if (levelRatio > 0.8) {
            // Peak - magenta
            ctx.fillStyle = `hsla(320, 100%, ${50 + value * 20}%, ${0.8 + value * 0.2})`;
          } else if (levelRatio > 0.6) {
            // Upper mid - purple  
            ctx.fillStyle = `hsla(280, 90%, ${45 + value * 20}%, ${0.8 + value * 0.2})`;
          } else {
            // Lower - cyan
            const hue = 180 + (barIndex / barCount) * 60;
            ctx.fillStyle = `hsla(${hue}, 100%, ${40 + value * 25}%, ${0.8 + value * 0.2})`;
          }
          
          // Add glow effect for lit LEDs
          if (ledIndex === litLeds - 1) {
            ctx.shadowColor = levelRatio > 0.8 ? 'rgba(255, 0, 128, 0.6)' : 
                              levelRatio > 0.6 ? 'rgba(180, 0, 255, 0.5)' : 
                              'rgba(0, 255, 255, 0.4)';
            ctx.shadowBlur = 6;
          } else {
            ctx.shadowBlur = 2;
          }
        } else {
          // Unlit LED
          ctx.fillStyle = 'rgba(30, 30, 40, 0.5)';
          ctx.shadowBlur = 0;
        }
        
        ctx.fillRect(x, y + ledGap, barWidth, ledHeight - ledGap * 2);
      }
      
      // Reset shadow for next iteration
      ctx.shadowBlur = 0;
    }
    
    // Draw reflection if enabled
    if (showReflection && !compact) {
      const reflectionHeight = height * 0.2;
      const reflectionY = height * 0.8;
      
      // Create gradient for fade effect
      const gradient = ctx.createLinearGradient(0, reflectionY, 0, height);
      gradient.addColorStop(0, 'rgba(0, 0, 0, 0.3)');
      gradient.addColorStop(1, 'rgba(0, 0, 0, 0)');
      
      ctx.save();
      ctx.globalAlpha = 0.2;
      ctx.translate(0, height);
      ctx.scale(1, -1);
      
      // Draw simplified reflection (just 3 LEDs worth)
      for (let barIndex = 0; barIndex < barCount; barIndex++) {
        const value = spectrumData[barIndex] || 0;
        const scaledValue = Math.pow(value, 0.6) * 1.3;
        const litLeds = Math.min(3, Math.floor(scaledValue * ledCount));
        const x = barIndex * (barWidth + gap);
        
        for (let ledIndex = 0; ledIndex < 3; ledIndex++) {
          if (ledIndex < litLeds && isPlaying) {
            const hue = 180 + (barIndex / barCount) * 60;
            ctx.fillStyle = `hsla(${hue}, 100%, 50%, 0.5)`;
          } else {
            continue;
          }
          
          const y = height - reflectionHeight + ledIndex * (reflectionHeight / 3);
          ctx.fillRect(x, y, barWidth, reflectionHeight / 3 - 1);
        }
      }
      
      ctx.restore();
    }
    
    // Continue animation loop
    animationRef.current = requestAnimationFrame(draw);
  }, [barCount, ledCount, compact, showReflection]);
  
  // Setup canvas and start animation
  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;
    
    // Set canvas size based on container
    const resizeObserver = new ResizeObserver((entries) => {
      for (const entry of entries) {
        const { width, height } = entry.contentRect;
        // Use device pixel ratio for sharp rendering
        const dpr = window.devicePixelRatio || 1;
        canvas.width = width * dpr;
        canvas.height = height * dpr;
        canvas.style.width = `${width}px`;
        canvas.style.height = `${height}px`;
        
        const ctx = canvas.getContext('2d');
        if (ctx) {
          ctx.scale(dpr, dpr);
        }
      }
    });
    
    resizeObserver.observe(canvas.parentElement || canvas);
    
    // Start animation loop
    animationRef.current = requestAnimationFrame(draw);
    
    return () => {
      resizeObserver.disconnect();
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, [draw]);
  
  return (
    <div className={`relative ${className}`} style={{ minHeight: compact ? '60px' : '100px' }}>
      <canvas 
        ref={canvasRef}
        className="w-full h-full"
        style={{ display: 'block' }}
      />
      
      {/* Scanline effect overlay */}
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
};

export default SpectrumAnalyzer;
