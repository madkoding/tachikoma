import React, { useRef, useEffect, useCallback } from 'react';
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
  Music
} from 'lucide-react';
import { useMusicStore, formatDuration, RepeatMode } from '../../stores/musicStore';
import { musicApi } from '../../api/client';

// Equalizer frequency bands in Hz
const EQ_FREQUENCIES = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000, 20, 45, 90, 180, 350, 700];

interface MusicPlayerProps {
  compact?: boolean;
}

export const MusicPlayer: React.FC<MusicPlayerProps> = ({ compact = false }) => {
  const audioRef = useRef<HTMLAudioElement>(null);
  const progressRef = useRef<HTMLDivElement>(null);
  const mobileProgressRef = useRef<HTMLDivElement>(null);
  
  const {
    player,
    equalizer,
    togglePlay,
    pause,
    nextSong,
    previousSong,
    seek,
    setVolume,
    toggleMute,
    toggleShuffle,
    setRepeatMode,
    setPlayerLoading,
    setCurrentTime,
    setDuration,
    setSpectrumData,
  } = useMusicStore();

  // Audio context for spectrum analysis and equalizer
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const sourceRef = useRef<MediaElementAudioSourceNode | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const eqFiltersRef = useRef<BiquadFilterNode[]>([]);
  const highpassFilterRef = useRef<BiquadFilterNode | null>(null);
  const lowpassFilterRef = useRef<BiquadFilterNode | null>(null);
  const animationRef = useRef<number | null>(null);
  const isAudioInitializedRef = useRef(false);

  // Get audio filters from store
  const audioFilters = useMusicStore(state => state.audioFilters);

  // Initialize audio context, analyzer, and equalizer
  const initAudioContext = useCallback(() => {
    if (!audioRef.current || isAudioInitializedRef.current) return;

    try {
      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 256;
      analyser.smoothingTimeConstant = 0.8;

      const source = audioContext.createMediaElementSource(audioRef.current);
      const gainNode = audioContext.createGain();

      // Create highpass filter
      const highpassFilter = audioContext.createBiquadFilter();
      highpassFilter.type = 'highpass';
      highpassFilter.frequency.value = 80;
      highpassFilter.Q.value = 0.7;

      // Create lowpass filter
      const lowpassFilter = audioContext.createBiquadFilter();
      lowpassFilter.type = 'lowpass';
      lowpassFilter.frequency.value = 12000;
      lowpassFilter.Q.value = 0.7;

      // Create equalizer filters for each frequency band
      const eqFilters: BiquadFilterNode[] = EQ_FREQUENCIES.map((freq, index) => {
        const filter = audioContext.createBiquadFilter();
        filter.type = 'peaking';
        filter.frequency.value = freq;
        filter.Q.value = 1.4; // Quality factor
        filter.gain.value = equalizer.bands[index] || 0;
        return filter;
      });

      // Connect the audio chain:
      // source -> highpass -> lowpass -> eq filters -> analyser -> gain -> destination
      // Analyser is BEFORE gain so volume doesn't affect spectrum
      let lastNode: AudioNode = source;
      
      // Highpass and lowpass filters (start bypassed)
      lastNode.connect(highpassFilter);
      lastNode = highpassFilter;
      
      lastNode.connect(lowpassFilter);
      lastNode = lowpassFilter;
      
      // EQ filters
      eqFilters.forEach((filter) => {
        lastNode.connect(filter);
        lastNode = filter;
      });
      
      // Analyser (before volume)
      lastNode.connect(analyser);
      
      // Gain (volume control) after analyser
      analyser.connect(gainNode);
      gainNode.connect(audioContext.destination);

      // Store references
      audioContextRef.current = audioContext;
      analyserRef.current = analyser;
      sourceRef.current = source;
      gainNodeRef.current = gainNode;
      eqFiltersRef.current = eqFilters;
      highpassFilterRef.current = highpassFilter;
      lowpassFilterRef.current = lowpassFilter;
      isAudioInitializedRef.current = true;

      console.log('🎵 Audio context initialized with equalizer and filters');

      // Start spectrum animation
      const updateSpectrum = () => {
        if (!analyserRef.current) return;
        
        const dataArray = new Uint8Array(analyserRef.current.frequencyBinCount);
        analyserRef.current.getByteFrequencyData(dataArray);
        
        // Convert to 32 bars normalized to 0-1
        const barCount = 32;
        const bars: number[] = [];
        const step = Math.floor(dataArray.length / barCount);
        
        for (let i = 0; i < barCount; i++) {
          let sum = 0;
          for (let j = 0; j < step; j++) {
            sum += dataArray[i * step + j];
          }
          bars.push(sum / step / 255);
        }
        
        setSpectrumData(bars);
        animationRef.current = requestAnimationFrame(updateSpectrum);
      };

      updateSpectrum();
    } catch (error) {
      console.error('Failed to initialize audio context:', error);
    }
  }, [setSpectrumData, equalizer.bands]);

  // Update equalizer filters when bands change
  useEffect(() => {
    if (!eqFiltersRef.current.length) return;
    
    eqFiltersRef.current.forEach((filter, index) => {
      if (equalizer.enabled) {
        filter.gain.value = equalizer.bands[index] || 0;
      } else {
        filter.gain.value = 0; // Bypass when disabled
      }
    });
  }, [equalizer.bands, equalizer.enabled]);

  // Update highpass/lowpass filters when they change
  useEffect(() => {
    if (highpassFilterRef.current) {
      // When disabled, set frequency to 0 (bypass)
      highpassFilterRef.current.frequency.value = audioFilters.highpassEnabled 
        ? audioFilters.highpassFreq 
        : 0;
    }
    if (lowpassFilterRef.current) {
      // When disabled, set frequency to max (bypass)
      lowpassFilterRef.current.frequency.value = audioFilters.lowpassEnabled 
        ? audioFilters.lowpassFreq 
        : 22050;
    }
  }, [audioFilters.highpassEnabled, audioFilters.lowpassEnabled, audioFilters.highpassFreq, audioFilters.lowpassFreq]);

  // Resume audio context on user interaction (required by browsers)
  useEffect(() => {
    const resumeAudioContext = () => {
      if (audioContextRef.current?.state === 'suspended') {
        audioContextRef.current.resume();
      }
    };

    document.addEventListener('click', resumeAudioContext);
    document.addEventListener('keydown', resumeAudioContext);

    return () => {
      document.removeEventListener('click', resumeAudioContext);
      document.removeEventListener('keydown', resumeAudioContext);
    };
  }, []);

  // Handle visibility change - prevent playback stopping when switching tabs
  // Store playing state before tab becomes hidden
  const wasPlayingRef = useRef(false);
  const isPlayingRef = useRef(false);
  
  // Keep isPlayingRef in sync with player.isPlaying
  useEffect(() => {
    isPlayingRef.current = player.isPlaying;
  }, [player.isPlaying]);
  
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'hidden') {
        // Store current playing state when leaving tab
        wasPlayingRef.current = isPlayingRef.current;
        console.log('🎵 Tab hidden, was playing:', wasPlayingRef.current);
      } else if (document.visibilityState === 'visible') {
        console.log('🎵 Tab visible, resuming... wasPlaying:', wasPlayingRef.current);
        // Resume audio context when tab becomes visible
        if (audioContextRef.current?.state === 'suspended') {
          audioContextRef.current.resume().then(() => {
            console.log('🎵 AudioContext resumed');
          });
        }
        // If was playing before tab switch, resume playback
        if (wasPlayingRef.current && audioRef.current && audioRef.current.paused) {
          console.log('🎵 Resuming playback...');
          audioRef.current.play().catch(e => console.error('Resume failed:', e));
        }
      }
    };

    // Also handle page show event for back/forward cache
    const handlePageShow = (e: PageTransitionEvent) => {
      if (e.persisted && audioRef.current?.paused && wasPlayingRef.current) {
        audioContextRef.current?.resume();
        audioRef.current.play().catch(console.error);
      }
    };

    document.addEventListener('visibilitychange', handleVisibilityChange);
    window.addEventListener('pageshow', handlePageShow);

    return () => {
      document.removeEventListener('visibilitychange', handleVisibilityChange);
      window.removeEventListener('pageshow', handlePageShow);
    };
  }, []); // No dependencies - use refs instead

  // Handle audio source change
  useEffect(() => {
    if (!audioRef.current || !player.currentSong) return;

    const streamUrl = musicApi.getStreamUrl(player.currentSong.id);
    audioRef.current.src = streamUrl;
    audioRef.current.load();

    if (player.isPlaying) {
      audioRef.current.play().catch(console.error);
    }
  }, [player.currentSong?.id]);

  // Handle play/pause
  useEffect(() => {
    if (!audioRef.current) return;

    if (player.isPlaying) {
      // Don't call pause() on error - this would cause a loop
      // The audio might fail to play in background but that's ok
      // It will resume when the tab becomes visible
      audioRef.current.play().catch((error) => {
        // Only log, don't change state - tab switching can cause this
        console.log('🎵 Play interrupted (tab switch?):', error.name);
      });
    } else {
      audioRef.current.pause();
    }
  }, [player.isPlaying]);

  // Handle volume
  useEffect(() => {
    if (!audioRef.current) return;
    audioRef.current.volume = player.isMuted ? 0 : player.volume;
  }, [player.volume, player.isMuted]);

  // Audio event handlers
  const handleLoadedMetadata = () => {
    if (audioRef.current) {
      // Use song duration from metadata since stream is infinite
      // Only use audio duration if valid (not Infinity or NaN)
      const audioDuration = audioRef.current.duration;
      if (Number.isFinite(audioDuration) && audioDuration > 0) {
        setDuration(audioDuration);
      } else if (player.currentSong?.duration) {
        // Fallback to stored song duration
        setDuration(player.currentSong.duration);
      }
      setPlayerLoading(false);
      initAudioContext();
    }
  };

  const handleTimeUpdate = () => {
    if (audioRef.current) {
      setCurrentTime(audioRef.current.currentTime);
    }
  };

  const handleEnded = () => {
    if (player.repeatMode === 'one') {
      if (audioRef.current) {
        audioRef.current.currentTime = 0;
        audioRef.current.play();
      }
    } else {
      nextSong();
    }
  };

  const handleError = () => {
    console.error('Audio playback error');
    setPlayerLoading(false);
  };

  const handleCanPlay = () => {
    setPlayerLoading(false);
    if (player.isPlaying && audioRef.current) {
      audioRef.current.play().catch(console.error);
    }
  };

  // Progress bar click
  const handleProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!progressRef.current || !audioRef.current) return;
    
    const rect = progressRef.current.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    const newTime = percent * player.duration;
    
    audioRef.current.currentTime = newTime;
    seek(newTime);
  };

  // Mobile progress bar click
  const handleMobileProgressClick = (e: React.MouseEvent<HTMLDivElement>) => {
    if (!mobileProgressRef.current || !audioRef.current) return;
    
    const rect = mobileProgressRef.current.getBoundingClientRect();
    const percent = (e.clientX - rect.left) / rect.width;
    const newTime = percent * player.duration;
    
    audioRef.current.currentTime = newTime;
    seek(newTime);
  };

  // Repeat mode cycle
  const cycleRepeatMode = () => {
    const modes: RepeatMode[] = ['off', 'all', 'one'];
    const currentIndex = modes.indexOf(player.repeatMode);
    const nextMode = modes[(currentIndex + 1) % modes.length];
    setRepeatMode(nextMode);
  };

  // Cleanup
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  const progress = player.duration > 0 ? (player.currentTime / player.duration) * 100 : 0;

  if (!player.currentSong && !compact) {
    return (
      <div className="h-16 sm:h-24 bg-gray-900/80 backdrop-blur-xl border-t border-cyan-500/30 flex items-center justify-center text-gray-500 px-2">
        <Music className="w-5 h-5 sm:w-6 sm:h-6 mr-2 opacity-50" />
        <span className="text-xs sm:text-sm">Selecciona una canción para reproducir</span>
      </div>
    );
  }

  if (!player.currentSong) return null;

  return (
    <div className={`
      bg-gray-900/90 backdrop-blur-xl border-t border-cyan-500/30
      ${compact ? 'p-2' : 'p-2 sm:p-4 pt-3 sm:pt-4'}
      w-full relative
    `}>
      {/* Audio element is now in AudioPlayer component (global) */}

      <div className="max-w-screen-xl mx-auto flex items-center gap-2 sm:gap-4">
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
              <div className="flex sm:hidden w-full absolute -top-1 left-0 right-0">
                <div
                  ref={mobileProgressRef}
                  onClick={handleMobileProgressClick}
                  className="w-full h-1 bg-gray-700 cursor-pointer"
                >
                  <div
                    className="h-full bg-gradient-to-r from-cyan-500 to-purple-500"
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>
              
              {/* Desktop progress bar */}
              <div className="hidden sm:flex items-center gap-2 w-48 md:w-72 lg:w-96">
                <span className="text-[10px] sm:text-xs led-time w-10 sm:w-12 text-right">
                  {formatDuration(player.currentTime)}
                </span>
                
                <div
                  ref={progressRef}
                  onClick={handleProgressClick}
                  className="flex-1 h-1 bg-gray-700 cursor-pointer group relative"
                >
                  {/* Progress fill */}
                  <div
                    className="h-full bg-gradient-to-r from-cyan-500 to-purple-500 relative"
                    style={{ width: `${progress}%` }}
                  >
                    {/* Knob */}
                    <div className="absolute right-0 top-1/2 -translate-y-1/2 w-3 h-3 bg-white shadow-lg opacity-0 group-hover:opacity-100 transition-opacity" />
                  </div>
                  
                  {/* Glow effect */}
                  <div
                    className="absolute h-full bg-cyan-500/30 blur-sm"
                    style={{ width: `${progress}%` }}
                  />
                </div>
                
                <span className="text-[10px] sm:text-xs led-time w-10 sm:w-12">
                  {formatDuration(player.duration)}
                </span>
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
          
          <div className="w-16 md:w-24 h-1 bg-gray-700 cursor-pointer group relative">
            <input
              type="range"
              min="0"
              max="1"
              step="0.01"
              value={player.isMuted ? 0 : player.volume}
              onChange={(e) => setVolume(parseFloat(e.target.value))}
              className="absolute inset-0 w-full h-full opacity-0 cursor-pointer"
            />
            <div
              className="h-full bg-gradient-to-r from-gray-400 to-white"
              style={{ width: `${(player.isMuted ? 0 : player.volume) * 100}%` }}
            />
          </div>
        </div>
      </div>
    </div>
  );
};

export default MusicPlayer;
