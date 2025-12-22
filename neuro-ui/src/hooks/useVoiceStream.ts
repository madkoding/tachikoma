/**
 * =============================================================================
 * useVoiceStream Hook - Web Audio API Voice Synthesis
 * =============================================================================
 * Custom React hook for streaming text-to-speech synthesis.
 * Uses Web Audio API with a queue-based playback system for smooth audio.
 * Connects to Piper TTS voice service (ultra-fast local TTS).
 * =============================================================================
 */

import { useCallback, useEffect, useRef, useState } from 'react';

// =============================================================================
// Types
// =============================================================================

export interface VoiceConfig {
  /** Enable/disable voice synthesis */
  enabled: boolean;
  /** Volume level (0.0 - 1.0) */
  volume: number;
  /** Auto-play voice on new messages */
  autoPlay: boolean;
  /** Voice name (e.g., es_ES-sharvard-medium) */
  voice: string;
  /** Speech speed multiplier (0.5 = slow, 1.0 = normal, 2.0 = fast) */
  speed: number;
}

export interface VoiceState {
  /** Whether voice synthesis is available on the server */
  isAvailable: boolean;
  /** Whether audio is currently playing */
  isPlaying: boolean;
  /** Whether audio is being loaded/synthesized */
  isLoading: boolean;
  /** Current error message if any */
  error: string | null;
  /** Queue of audio chunks waiting to play */
  queueLength: number;
}

export interface UseVoiceStreamReturn {
  /** Current voice state */
  state: VoiceState;
  /** Current voice configuration */
  config: VoiceConfig;
  /** Synthesize and play text */
  speak: (text: string) => Promise<void>;
  /** Stop current playback */
  stop: () => void;
  /** Pause playback */
  pause: () => void;
  /** Resume playback */
  resume: () => void;
  /** Update voice configuration */
  updateConfig: (config: Partial<VoiceConfig>) => void;
  /** Check if voice service is available */
  checkAvailability: () => Promise<boolean>;
}

// =============================================================================
// Constants
// =============================================================================

// Voice service URL (proxied through Vite in dev, direct in production)
const VOICE_SERVICE_URL = '/voice';
const SAMPLE_RATE = 44100; // High quality audio at 44.1kHz (CD quality)
const OPUS_SAMPLE_RATE = 48000; // Opus encoder uses 48kHz (highest quality)

// Use Opus format for ~10x smaller payload and faster streaming
const USE_OPUS = true;

// Default configuration
const DEFAULT_CONFIG: VoiceConfig = {
  enabled: true,
  volume: 0.8,
  autoPlay: true,
  voice: 'es_MX-claude-high',
  speed: 1,
};

// =============================================================================
// Hook Implementation
// =============================================================================

export function useVoiceStream(initialConfig?: Partial<VoiceConfig>): UseVoiceStreamReturn {
  // State
  const [state, setState] = useState<VoiceState>({
    isAvailable: false,
    isPlaying: false,
    isLoading: false,
    error: null,
    queueLength: 0,
  });

  const [config, setConfig] = useState<VoiceConfig>({
    ...DEFAULT_CONFIG,
    ...initialConfig,
  });

  // Refs for audio context and queue management
  const audioContextRef = useRef<AudioContext | null>(null);
  const gainNodeRef = useRef<GainNode | null>(null);
  const audioQueueRef = useRef<AudioBuffer[]>([]);
  const isPlayingRef = useRef(false);
  const currentSourceRef = useRef<AudioBufferSourceNode | null>(null);
  const abortControllerRef = useRef<AbortController | null>(null);

  // ==========================================================================
  // Audio Context Initialization
  // ==========================================================================

  const initAudioContext = useCallback(() => {
    if (!audioContextRef.current) {
      // Use appropriate sample rate based on format
      const sampleRate = USE_OPUS ? OPUS_SAMPLE_RATE : SAMPLE_RATE;
      audioContextRef.current = new (globalThis.AudioContext || (globalThis as any).webkitAudioContext)({
        sampleRate: sampleRate,
      });

      // Create gain node for volume control
      gainNodeRef.current = audioContextRef.current.createGain();
      gainNodeRef.current.connect(audioContextRef.current.destination);
      gainNodeRef.current.gain.value = config.volume;
    }

    // Resume context if suspended (required by browser autoplay policies)
    if (audioContextRef.current.state === 'suspended') {
      audioContextRef.current.resume();
    }

    return audioContextRef.current;
  }, [config.volume]);

  // ==========================================================================
  // Audio Queue Processing
  // ==========================================================================

  const processQueue = useCallback(async () => {
    if (isPlayingRef.current || audioQueueRef.current.length === 0) {
      return;
    }

    const audioContext = audioContextRef.current;
    const gainNode = gainNodeRef.current;

    if (!audioContext || !gainNode) return;

    isPlayingRef.current = true;
    setState(prev => ({ ...prev, isPlaying: true }));

    while (audioQueueRef.current.length > 0) {
      const audioBuffer = audioQueueRef.current.shift()!;
      
      setState(prev => ({ ...prev, queueLength: audioQueueRef.current.length }));

      // Create source node
      const source = audioContext.createBufferSource();
      source.buffer = audioBuffer;
      source.connect(gainNode);
      currentSourceRef.current = source;

      // Wait for playback to complete
      await new Promise<void>((resolve) => {
        source.onended = () => resolve();
        source.start(0);
      });

      currentSourceRef.current = null;
    }

    isPlayingRef.current = false;
    setState(prev => ({ ...prev, isPlaying: false, queueLength: 0 }));
  }, []);

  // ==========================================================================
  // Audio Decoding (WAV and OGG/Opus)
  // ==========================================================================

  // Fallback manual WAV parsing when Web Audio API fails
  const parseWavManually = useCallback((audioData: ArrayBuffer, audioContext: AudioContext): AudioBuffer => {
    const view = new DataView(audioData);
    const numChannels = view.getUint16(22, true);
    const sampleRate = view.getUint32(24, true);
    const bitsPerSample = view.getUint16(34, true);
    
    // Find data chunk
    const dataOffset = 44; // Standard WAV header size
    const dataLength = (audioData.byteLength - dataOffset);
    const numSamples = dataLength / (numChannels * (bitsPerSample / 8));
    
    const buffer = audioContext.createBuffer(numChannels, numSamples, sampleRate);
    
    for (let channel = 0; channel < numChannels; channel++) {
      const channelData = buffer.getChannelData(channel);
      for (let i = 0; i < numSamples; i++) {
        const offset = dataOffset + (i * numChannels + channel) * (bitsPerSample / 8);
        if (bitsPerSample === 16) {
          channelData[i] = view.getInt16(offset, true) / 32768;
        } else if (bitsPerSample === 32) {
          channelData[i] = view.getFloat32(offset, true);
        }
      }
    }
    
    return buffer;
  }, []);

  const decodeAudioToBuffer = useCallback(async (audioData: ArrayBuffer, format: 'wav' | 'opus'): Promise<AudioBuffer> => {
    const audioContext = initAudioContext();
    
    try {
      // Create a copy of the ArrayBuffer because decodeAudioData may detach it
      const audioDataCopy = audioData.slice(0);
      
      // Web Audio API can decode both WAV and OGG/Opus natively
      return await audioContext.decodeAudioData(audioDataCopy);
    } catch (error) {
      console.error(`Failed to decode ${format.toUpperCase()}:`, error);
      
      // Fallback: Manual WAV parsing for simple PCM data (only works for WAV)
      if (format === 'wav') {
        try {
          return parseWavManually(audioData, audioContext);
        } catch (fallbackError) {
          console.error('Fallback WAV decoding also failed:', fallbackError);
          throw new Error('Failed to decode audio data');
        }
      }
      
      throw new Error(`Failed to decode ${format.toUpperCase()} audio data`);
    }
  }, [initAudioContext, parseWavManually]);

  // ==========================================================================
  // Check Availability
  // ==========================================================================

  const checkAvailability = useCallback(async (): Promise<boolean> => {
    try {
      const response = await fetch(`${VOICE_SERVICE_URL}/health`);
      if (!response.ok) {
        setState(prev => ({ ...prev, isAvailable: false }));
        return false;
      }

      const data = await response.json();
      const available = data.status === 'healthy' && data.model_loaded === true;
      setState(prev => ({ ...prev, isAvailable: available }));
      return available;
    } catch (error) {
      console.error('Voice status check failed:', error);
      setState(prev => ({ ...prev, isAvailable: false }));
      return false;
    }
  }, []);

  // ==========================================================================
  // Speak (Synthesize and Play) - Streaming version
  // ==========================================================================

  // Helper: Decode base64 audio data to Uint8Array
  const decodeBase64Audio = useCallback((base64Data: string): Uint8Array => {
    const binaryString = atob(base64Data);
    const bytes = new Uint8Array(binaryString.length);
    for (let i = 0; i < binaryString.length; i++) {
      bytes[i] = binaryString.codePointAt(i) || 0;
    }
    return bytes;
  }, []);

  // Helper: Process SSE audio chunk
  const processAudioChunk = useCallback(async (data: any, audioFormat: 'wav' | 'opus') => {
    if (data.type === 'audio' && data.data) {
      const bytes = decodeBase64Audio(data.data);
      const audioBuffer = await decodeAudioToBuffer(bytes.buffer, audioFormat);
      audioQueueRef.current.push(audioBuffer);
      setState(prev => ({ 
        ...prev, 
        queueLength: audioQueueRef.current.length,
        isLoading: false 
      }));
      processQueue();
    } else if (data.type === 'done') {
      setState(prev => ({ ...prev, isLoading: false }));
    }
  }, [decodeBase64Audio, decodeAudioToBuffer, processQueue]);

  // Helper: Process SSE stream
  const processSSEStream = useCallback(async (reader: ReadableStreamDefaultReader<Uint8Array>, audioFormat: 'wav' | 'opus') => {
    const decoder = new TextDecoder();
    let buffer = '';

    while (true) {
      const { done, value } = await reader.read();
      if (done) break;

      buffer += decoder.decode(value, { stream: true });
      const lines = buffer.split('\n\n');
      buffer = lines.pop() || '';

      for (const line of lines) {
        if (line.startsWith('data: ')) {
          try {
            const data = JSON.parse(line.slice(6));
            await processAudioChunk(data, audioFormat);
          } catch (e) {
            console.error('Failed to parse SSE data:', e);
          }
        }
      }
    }
  }, [processAudioChunk]);

  const speak = useCallback(async (text: string): Promise<void> => {
    if (!config.enabled || !text.trim()) {
      return;
    }

    // Cancel any ongoing request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
    }

    abortControllerRef.current = new AbortController();
    setState(prev => ({ ...prev, isLoading: true, error: null }));

    try {
      initAudioContext();

      // Choose endpoint based on format preference
      const endpoint = USE_OPUS 
        ? `${VOICE_SERVICE_URL}/synthesize/opus`
        : `${VOICE_SERVICE_URL}/synthesize/stream`;
      
      const audioFormat = USE_OPUS ? 'opus' : 'wav';

      // Use streaming endpoint for real-time playback
      const response = await fetch(endpoint, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          text: text,
          voice: config.voice,
          speed: config.speed,
          pitch_shift: 6,  // +6 semitones (Tachikoma voice)
          robot_effect: true,  // Enable robot effect chain
        }),
        signal: abortControllerRef.current.signal,
      });

      if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.detail || 'Voice synthesis failed');
      }

      // Process SSE stream
      const reader = response.body?.getReader();
      if (!reader) {
        throw new Error('No response body');
      }

      await processSSEStream(reader, audioFormat);
      setState(prev => ({ ...prev, isLoading: false }));
    } catch (error: any) {
      if (error.name === 'AbortError') {
        setState(prev => ({ ...prev, isLoading: false }));
        return;
      }

      console.error('Voice synthesis error:', error);
      setState(prev => ({
        ...prev,
        isLoading: false,
        error: error.message || 'Voice synthesis failed',
      }));
    }
  }, [config.enabled, config.voice, config.speed, initAudioContext, processSSEStream]);

  // ==========================================================================
  // Playback Controls
  // ==========================================================================

  const stop = useCallback(() => {
    // Abort ongoing request
    if (abortControllerRef.current) {
      abortControllerRef.current.abort();
      abortControllerRef.current = null;
    }

    // Stop current playback
    if (currentSourceRef.current) {
      currentSourceRef.current.stop();
      currentSourceRef.current = null;
    }

    // Clear queue
    audioQueueRef.current = [];
    isPlayingRef.current = false;

    setState(prev => ({
      ...prev,
      isPlaying: false,
      isLoading: false,
      queueLength: 0,
    }));
  }, []);

  const pause = useCallback(() => {
    if (audioContextRef.current?.state === 'running') {
      audioContextRef.current.suspend();
      setState(prev => ({ ...prev, isPlaying: false }));
    }
  }, []);

  const resume = useCallback(() => {
    if (audioContextRef.current?.state === 'suspended') {
      audioContextRef.current.resume();
      setState(prev => ({ ...prev, isPlaying: true }));
    }
  }, []);

  // ==========================================================================
  // Configuration Update
  // ==========================================================================

  const updateConfig = useCallback((newConfig: Partial<VoiceConfig>) => {
    setConfig(prev => {
      const updated = { ...prev, ...newConfig };

      // Update volume immediately if changed
      if (newConfig.volume !== undefined && gainNodeRef.current) {
        gainNodeRef.current.gain.value = newConfig.volume;
      }

      // Stop playback if disabled
      if (newConfig.enabled === false) {
        stop();
      }

      // Save to localStorage
      localStorage.setItem('voiceConfig', JSON.stringify(updated));

      return updated;
    });
  }, [stop]);

  // ==========================================================================
  // Effects
  // ==========================================================================

  // Load saved config on mount
  useEffect(() => {
    const saved = localStorage.getItem('voiceConfig');
    if (saved) {
      try {
        const parsed = JSON.parse(saved);
        setConfig(prev => ({ ...prev, ...parsed }));
      } catch (e) {
        console.error('Failed to load voice config:', e);
      }
    }

    // Check availability on mount
    checkAvailability();

    // Cleanup on unmount
    return () => {
      stop();
      if (audioContextRef.current) {
        audioContextRef.current.close();
      }
    };
  }, [checkAvailability, stop]);

  // Update gain when volume changes
  useEffect(() => {
    if (gainNodeRef.current) {
      gainNodeRef.current.gain.value = config.volume;
    }
  }, [config.volume]);

  return {
    state,
    config,
    speak,
    stop,
    pause,
    resume,
    updateConfig,
    checkAvailability,
  };
}

export default useVoiceStream;
