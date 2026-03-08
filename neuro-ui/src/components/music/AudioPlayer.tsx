import { useRef, useEffect, useCallback, useState } from 'react';
import { useMusicStore, useHasHydrated } from '../../stores/musicStore';
import { musicApi } from '../../api/client';
import { useMediaSession } from '../../hooks/useMediaSession';
import { songCache } from '../../services/songCache';

// Equalizer frequency bands in Hz (8 bands)
const EQ_FREQUENCIES = [60, 170, 310, 600, 1000, 3000, 6000, 12000];

// Minimum buffer time in seconds before starting playback (for streaming)
const MIN_BUFFER_SECONDS = 3;
// Maximum time to wait for buffer before playing anyway (in ms)
const MAX_BUFFER_WAIT_MS = 5000;
// Minimum buffer to start playing after timeout (in seconds)
const MIN_BUFFER_FALLBACK = 0.5;

/**
 * AudioPlayer - Global audio element that handles all playback
 * This component should be mounted at the app root level so audio
 * continues playing when navigating between pages.
 */
export const AudioPlayer: React.FC = () => {
  const audioRef = useRef<HTMLAudioElement>(null);
  
  const {
    player,
    equalizer,
    nextSong,
    clearSeek,
    setPlayerLoading,
    setCurrentTime,
    setDuration,
  } = useMusicStore();

  // Track hydration state using the proper Zustand API
  const hasHydrated = useHasHydrated();

  // Media Session API for background playback and notification controls
  useMediaSession();

  // Audio context for spectrum analysis and equalizer
  const audioContextRef = useRef<AudioContext | null>(null);
  const analyserRef = useRef<AnalyserNode | null>(null);
  const sourceRef = useRef<MediaElementAudioSourceNode | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const eqFiltersRef = useRef<BiquadFilterNode[]>([]);
  const highpassFilterRef = useRef<BiquadFilterNode | null>(null);
  const lowpassFilterRef = useRef<BiquadFilterNode | null>(null);
  // Effect nodes
  const loudnessLowRef = useRef<BiquadFilterNode | null>(null);
  const loudnessHighRef = useRef<BiquadFilterNode | null>(null);
  const bassBoostRef = useRef<BiquadFilterNode | null>(null);
  const vocalEnhancerRef = useRef<BiquadFilterNode | null>(null);
  // For stereo wide effect
  const splitterRef = useRef<ChannelSplitterNode | null>(null);
  const mergerRef = useRef<ChannelMergerNode | null>(null);
  const stereoDelayRRef = useRef<DelayNode | null>(null);
  
  const animationRef = useRef<number | null>(null);
  const isAudioInitializedRef = useRef(false);
  // Reuse Uint8Array to avoid memory allocation every frame
  const spectrumDataArrayRef = useRef<Uint8Array | null>(null);
  
  // Cache-related state
  const [isFromCache, setIsFromCache] = useState(false);
  const currentBlobUrlRef = useRef<string | null>(null);
  const hasEnoughBufferRef = useRef(false);
  const bufferTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // Get audio filters from store
  const audioFilters = useMusicStore(state => state.audioFilters);

  // Initialize audio context, analyzer, and equalizer
  const initAudioContext = useCallback(() => {
    if (!audioRef.current || isAudioInitializedRef.current) return;
    
    // Wait for hydration to complete before initializing
    if (!useMusicStore.persist.hasHydrated()) {
      console.log('🎵 Waiting for hydration before initializing audio...');
      return;
    }
    
    // Check if audio element has a source
    if (!audioRef.current.src || audioRef.current.src === '') {
      console.log('🎵 No audio source yet, skipping initialization');
      return;
    }

    // Get current state values
    const currentState = useMusicStore.getState();
    const currentFilters = currentState.audioFilters;
    const currentPlayer = currentState.player;
    
    console.log('🎛️ Initializing with EQ bands:', currentState.equalizer.bands);

    try {
      const audioContext = new AudioContext();
      const analyser = audioContext.createAnalyser();
      analyser.fftSize = 256;
      analyser.smoothingTimeConstant = 0.8;
      analyser.minDecibels = -70;
      analyser.maxDecibels = -10;

      const source = audioContext.createMediaElementSource(audioRef.current);
      const gainNode = audioContext.createGain();
      
      // Set initial volume from store (max 0.3 to prevent distortion)
      const initialVolume = currentPlayer.isMuted ? 0 : Math.min(currentPlayer.volume, 1) * 0.3;
      gainNode.gain.value = initialVolume;

      // Create highpass filter
      const highpassFilter = audioContext.createBiquadFilter();
      highpassFilter.type = 'highpass';
      highpassFilter.frequency.value = currentFilters.highpassEnabled ? currentFilters.highpassFreq : 1;
      highpassFilter.Q.value = 1;

      // Create lowpass filter
      const lowpassFilter = audioContext.createBiquadFilter();
      lowpassFilter.type = 'lowpass';
      lowpassFilter.frequency.value = currentFilters.lowpassEnabled ? currentFilters.lowpassFreq : 22050;
      lowpassFilter.Q.value = 1;

      // === NEW EFFECTS ===
      
      // Loudness compensation (Fletcher-Munson) - boost lows and highs
      const loudnessLow = audioContext.createBiquadFilter();
      loudnessLow.type = 'lowshelf';
      loudnessLow.frequency.value = 100;
      loudnessLow.gain.value = currentFilters.loudnessEnabled ? 6 : 0;
      
      const loudnessHigh = audioContext.createBiquadFilter();
      loudnessHigh.type = 'highshelf';
      loudnessHigh.frequency.value = 8000;
      loudnessHigh.gain.value = currentFilters.loudnessEnabled ? 4 : 0;

      // Bass Boost - sub-bass enhancement
      const bassBoost = audioContext.createBiquadFilter();
      bassBoost.type = 'peaking';
      bassBoost.frequency.value = 60;
      bassBoost.Q.value = 1;
      bassBoost.gain.value = currentFilters.bassBoostEnabled ? 8 : 0;

      // Vocal Enhancer - presence boost
      const vocalEnhancer = audioContext.createBiquadFilter();
      vocalEnhancer.type = 'peaking';
      vocalEnhancer.frequency.value = 3000;
      vocalEnhancer.Q.value = 1.5;
      vocalEnhancer.gain.value = currentFilters.vocalEnhancerEnabled ? 5 : 0;

      // Stereo processing setup
      const splitter = audioContext.createChannelSplitter(2);
      const merger = audioContext.createChannelMerger(2);
      
      // Stereo Wide - small delay on right channel
      const stereoDelayR = audioContext.createDelay(0.1);
      stereoDelayR.delayTime.value = currentFilters.stereoWideEnabled ? 0.012 : 0; // 12ms delay

      // Create equalizer filters
      const eqFilters: BiquadFilterNode[] = EQ_FREQUENCIES.map((freq, index) => {
        const filter = audioContext.createBiquadFilter();
        filter.type = 'peaking';
        filter.frequency.value = freq;
        filter.Q.value = 1.4;
        filter.gain.value = equalizer.bands[index] || 0;
        return filter;
      });

      // === AUDIO CHAIN CONNECTION ===
      // Simple chain: source -> effects -> EQ -> analyser -> stereo processing -> compressor -> gain -> output
      
      let lastNode: AudioNode = source;
      
      // Highpass & Lowpass
      lastNode.connect(highpassFilter);
      lastNode = highpassFilter;
      lastNode.connect(lowpassFilter);
      lastNode = lowpassFilter;
      
      // Effect filters (loudness, bass boost, vocal enhancer)
      lastNode.connect(loudnessLow);
      lastNode = loudnessLow;
      lastNode.connect(loudnessHigh);
      lastNode = loudnessHigh;
      lastNode.connect(bassBoost);
      lastNode = bassBoost;
      lastNode.connect(vocalEnhancer);
      lastNode = vocalEnhancer;
      
      // EQ filters
      eqFilters.forEach((filter) => {
        lastNode.connect(filter);
        lastNode = filter;
      });
      
      // Analyser (before stereo processing)
      lastNode.connect(analyser);
      
      // Stereo processing (for stereo wide effect)
      lastNode.connect(splitter);
      
      // Left channel - direct
      splitter.connect(merger, 0, 0);
      
      // Right channel - with optional delay for stereo widening
      splitter.connect(stereoDelayR, 1);
      stereoDelayR.connect(merger, 0, 1);
      
      // Output
      merger.connect(gainNode);
      gainNode.connect(audioContext.destination);

      // Store references
      audioContextRef.current = audioContext;
      analyserRef.current = analyser;
      sourceRef.current = source;
      gainNodeRef.current = gainNode;
      eqFiltersRef.current = eqFilters;
      highpassFilterRef.current = highpassFilter;
      lowpassFilterRef.current = lowpassFilter;
      // Effect refs
      loudnessLowRef.current = loudnessLow;
      loudnessHighRef.current = loudnessHigh;
      bassBoostRef.current = bassBoost;
      vocalEnhancerRef.current = vocalEnhancer;
      splitterRef.current = splitter;
      mergerRef.current = merger;
      stereoDelayRRef.current = stereoDelayR;
      
      isAudioInitializedRef.current = true;

      console.log('🎵 Global audio context initialized with all effects');
      console.log('🎛️ Initial HPF:', currentFilters.highpassEnabled ? `ON at ${currentFilters.highpassFreq}Hz` : 'OFF');
      console.log('🎛️ Initial LPF:', currentFilters.lowpassEnabled ? `ON at ${currentFilters.lowpassFreq}Hz` : 'OFF');
      console.log('🔊 Initial volume:', currentPlayer.volume);
      
      // Set audio element volume to max - all volume control goes through gainNode
      // This ensures the spectrum analyzer is not affected by volume changes
      if (audioRef.current) {
        audioRef.current.volume = 1;
      }

      // Start spectrum animation
      // Frequency compensation: bass frequencies naturally have more energy
      // We apply a stronger curve to reduce bass and boost treble for visual balance
      const frequencyCompensation = (index: number, total: number): number => {
        // Stronger curve: 0.35 (bass) to 1.8 (treble)
        const position = index / total;
        // Use exponential curve for more natural compensation
        return 0.35 + Math.pow(position, 0.7) * 1.45;
      };
      
      // Create reusable Uint8Array for spectrum data (avoid allocations every frame)
      spectrumDataArrayRef.current = new Uint8Array(analyser.frequencyBinCount);
      
      let frameCount = 0;
      const barCount = 32;
      const step = Math.floor(analyser.frequencyBinCount / barCount);
      
      const updateSpectrum = () => {
        if (!analyserRef.current || !spectrumDataArrayRef.current) return;
        
        // Reuse the existing Uint8Array instead of creating a new one
        analyserRef.current.getByteFrequencyData(spectrumDataArrayRef.current as any);
        
        const bars: number[] = new Array(barCount);
        
        for (let i = 0; i < barCount; i++) {
          let sum = 0;
          const baseIndex = i * step;
          for (let j = 0; j < step; j++) {
            sum += spectrumDataArrayRef.current[baseIndex + j];
          }
          // Normalize to 0-1 and apply gain boost
          let normalized = (sum / step / 255) * 2.0;
          
          // Apply frequency compensation (reduce bass, boost treble)
          normalized *= frequencyCompensation(i, barCount);
          
          // Apply soft knee compression to prevent clipping while keeping dynamics
          if (normalized > 0.4) {
            normalized = 0.4 + (normalized - 0.4) * 0.4;
          }
          
          bars[i] = Math.min(1, Math.max(0, normalized));
        }
        
        // Use getState() to avoid re-render loops - this is called ~60fps
        useMusicStore.getState().setSpectrumData(bars);
        
        // Log every 60 frames (~1 second) for debugging
        frameCount++;
        if (frameCount % 60 === 0) {
          const maxBar = Math.max(...bars);
          console.log('📊 Spectrum update - max bar:', maxBar.toFixed(3));
        }
        
        animationRef.current = requestAnimationFrame(updateSpectrum);
      };

      console.log('📊 Starting spectrum animation loop...');
      updateSpectrum();
    } catch (error) {
      console.error('Failed to initialize audio context:', error);
    }
  }, []); // No dependencies - setSpectrumData is accessed via getState()

  // Retry audio initialization when hydration completes
  useEffect(() => {
    if (hasHydrated && !isAudioInitializedRef.current && audioRef.current) {
      console.log('🎵 Hydration complete, initializing audio with persisted settings...');
      initAudioContext();
    }
  }, [hasHydrated, initAudioContext]);

  // Update equalizer filters when bands change
  useEffect(() => {
    if (!eqFiltersRef.current.length) return;
    
    console.log('🎛️ Updating EQ bands:', equalizer.bands);
    eqFiltersRef.current.forEach((filter, index) => {
      if (equalizer.enabled) {
        filter.gain.value = equalizer.bands[index] || 0;
      } else {
        filter.gain.value = 0;
      }
    });
  }, [equalizer.bands, equalizer.enabled]);

  // Update highpass/lowpass filters when they change
  useEffect(() => {
    if (highpassFilterRef.current) {
      const hpFreq = audioFilters.highpassEnabled ? audioFilters.highpassFreq : 1;
      highpassFilterRef.current.frequency.value = hpFreq;
      console.log('🎛️ Highpass filter:', audioFilters.highpassEnabled ? `ON at ${hpFreq}Hz` : 'OFF (1Hz)');
    }
    if (lowpassFilterRef.current) {
      const lpFreq = audioFilters.lowpassEnabled ? audioFilters.lowpassFreq : 22050;
      lowpassFilterRef.current.frequency.value = lpFreq;
      console.log('🎛️ Lowpass filter:', audioFilters.lowpassEnabled ? `ON at ${lpFreq}Hz` : 'OFF (22050Hz)');
    }
  }, [audioFilters.highpassEnabled, audioFilters.lowpassEnabled, audioFilters.highpassFreq, audioFilters.lowpassFreq]);

  // Update loudness compensation
  useEffect(() => {
    if (loudnessLowRef.current && loudnessHighRef.current) {
      loudnessLowRef.current.gain.value = audioFilters.loudnessEnabled ? 6 : 0;
      loudnessHighRef.current.gain.value = audioFilters.loudnessEnabled ? 4 : 0;
      console.log('🎛️ Loudness:', audioFilters.loudnessEnabled ? 'ON' : 'OFF');
    }
  }, [audioFilters.loudnessEnabled]);

  // Update bass boost
  useEffect(() => {
    if (bassBoostRef.current) {
      bassBoostRef.current.gain.value = audioFilters.bassBoostEnabled ? 8 : 0;
      console.log('🎛️ Bass Boost:', audioFilters.bassBoostEnabled ? 'ON' : 'OFF');
    }
  }, [audioFilters.bassBoostEnabled]);

  // Update stereo wide
  useEffect(() => {
    if (stereoDelayRRef.current) {
      stereoDelayRRef.current.delayTime.value = audioFilters.stereoWideEnabled ? 0.012 : 0;
      console.log('🎛️ Stereo Wide:', audioFilters.stereoWideEnabled ? 'ON' : 'OFF');
    }
  }, [audioFilters.stereoWideEnabled]);

  // Update vocal enhancer
  useEffect(() => {
    if (vocalEnhancerRef.current) {
      vocalEnhancerRef.current.gain.value = audioFilters.vocalEnhancerEnabled ? 5 : 0;
      console.log('🎛️ Vocal Enhancer:', audioFilters.vocalEnhancerEnabled ? 'ON' : 'OFF');
    }
  }, [audioFilters.vocalEnhancerEnabled]);

  // Resume audio context on user interaction
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

  // Handle visibility change - track playing state
  const wasPlayingRef = useRef(false);
  const isPlayingRef = useRef(false);
  
  useEffect(() => {
    isPlayingRef.current = player.isPlaying;
  }, [player.isPlaying]);
  
  useEffect(() => {
    const handleVisibilityChange = () => {
      if (document.visibilityState === 'hidden') {
        wasPlayingRef.current = isPlayingRef.current;
        console.log('🎵 Tab hidden, was playing:', wasPlayingRef.current);
      } else if (document.visibilityState === 'visible') {
        console.log('🎵 Tab visible, resuming... wasPlaying:', wasPlayingRef.current);
        if (audioContextRef.current?.state === 'suspended') {
          audioContextRef.current.resume().then(() => {
            console.log('🎵 AudioContext resumed');
          });
        }
        if (wasPlayingRef.current && audioRef.current?.paused) {
          console.log('🎵 Resuming playback...');
          audioRef.current.play().catch(e => console.error('Resume failed:', e));
        }
      }
    };

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
  }, []);

  // Handle audio source change - check cache first, then stream
  useEffect(() => {
    if (!audioRef.current || !player.currentSong) return;

    const loadAudioSource = async () => {
      // Clean up previous blob URL if any
      if (currentBlobUrlRef.current) {
        URL.revokeObjectURL(currentBlobUrlRef.current);
        currentBlobUrlRef.current = null;
      }
      
      // Clear any existing buffer timeout
      if (bufferTimeoutRef.current) {
        clearTimeout(bufferTimeoutRef.current);
        bufferTimeoutRef.current = null;
      }
      
      hasEnoughBufferRef.current = false;
      setPlayerLoading(true);

      // Check cache first
      const cachedBlob = await songCache.get(player.currentSong!.id);
      
      if (cachedBlob) {
        // Play from cache
        console.log('🎵 Playing from cache:', player.currentSong!.title);
        const blobUrl = URL.createObjectURL(cachedBlob);
        currentBlobUrlRef.current = blobUrl;
        audioRef.current!.src = blobUrl;
        setIsFromCache(true);
        hasEnoughBufferRef.current = true; // Cache is fully loaded
      } else {
        // Stream from server
        console.log('🎵 Streaming from server:', player.currentSong!.title);
        const streamUrl = musicApi.getStreamUrl(player.currentSong!.id);
        audioRef.current!.src = streamUrl;
        setIsFromCache(false);
        
        // Set a timeout to start playing even if buffer is small
        bufferTimeoutRef.current = setTimeout(() => {
          if (!hasEnoughBufferRef.current && audioRef.current) {
            const buffered = audioRef.current.buffered;
            if (buffered.length > 0) {
              const currentTime = audioRef.current.currentTime || 0;
              const bufferedEnd = buffered.end(buffered.length - 1);
              const bufferedSeconds = bufferedEnd - currentTime;
              
              if (bufferedSeconds >= MIN_BUFFER_FALLBACK) {
                console.log(`🎵 Buffer timeout - starting with ${bufferedSeconds.toFixed(1)}s buffer`);
                hasEnoughBufferRef.current = true;
                setPlayerLoading(false);
                if (player.isPlaying && audioRef.current.paused) {
                  audioRef.current.play().catch(console.error);
                }
              }
            }
          }
        }, MAX_BUFFER_WAIT_MS);
      }

      audioRef.current!.load();

      if (player.isPlaying) {
        // For streaming, wait for buffer before playing
        if (!hasEnoughBufferRef.current) {
          // Will be handled by onCanPlay/onProgress or timeout
          console.log('🎵 Waiting for buffer...');
        } else {
          audioRef.current!.play().catch(console.error);
        }
      }
    };

    loadAudioSource();

    // Cleanup blob URL on unmount or song change
    return () => {
      if (currentBlobUrlRef.current) {
        URL.revokeObjectURL(currentBlobUrlRef.current);
        currentBlobUrlRef.current = null;
      }
      if (bufferTimeoutRef.current) {
        clearTimeout(bufferTimeoutRef.current);
        bufferTimeoutRef.current = null;
      }
    };
  }, [player.currentSong?.id, setPlayerLoading]);

  // Handle play/pause
  useEffect(() => {
    if (!audioRef.current) return;

    if (player.isPlaying) {
      audioRef.current.play().catch((error) => {
        console.log('🎵 Play interrupted:', error.name);
      });
    } else {
      audioRef.current.pause();
    }
  }, [player.isPlaying]);

  // Handle volume - use gainNode to not affect spectrum analyzer
  useEffect(() => {
    if (gainNodeRef.current) {
      // Use gainNode for volume control so spectrum analyzer is not affected
      const volume = player.isMuted ? 0 : Math.min(player.volume, 1.0);
      // Max gain at 0.3 to prevent distortion and keep comfortable levels
      gainNodeRef.current.gain.value = volume * 0.3;
    } else if (audioRef.current) {
      // Fallback to audio element volume if context not initialized yet
      audioRef.current.volume = player.isMuted ? 0 : Math.min(player.volume, 1.0) * 0.3;
    }
  }, [player.volume, player.isMuted]);

  // Handle seek requests from store
  useEffect(() => {
    if (player.seekTo !== null && audioRef.current && Number.isFinite(player.seekTo) && player.seekTo >= 0) {
      console.log('🎵 Seeking to:', player.seekTo);
      audioRef.current.currentTime = player.seekTo;
      clearSeek();
    } else if (player.seekTo !== null) {
      // Invalid seek value, just clear it
      clearSeek();
    }
  }, [player.seekTo, clearSeek]);

  // Audio event handlers
  const handleLoadedMetadata = () => {
    if (audioRef.current) {
      const audioDuration = audioRef.current.duration;
      if (Number.isFinite(audioDuration) && audioDuration > 0) {
        setDuration(audioDuration);
      } else if (player.currentSong?.duration) {
        setDuration(player.currentSong.duration);
      }
      // Only set loading false for cached content - streaming will be handled by buffer logic
      if (isFromCache || hasEnoughBufferRef.current) {
        setPlayerLoading(false);
      }
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
    if (!audioRef.current) return;

    // If we already have enough buffer, don't re-check
    if (hasEnoughBufferRef.current) {
      if (player.isPlaying && audioRef.current.paused) {
        audioRef.current.play().catch(console.error);
      }
      return;
    }

    // For cached songs, we can play immediately
    if (isFromCache) {
      if (bufferTimeoutRef.current) {
        clearTimeout(bufferTimeoutRef.current);
        bufferTimeoutRef.current = null;
      }
      setPlayerLoading(false);
      hasEnoughBufferRef.current = true;
      if (player.isPlaying) {
        audioRef.current.play().catch(console.error);
      }
      return;
    }

    // For streaming, check buffer level
    const buffered = audioRef.current.buffered;
    if (buffered.length > 0) {
      const currentTime = audioRef.current.currentTime || 0;
      const bufferedEnd = buffered.end(buffered.length - 1);
      const bufferedSeconds = bufferedEnd - currentTime;

      // Only log occasionally to avoid spam
      console.log(`🎵 CanPlay - Buffer: ${bufferedSeconds.toFixed(1)}s / ${MIN_BUFFER_SECONDS}s required`);

      if (bufferedSeconds >= MIN_BUFFER_SECONDS) {
        if (bufferTimeoutRef.current) {
          clearTimeout(bufferTimeoutRef.current);
          bufferTimeoutRef.current = null;
        }
        hasEnoughBufferRef.current = true;
        setPlayerLoading(false);
        if (player.isPlaying) {
          audioRef.current.play().catch(console.error);
        }
      }
      // Don't log "still buffering" here - progress event will handle updates
    }
  };

  // Handle progress (buffer updates) for streaming
  const handleProgress = () => {
    if (!audioRef.current || isFromCache || hasEnoughBufferRef.current) return;

    const buffered = audioRef.current.buffered;
    if (buffered.length > 0) {
      const currentTime = audioRef.current.currentTime || 0;
      const bufferedEnd = buffered.end(buffered.length - 1);
      const bufferedSeconds = bufferedEnd - currentTime;

      console.log(`🎵 Progress - Buffer: ${bufferedSeconds.toFixed(1)}s / ${MIN_BUFFER_SECONDS}s required`);

      if (bufferedSeconds >= MIN_BUFFER_SECONDS && !hasEnoughBufferRef.current) {
        console.log(`🎵 Buffer ready: ${bufferedSeconds.toFixed(1)}s`);
        if (bufferTimeoutRef.current) {
          clearTimeout(bufferTimeoutRef.current);
          bufferTimeoutRef.current = null;
        }
        hasEnoughBufferRef.current = true;
        setPlayerLoading(false);
        if (player.isPlaying && audioRef.current.paused) {
          audioRef.current.play().catch(console.error);
        }
      }
    }
  };

  // Cleanup
  useEffect(() => {
    return () => {
      if (animationRef.current) {
        cancelAnimationFrame(animationRef.current);
      }
    };
  }, []);

  // This component renders only the audio element, no UI
  return (
    <audio
      ref={audioRef}
      crossOrigin="anonymous"
      preload="auto"
      onLoadedMetadata={handleLoadedMetadata}
      onTimeUpdate={handleTimeUpdate}
      onEnded={handleEnded}
      onError={handleError}
      onCanPlay={handleCanPlay}
      onProgress={handleProgress}
      onLoadStart={() => setPlayerLoading(true)}
      style={{ display: 'none' }}
    />
  );
};

export default AudioPlayer;
