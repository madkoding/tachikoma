import React, { useState, useRef, useEffect, useCallback } from 'react';
import { Play, Pause, SkipForward, SkipBack, GripHorizontal, Music } from 'lucide-react';
import { useMusicStore, formatDuration } from '../../stores/musicStore';
import { useLocation, useNavigate } from 'react-router-dom';
import { SpectrumAnalyzer } from './SpectrumAnalyzer';

interface MiniPlayerProps {
  onClose?: () => void;
}

export const MiniPlayer: React.FC<MiniPlayerProps> = () => {
  const location = useLocation();
  const navigate = useNavigate();
  const { player, togglePlay, nextSong, previousSong, seek } = useMusicStore();
  const mobileProgressRef = useRef<HTMLDivElement>(null);
  
  // Check if mobile
  const [isMobile, setIsMobile] = useState(() => {
    if (typeof window !== 'undefined') {
      return window.innerWidth < 640;
    }
    return false;
  });

  // Detect if keyboard is open (input focused on mobile)
  const [isKeyboardOpen, setIsKeyboardOpen] = useState(false);

  // Update isMobile on resize
  useEffect(() => {
    const handleResize = () => {
      setIsMobile(window.innerWidth < 640);
    };
    window.addEventListener('resize', handleResize);
    return () => window.removeEventListener('resize', handleResize);
  }, []);

  // Detect keyboard open/close on mobile via visualViewport and focus events
  useEffect(() => {
    if (!isMobile) return;

    const handleFocusIn = (e: FocusEvent) => {
      const target = e.target as HTMLElement;
      if (target.tagName === 'INPUT' || target.tagName === 'TEXTAREA' || target.isContentEditable) {
        setIsKeyboardOpen(true);
      }
    };

    const handleFocusOut = () => {
      // Small delay to avoid flickering when switching between inputs
      setTimeout(() => {
        const activeElement = document.activeElement as HTMLElement;
        if (!activeElement || 
            (activeElement.tagName !== 'INPUT' && 
             activeElement.tagName !== 'TEXTAREA' && 
             !activeElement.isContentEditable)) {
          setIsKeyboardOpen(false);
        }
      }, 100);
    };

    document.addEventListener('focusin', handleFocusIn);
    document.addEventListener('focusout', handleFocusOut);

    return () => {
      document.removeEventListener('focusin', handleFocusIn);
      document.removeEventListener('focusout', handleFocusOut);
    };
  }, [isMobile]);
  
  // Draggable state (only for desktop)
  const [position, setPosition] = useState({ x: 0, y: 0 });
  const [isDragging, setIsDragging] = useState(false);
  const dragStartRef = useRef({ x: 0, y: 0, posX: 0, posY: 0 });

  // Initialize position from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('miniPlayerPosition');
    if (saved) {
      try {
        const pos = JSON.parse(saved);
        setPosition(pos);
      } catch {
        // Use default position
      }
    }
  }, []);

  // Save position to localStorage
  useEffect(() => {
    if (position.x !== 0 || position.y !== 0) {
      localStorage.setItem('miniPlayerPosition', JSON.stringify(position));
    }
  }, [position]);

  const handleMouseDown = useCallback((e: React.MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    setIsDragging(true);
    dragStartRef.current = {
      x: e.clientX,
      y: e.clientY,
      posX: position.x,
      posY: position.y,
    };
  }, [position]);

  const handleMouseMove = useCallback((e: MouseEvent) => {
    if (!isDragging) return;
    
    const deltaX = e.clientX - dragStartRef.current.x;
    const deltaY = e.clientY - dragStartRef.current.y;
    
    setPosition({
      x: dragStartRef.current.posX + deltaX,
      y: dragStartRef.current.posY + deltaY,
    });
  }, [isDragging]);

  const handleMouseUp = useCallback(() => {
    setIsDragging(false);
  }, []);

  useEffect(() => {
    if (isDragging) {
      window.addEventListener('mousemove', handleMouseMove);
      window.addEventListener('mouseup', handleMouseUp);
      return () => {
        window.removeEventListener('mousemove', handleMouseMove);
        window.removeEventListener('mouseup', handleMouseUp);
      };
    }
  }, [isDragging, handleMouseMove, handleMouseUp]);

  // Don't show on music page
  if (location.pathname === '/music') return null;
  
  // Don't show if nothing is playing
  if (!player.currentSong) return null;

  // Hide on mobile when keyboard is open
  if (isMobile && isKeyboardOpen) return null;

  const progress = player.duration > 0 ? (player.currentTime / player.duration) * 100 : 0;

  // Handle progress bar click for mobile
  const handleMobileProgressClick = (e: React.MouseEvent<HTMLDivElement> | React.TouchEvent<HTMLDivElement>) => {
    if (!mobileProgressRef.current || !player.duration || player.duration <= 0) return;
    
    const rect = mobileProgressRef.current.getBoundingClientRect();
    const clientX = 'touches' in e ? e.touches[0].clientX : e.clientX;
    const percent = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const newTime = percent * player.duration;
    
    if (Number.isFinite(newTime) && newTime >= 0) {
      seek(newTime);
    }
  };

  // Mobile version - fixed bottom bar
  if (isMobile) {
    return (
      <div className="fixed bottom-0 left-0 right-0 z-50 bg-gray-900/95 backdrop-blur-xl border-t border-cyber-cyan/30 safe-area-inset-bottom">
        {/* Progress bar at top - clickable */}
        <div 
          ref={mobileProgressRef}
          onClick={handleMobileProgressClick}
          onTouchStart={handleMobileProgressClick}
          className="h-2 bg-gray-800 relative cursor-pointer active:h-3 transition-all"
        >
          <div 
            className="h-full bg-gradient-to-r from-cyber-cyan to-cyber-purple transition-all duration-200"
            style={{ width: `${progress}%` }}
          />
          {/* Knob indicator */}
          <div 
            className="absolute top-1/2 -translate-y-1/2 w-3 h-3 bg-white rounded-full shadow-lg"
            style={{ left: `calc(${progress}% - 6px)` }}
          />
        </div>
        
        <div className="px-2 py-1.5 flex items-center gap-2">
          {/* Album art */}
          <button 
            type="button"
            className="w-12 h-12 bg-gray-800 overflow-hidden flex-shrink-0 cursor-pointer relative rounded"
            onClick={() => navigate('/music')}
            aria-label="Ir al reproductor de música"
          >
            {player.currentSong.cover_url || player.currentSong.thumbnail_url ? (
              <img
                src={player.currentSong.cover_url || player.currentSong.thumbnail_url}
                alt={player.currentSong.title}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyber-cyan/20 to-cyber-purple/20">
                <Music className="w-5 h-5 text-gray-600" />
              </div>
            )}
          </button>

          {/* Song info */}
          <button 
            type="button"
            className="flex-1 min-w-0 cursor-pointer text-left"
            onClick={() => navigate('/music')}
            aria-label={`Reproduciendo: ${player.currentSong.title}`}
          >
            <div className="font-medium text-white text-xs truncate font-cyber leading-tight">
              {player.currentSong.title}
            </div>
            <div className="text-[9px] text-gray-400 truncate font-mono leading-tight">
              {player.currentSong.artist || 'Artista desconocido'}
            </div>
          </button>

          {/* Controls with time */}
          <div className="flex flex-col items-center">
            <span className="led-time text-[8px] leading-none mt-1 mb-2.5">
              {formatDuration(player.currentTime)}/{formatDuration(player.duration)}
            </span>
            <div className="flex items-center">
              <button
                type="button"
                onClick={previousSong}
                className="p-1.5 text-gray-400 active:text-white transition-all"
                aria-label="Canción anterior"
              >
                <SkipBack className="w-4 h-4" />
              </button>
              
              <button
                type="button"
                onClick={togglePlay}
                className="p-2 bg-cyber-cyan text-black active:bg-cyber-cyan/80 transition-all rounded-full mx-0.5"
                aria-label={player.isPlaying ? 'Pausar' : 'Reproducir'}
              >
                {player.isPlaying ? (
                  <Pause className="w-4 h-4" />
                ) : (
                  <Play className="w-4 h-4 ml-0.5" />
                )}
              </button>
              
              <button
                type="button"
                onClick={nextSong}
                className="p-1.5 text-gray-400 active:text-white transition-all"
                aria-label="Siguiente canción"
              >
                <SkipForward className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    );
  }

  // Desktop version - floating draggable
  return (
    <div 
      className="fixed z-50 animate-in slide-in-from-bottom-5 duration-300"
      style={{ 
        bottom: `${16 - position.y}px`, 
        right: `${16 - position.x}px`,
        userSelect: isDragging ? 'none' : 'auto',
      }}
    >
      <div className="bg-gray-900/95 backdrop-blur-xl border border-cyber-cyan/30 shadow-2xl shadow-cyber-cyan/20 overflow-hidden w-80 relative">
        {/* Spectrum Background - Blurred */}
        <div className="absolute inset-0 overflow-hidden opacity-30">
          <div className="absolute inset-0 blur-md">
            <SpectrumAnalyzer 
              barCount={16} 
              compact 
              showReflection={false}
              className="h-full w-full"
            />
          </div>
        </div>
        
        {/* Progress bar at top */}
        <div className="h-1 bg-gray-800 relative z-10">
          <div 
            className="h-full bg-gradient-to-r from-cyber-cyan to-cyber-purple transition-all duration-200"
            style={{ width: `${progress}%` }}
          />
        </div>
        
        <div className="p-3 relative z-10">
          <div className="flex items-center gap-3">
            {/* Drag handle */}
            <div
              className="p-1 text-gray-500 hover:text-cyber-cyan cursor-grab active:cursor-grabbing transition-colors"
              onMouseDown={handleMouseDown}
              aria-label="Arrastrar reproductor"
            >
              <GripHorizontal className="w-4 h-4" />
            </div>
            
            {/* Album art */}
            <button 
              type="button"
              className="w-10 h-10 bg-gray-800 overflow-hidden flex-shrink-0 cursor-pointer hover:ring-2 hover:ring-cyber-cyan/50 transition-all relative"
              onClick={() => navigate('/music')}
              aria-label="Ir al reproductor de música"
            >
              {player.currentSong.cover_url || player.currentSong.thumbnail_url ? (
                <img
                  src={player.currentSong.cover_url || player.currentSong.thumbnail_url}
                  alt={player.currentSong.title}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyber-cyan/20 to-cyber-purple/20">
                  <Music className="w-4 h-4 text-gray-600" />
                </div>
              )}
            </button>

            {/* Song info */}
            <button 
              type="button"
              className="flex-1 min-w-0 cursor-pointer text-left"
              onClick={() => navigate('/music')}
              aria-label={`Reproduciendo: ${player.currentSong.title}`}
            >
              <div className="font-medium text-white text-sm truncate font-cyber overflow-hidden">
                <span className="inline-block animate-marquee">{player.currentSong.title}</span>
              </div>
              <div className="text-xs text-gray-400 truncate font-mono">
                {player.currentSong.artist || 'Artista desconocido'}
              </div>
              <div className="text-xs led-time mt-0.5">
                {formatDuration(player.currentTime)} / {formatDuration(player.duration)}
              </div>
            </button>

            {/* Controls */}
            <div className="flex items-center gap-1">
              <button
                type="button"
                onClick={togglePlay}
                className="p-2 bg-cyber-cyan text-black hover:bg-cyber-cyan/80 transition-all"
                aria-label={player.isPlaying ? 'Pausar' : 'Reproducir'}
              >
                {player.isPlaying ? (
                  <Pause className="w-4 h-4" />
                ) : (
                  <Play className="w-4 h-4 ml-0.5" />
                )}
              </button>
              
              <button
                type="button"
                onClick={nextSong}
                className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 transition-all"
                aria-label="Siguiente canción"
              >
                <SkipForward className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
};

export default MiniPlayer;
