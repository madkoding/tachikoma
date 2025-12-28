import React, { useRef, useState } from 'react';
import { 
  Play, 
  Pause, 
  SkipBack, 
  SkipForward, 
  Volume2, 
  VolumeX, 
  Shuffle, 
  Repeat, 
  Repeat1,
  Music,
  Heart
} from 'lucide-react';
import { useMusicStore, usePlayerState, usePlayerActions, formatDuration, RepeatMode } from '../../stores/musicStore';
import { musicApi } from '../../api/client';
import { SpectrumAnalyzer } from './SpectrumAnalyzer';
import { AnimatedLedDigits } from '../common';

interface MusicPlayerProps {
  compact?: boolean;
}

export const MusicPlayer: React.FC<MusicPlayerProps> = ({ compact = false }) => {
  const progressRef = useRef<HTMLDivElement>(null);
  const mobileProgressRef = useRef<HTMLDivElement>(null);
  
  // Use optimized selectors instead of destructuring the entire store
  const player = usePlayerState();
  const { 
    togglePlay, 
    nextSong, 
    previousSong, 
    seek, 
    setVolume, 
    toggleMute, 
    toggleShuffle 
  } = usePlayerActions();
  
  // These actions aren't in usePlayerActions, get them separately
  const { setRepeatMode, toggleSongLike } = useMusicStore();

  // State for like button loading
  const [isLiking, setIsLiking] = useState(false);

  // Progress bar click - updates store, AudioPlayer will handle the actual seek
  const handleProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!progressRef.current) return;
    
    const rect = progressRef.current.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    const newTime = percent * player.duration;
    
    seek(newTime);
  };

  // Mobile progress bar click
  const handleMobileProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!mobileProgressRef.current) return;
    
    const rect = mobileProgressRef.current.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    const newTime = percent * player.duration;
    
    seek(newTime);
  };

  // Repeat mode cycle
  const cycleRepeatMode = () => {
    const modes: RepeatMode[] = ['off', 'all', 'one'];
    const currentIndex = modes.indexOf(player.repeatMode);
    const nextMode = modes[(currentIndex + 1) % modes.length];
    setRepeatMode(nextMode);
  };

  const progress = player.duration > 0 ? (player.currentTime / player.duration) * 100 : 0;

  if (!player.currentSong && !compact) {
    return (
      <div className="h-16 sm:h-24 bg-gray-900/60 backdrop-blur-xl border-t border-cyan-500/30 flex items-center justify-center text-gray-500 px-2">
        <Music className="w-5 h-5 sm:w-6 sm:h-6 mr-2 opacity-50" />
        <span className="text-xs sm:text-sm">Selecciona una canción para reproducir</span>
      </div>
    );
  }

  if (!player.currentSong) return null;

  return (
    <div className={`
      bg-gray-900/60 backdrop-blur-xl border-t border-cyan-500/30
      ${compact ? 'p-2' : 'p-2 sm:p-4 pt-3 sm:pt-4'}
      w-full relative overflow-hidden
    `}>
      {/* Spectrum Background - Blurred */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div className="absolute inset-0 blur-2xl scale-[1.5]">
          <SpectrumAnalyzer 
            barCount={32} 
            compact 
            showReflection={false}
            className="h-full w-full"
          />
        </div>
      </div>
      
      {/* Audio element is now in AudioPlayer component (global) */}

      <div className="max-w-screen-xl mx-auto flex items-center gap-2 sm:gap-4 relative z-30">
        {/* Song Info */}
        <div className="flex items-center gap-2 sm:gap-3 min-w-0 flex-1">
          {/* Cover Art */}
          <div className="relative w-10 h-10 sm:w-14 sm:h-14 overflow-hidden bg-gray-800 flex-shrink-0">
            {player.currentSong.cover_url || player.currentSong.thumbnail_url ? (
              <img
                src={player.currentSong.cover_url || player.currentSong.thumbnail_url}
                alt={player.currentSong.title}
                className="w-full h-full object-cover"
              />
            ) : (
              <div className="w-full h-full flex items-center justify-center">
                <Music className="w-4 h-4 sm:w-6 sm:h-6 text-gray-600" />
              </div>
            )}
            
            {/* Animated glow */}
            {player.isPlaying && (
              <div className="absolute inset-0 bg-gradient-to-t from-cyan-500/20 to-transparent animate-pulse" />
            )}
          </div>

          {/* Title & Artist */}
          <div className="min-w-0 overflow-hidden">
            <div className="font-medium text-white truncate text-xs sm:text-sm">
              <span className={player.isPlaying ? 'inline-block animate-marquee' : ''}>{player.currentSong.title}</span>
            </div>
            {player.currentSong.artist && (
              <div className="text-[10px] sm:text-xs text-gray-400 truncate">
                {player.currentSong.artist}
              </div>
            )}
          </div>

          {/* Like Button */}
          <button
            onClick={async () => {
              if (!player.currentSong || isLiking) return;
              console.log('👆 MusicPlayer like button clicked for song:', player.currentSong.id, player.currentSong.title);
              setIsLiking(true);
              try {
                await toggleSongLike(player.currentSong.id);
              } catch (err) {
                console.error('Failed to toggle like:', err);
              } finally {
                setIsLiking(false);
              }
            }}
            disabled={isLiking || !player.currentSong}
            className={`p-1.5 sm:p-2 transition-all flex-shrink-0 ${
              isLiking
                ? 'text-gray-400 cursor-wait'
                : player.currentSong?.is_liked
                  ? 'text-red-500 hover:text-red-400'
                  : 'text-gray-400 hover:text-red-500'
            }`}
            title={player.currentSong?.is_liked ? 'Quitar de Me gusta' : 'Añadir a Me gusta'}
          >
            <Heart 
              className={`w-4 h-4 sm:w-5 sm:h-5 ${isLiking ? 'animate-pulse' : ''}`}
              fill={player.currentSong?.is_liked ? 'currentColor' : 'none'} 
            />
          </button>
        </div>

        {/* Center Controls */}
        <div className="flex flex-col items-center gap-1 sm:gap-2 flex-shrink-0">
          {/* Playback Controls */}
          <div className="flex items-center gap-0.5 sm:gap-2">
            {/* Shuffle - hidden on mobile */}
            <button
              onClick={toggleShuffle}
              className={`hidden sm:block p-2 transition-all hover:bg-gray-800 ${
                player.shuffle 
                  ? 'text-cyan-400 hover:text-cyan-300' 
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              <Shuffle className="w-4 h-4" />
            </button>

            {/* Previous */}
            <button
              onClick={previousSong}
              className="p-1.5 sm:p-2 text-gray-300 hover:text-white hover:bg-gray-800 transition-all"
            >
              <SkipBack className="w-4 h-4 sm:w-5 sm:h-5" />
            </button>

            {/* Play/Pause */}
            <button
              onClick={togglePlay}
              disabled={player.isLoading}
              className={`
                p-2 sm:p-3 transition-all
                ${player.isLoading 
                  ? 'bg-gray-700 cursor-wait' 
                  : 'bg-cyan-500 hover:bg-cyan-400 hover:shadow-lg hover:shadow-cyan-500/50'
                }
              `}
            >
              {player.isLoading ? (
                <div className="w-4 h-4 sm:w-5 sm:h-5 border-2 border-white/30 border-t-white rounded-full animate-spin" />
              ) : player.isPlaying ? (
                <Pause className="w-4 h-4 sm:w-5 sm:h-5 text-black" />
              ) : (
                <Play className="w-4 h-4 sm:w-5 sm:h-5 text-black ml-0.5" />
              )}
            </button>

            {/* Next */}
            <button
              onClick={nextSong}
              className="p-1.5 sm:p-2 text-gray-300 hover:text-white hover:bg-gray-800 transition-all"
            >
              <SkipForward className="w-4 h-4 sm:w-5 sm:h-5" />
            </button>

            {/* Repeat - hidden on mobile */}
            <button
              onClick={cycleRepeatMode}
              className={`hidden sm:block p-2 transition-all hover:bg-gray-800 ${
                player.repeatMode !== 'off' 
                  ? 'text-cyan-400 hover:text-cyan-300' 
                  : 'text-gray-400 hover:text-white'
              }`}
            >
              {player.repeatMode === 'one' ? (
                <Repeat1 className="w-4 h-4" />
              ) : (
                <Repeat className="w-4 h-4" />
              )}
            </button>
          </div>

          {/* Progress Bar - simplified on mobile */}
          {!compact && (
            <>
              {/* Mobile progress bar - full width, minimal */}
              <div className="flex sm:hidden w-full absolute -top-1 left-0 right-0 z-40">
                <div
                  ref={mobileProgressRef}
                  className="w-full h-3 bg-gray-700 cursor-pointer rounded-[5px] overflow-hidden relative"
                >
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="0.1"
                    value={progress}
                    onChange={(e) => {
                      const newProgress = parseFloat(e.target.value);
                      const newTime = (newProgress / 100) * player.duration;
                      seek(newTime);
                    }}
                    className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                  />
                  <div
                    className="h-full bg-gradient-to-r from-cyan-500 to-purple-500 rounded-[5px] pointer-events-none"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>
              
              {/* Desktop progress bar */}
              <div className="hidden sm:flex items-center gap-2 w-48 md:w-72 lg:w-96">
                <AnimatedLedDigits 
                  value={formatDuration(player.currentTime)} 
                  variant="time" 
                  className="text-[10px] sm:text-xs w-10 sm:w-12 text-right"
                  animate={false}
                />
                
                <div
                  ref={progressRef}
                  className="flex-1 h-3 bg-gray-700 cursor-pointer group relative rounded-[5px] overflow-hidden"
                >
                  <input
                    type="range"
                    min="0"
                    max="100"
                    step="0.1"
                    value={progress}
                    onChange={(e) => {
                      const newProgress = parseFloat(e.target.value);
                      const newTime = (newProgress / 100) * player.duration;
                      seek(newTime);
                    }}
                    className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
                  />
                  {/* Progress fill */}
                  <div
                    className="h-full bg-gradient-to-r from-cyan-500 to-purple-500 relative rounded-[5px] pointer-events-none"
                    style={{ width: `${progress}%` }}
                  >
                    {/* Knob */}
                    <div className="absolute right-0 top-1/2 -translate-y-1/2 w-4 h-4 bg-white shadow-lg opacity-0 group-hover:opacity-100 transition-opacity rounded-[5px]" />
                  </div>
                  
                  {/* Glow effect */}
                  <div
                    className="absolute h-full bg-cyan-500/30 blur-sm rounded-[5px] pointer-events-none"
                    style={{ width: `${progress}%` }}
                  />
                </div>
                
                <AnimatedLedDigits 
                  value={formatDuration(player.duration)} 
                  variant="time" 
                  className="text-[10px] sm:text-xs w-10 sm:w-12"
                  animate={false}
                />
              </div>
            </>
          )}
        </div>

        {/* Volume Control - hidden on mobile */}
        <div className="hidden sm:flex items-center gap-2 flex-1 justify-end">
          <button
            onClick={toggleMute}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 transition-all"
          >
            {player.isMuted || player.volume === 0 ? (
              <VolumeX className="w-5 h-5" />
            ) : (
              <Volume2 className="w-5 h-5" />
            )}
          </button>
          
          <div className="w-16 md:w-24 h-3 bg-gray-700 cursor-pointer group relative rounded-[5px] overflow-hidden">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={player.isMuted ? 0 : player.volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
            />
            <div
              className="h-full bg-gradient-to-r from-gray-400 to-white rounded-[5px] pointer-events-none"
              style={{ width: `${(player.isMuted ? 0 : player.volume) * 100}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default MusicPlayer;
