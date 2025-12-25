import { useState, useEffect } from 'react';
import { create } from 'zustand';
import { persist, createJSONStorage } from 'zustand/middleware';
import { 
  musicApi, 
  PlaylistDto, 
  PlaylistWithSongsDto, 
  SongDto, 
  EqualizerSettingsDto,
  CreatePlaylistRequest,
  CreateSongRequest,
  UpdatePlaylistRequest,
  UpdateSongRequest,
  YouTubeSearchResultDto
} from '../api/client';

// =============================================================================
// Types
// =============================================================================

export type RepeatMode = 'off' | 'one' | 'all';

export interface PlayerState {
  currentSong: SongDto | null;
  currentPlaylist: PlaylistWithSongsDto | null;
  isPlaying: boolean;
  currentTime: number;
  duration: number;
  volume: number;
  isMuted: boolean;
  shuffle: boolean;
  repeatMode: RepeatMode;
  isLoading: boolean;
  seekTo: number | null; // When set, AudioPlayer will seek to this time and clear it
}

export interface AudioFilters {
  highpassEnabled: boolean;
  lowpassEnabled: boolean;
  highpassFreq: number; // Hz
  lowpassFreq: number;  // Hz
  // Audio effects
  loudnessEnabled: boolean;     // Fletcher-Munson compensation
  bassBoostEnabled: boolean;    // Sub-bass boost (40-80Hz)
  stereoWideEnabled: boolean;   // Stereo widening effect
  vocalEnhancerEnabled: boolean; // 2-4kHz boost
}

export interface MusicState {
  // Data
  playlists: PlaylistDto[];
  currentPlaylistDetail: PlaylistWithSongsDto | null;
  searchResults: YouTubeSearchResultDto[];
  
  // Player state
  player: PlayerState;
  
  // Equalizer
  equalizer: EqualizerSettingsDto;
  
  // Audio filters (highpass/lowpass)
  audioFilters: AudioFilters;
  
  // Spectrum analyzer data (updated from Web Audio API)
  spectrumData: number[];
  
  // UI state
  isLoadingPlaylists: boolean;
  isSearching: boolean;
  error: string | null;
  
  // Queue (for shuffle mode)
  queue: SongDto[];
  queueIndex: number;
}

// =============================================================================
// Initial State
// =============================================================================

const initialPlayerState: PlayerState = {
  currentSong: null,
  currentPlaylist: null,
  isPlaying: false,
  currentTime: 0,
  duration: 0,
  volume: 0.8,
  isMuted: false,
  shuffle: false,
  repeatMode: 'off',
  isLoading: false,
  seekTo: null,
};

const initialEqualizerSettings: EqualizerSettingsDto = {
  enabled: true,
  preset: undefined,
  bands: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
};

const initialAudioFilters: AudioFilters = {
  highpassEnabled: false,
  lowpassEnabled: false,
  highpassFreq: 300,   // Cut below 300Hz - removes bass/sub-bass
  lowpassFreq: 4000,   // Cut above 4kHz - removes highs/brilliance
  // Audio effects
  loudnessEnabled: false,
  bassBoostEnabled: false,
  stereoWideEnabled: false,
  vocalEnhancerEnabled: false,
};

// =============================================================================
// Store
// =============================================================================

interface MusicActions {
  // Playlist actions
  fetchPlaylists: () => Promise<void>;
  fetchPlaylistDetail: (id: string) => Promise<void>;
  createPlaylist: (request: CreatePlaylistRequest) => Promise<PlaylistDto>;
  updatePlaylist: (id: string, request: UpdatePlaylistRequest) => Promise<void>;
  deletePlaylist: (id: string) => Promise<void>;
  
  // Song actions
  addSong: (playlistId: string, request: CreateSongRequest) => Promise<SongDto>;
  updateSong: (playlistId: string, songId: string, request: UpdateSongRequest) => Promise<void>;
  deleteSong: (playlistId: string, songId: string) => Promise<void>;
  reorderSongs: (playlistId: string, songIds: string[]) => Promise<void>;
  
  // Player actions
  playSong: (song: SongDto, playlist?: PlaylistWithSongsDto) => void;
  togglePlay: () => void;
  pause: () => void;
  resume: () => void;
  stop: () => void;
  nextSong: () => void;
  previousSong: () => void;
  seek: (time: number) => void;
  clearSeek: () => void;
  setVolume: (volume: number) => void;
  toggleMute: () => void;
  toggleShuffle: () => void;
  setRepeatMode: (mode: RepeatMode) => void;
  setPlayerLoading: (loading: boolean) => void;
  setCurrentTime: (time: number) => void;
  setDuration: (duration: number) => void;
  
  // Equalizer actions
  fetchEqualizer: () => Promise<void>;
  updateEqualizer: (settings: EqualizerSettingsDto) => Promise<void>;
  setEqualizerBand: (band: number, value: number) => void;
  loadEqualizerPreset: (preset: string) => Promise<void>;
  
  // Spectrum
  setSpectrumData: (data: number[]) => void;
  
  // Audio filters
  toggleHighpass: () => void;
  toggleLowpass: () => void;
  setHighpassFreq: (freq: number) => void;
  setLowpassFreq: (freq: number) => void;
  // Audio effects
  toggleLoudness: () => void;
  toggleBassBoost: () => void;
  toggleStereoWide: () => void;
  toggleVocalEnhancer: () => void;
  
  // Search
  searchYouTube: (query: string) => Promise<void>;
  clearSearch: () => void;
  
  // Error handling
  setError: (error: string | null) => void;
  clearError: () => void;
}

export const useMusicStore = create<MusicState & MusicActions>()(
  persist(
    (set, get) => ({
  // Initial state
  playlists: [],
  currentPlaylistDetail: null,
  searchResults: [],
  player: initialPlayerState,
  equalizer: initialEqualizerSettings,
  audioFilters: initialAudioFilters,
  spectrumData: new Array(32).fill(0),
  isLoadingPlaylists: false,
  isSearching: false,
  error: null,
  queue: [],
  queueIndex: 0,

  // ==========================================================================
  // Playlist Actions
  // ==========================================================================

  fetchPlaylists: async () => {
    set({ isLoadingPlaylists: true, error: null });
    try {
      const playlists = await musicApi.listPlaylists();
      set({ playlists, isLoadingPlaylists: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Failed to fetch playlists',
        isLoadingPlaylists: false 
      });
    }
  },

  fetchPlaylistDetail: async (id: string) => {
    set({ error: null });
    try {
      const playlist = await musicApi.getPlaylist(id);
      set({ currentPlaylistDetail: playlist });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to fetch playlist' });
    }
  },

  createPlaylist: async (request: CreatePlaylistRequest) => {
    try {
      const playlist = await musicApi.createPlaylist(request);
      set(state => ({ playlists: [playlist, ...state.playlists] }));
      return playlist;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to create playlist' });
      throw error;
    }
  },

  updatePlaylist: async (id: string, request: UpdatePlaylistRequest) => {
    try {
      const updated = await musicApi.updatePlaylist(id, request);
      set(state => ({
        playlists: state.playlists.map(p => p.id === id ? { ...p, ...updated } : p),
        currentPlaylistDetail: state.currentPlaylistDetail?.id === id 
          ? { ...state.currentPlaylistDetail, ...updated } 
          : state.currentPlaylistDetail,
      }));
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to update playlist' });
      throw error;
    }
  },

  deletePlaylist: async (id: string) => {
    try {
      await musicApi.deletePlaylist(id);
      set(state => ({
        playlists: state.playlists.filter(p => p.id !== id),
        currentPlaylistDetail: state.currentPlaylistDetail?.id === id 
          ? null 
          : state.currentPlaylistDetail,
      }));
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete playlist' });
      throw error;
    }
  },

  // ==========================================================================
  // Song Actions
  // ==========================================================================

  addSong: async (playlistId: string, request: CreateSongRequest) => {
    try {
      const song = await musicApi.addSong(playlistId, request);
      set(state => {
        if (state.currentPlaylistDetail?.id === playlistId) {
          return {
            currentPlaylistDetail: {
              ...state.currentPlaylistDetail,
              songs: [...state.currentPlaylistDetail.songs, song],
              song_count: state.currentPlaylistDetail.song_count + 1,
              total_duration: state.currentPlaylistDetail.total_duration + song.duration,
            }
          };
        }
        return state;
      });
      return song;
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to add song' });
      throw error;
    }
  },

  updateSong: async (playlistId: string, songId: string, request: UpdateSongRequest) => {
    try {
      const updated = await musicApi.updateSong(playlistId, songId, request);
      set(state => {
        if (state.currentPlaylistDetail?.id === playlistId) {
          return {
            currentPlaylistDetail: {
              ...state.currentPlaylistDetail,
              songs: state.currentPlaylistDetail.songs.map(s => 
                s.id === songId ? updated : s
              ),
            }
          };
        }
        return state;
      });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to update song' });
      throw error;
    }
  },

  deleteSong: async (playlistId: string, songId: string) => {
    try {
      const song = get().currentPlaylistDetail?.songs.find(s => s.id === songId);
      await musicApi.deleteSong(playlistId, songId);
      set(state => {
        if (state.currentPlaylistDetail?.id === playlistId) {
          return {
            currentPlaylistDetail: {
              ...state.currentPlaylistDetail,
              songs: state.currentPlaylistDetail.songs.filter(s => s.id !== songId),
              song_count: state.currentPlaylistDetail.song_count - 1,
              total_duration: state.currentPlaylistDetail.total_duration - (song?.duration || 0),
            }
          };
        }
        return state;
      });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to delete song' });
      throw error;
    }
  },

  reorderSongs: async (playlistId: string, songIds: string[]) => {
    try {
      await musicApi.reorderSongs(playlistId, songIds);
      set(state => {
        if (state.currentPlaylistDetail?.id === playlistId) {
          const reorderedSongs = songIds
            .map(id => state.currentPlaylistDetail!.songs.find(s => s.id === id))
            .filter((s): s is SongDto => s !== undefined)
            .map((s, i) => ({ ...s, song_order: i }));
          return {
            currentPlaylistDetail: {
              ...state.currentPlaylistDetail,
              songs: reorderedSongs,
            }
          };
        }
        return state;
      });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to reorder songs' });
      throw error;
    }
  },

  // ==========================================================================
  // Player Actions
  // ==========================================================================

  playSong: (song: SongDto, playlist?: PlaylistWithSongsDto) => {
    const state = get();
    const targetPlaylist = playlist || state.currentPlaylistDetail;
    
    // Build queue
    let queue: SongDto[] = [];
    let queueIndex = 0;
    
    if (targetPlaylist) {
      if (state.player.shuffle) {
        // Shuffle the queue but keep current song first
        const otherSongs = targetPlaylist.songs.filter(s => s.id !== song.id);
        const shuffled = [...otherSongs].sort(() => Math.random() - 0.5);
        queue = [song, ...shuffled];
        queueIndex = 0;
      } else {
        queue = targetPlaylist.songs;
        queueIndex = queue.findIndex(s => s.id === song.id);
      }
    } else {
      queue = [song];
      queueIndex = 0;
    }

    set({
      player: {
        ...state.player,
        currentSong: song,
        currentPlaylist: targetPlaylist || null,
        isPlaying: true,
        currentTime: 0,
        isLoading: true,
      },
      queue,
      queueIndex,
    });
  },

  togglePlay: () => {
    set(state => ({
      player: {
        ...state.player,
        isPlaying: !state.player.isPlaying,
      }
    }));
  },

  pause: () => {
    set(state => ({
      player: { ...state.player, isPlaying: false }
    }));
  },

  resume: () => {
    set(state => ({
      player: { ...state.player, isPlaying: true }
    }));
  },

  stop: () => {
    set(state => ({
      player: {
        ...state.player,
        isPlaying: false,
        currentTime: 0,
      }
    }));
  },

  nextSong: () => {
    const { queue, queueIndex, player } = get();
    
    if (queue.length === 0) return;
    
    let nextIndex = queueIndex + 1;
    
    if (nextIndex >= queue.length) {
      if (player.repeatMode === 'all') {
        nextIndex = 0;
      } else {
        // End of playlist
        set(state => ({
          player: { ...state.player, isPlaying: false }
        }));
        return;
      }
    }

    const nextSong = queue[nextIndex];
    set(state => ({
      player: {
        ...state.player,
        currentSong: nextSong,
        isPlaying: true,
        currentTime: 0,
        isLoading: true,
      },
      queueIndex: nextIndex,
    }));
  },

  previousSong: () => {
    const { queue, queueIndex, player } = get();
    
    if (queue.length === 0) return;
    
    // If more than 3 seconds into song, restart it
    if (player.currentTime > 3) {
      set(state => ({
        player: { ...state.player, currentTime: 0 }
      }));
      return;
    }
    
    let prevIndex = queueIndex - 1;
    
    if (prevIndex < 0) {
      if (player.repeatMode === 'all') {
        prevIndex = queue.length - 1;
      } else {
        prevIndex = 0;
      }
    }

    const prevSong = queue[prevIndex];
    set(state => ({
      player: {
        ...state.player,
        currentSong: prevSong,
        isPlaying: true,
        currentTime: 0,
        isLoading: true,
      },
      queueIndex: prevIndex,
    }));
  },

  seek: (time: number) => {
    set(state => ({
      player: { ...state.player, currentTime: time, seekTo: time }
    }));
  },

  clearSeek: () => {
    set(state => ({
      player: { ...state.player, seekTo: null }
    }));
  },

  setVolume: (volume: number) => {
    set(state => ({
      player: { 
        ...state.player, 
        volume: Math.max(0, Math.min(1, volume)),
        isMuted: false,
      }
    }));
  },

  toggleMute: () => {
    set(state => ({
      player: { ...state.player, isMuted: !state.player.isMuted }
    }));
  },

  toggleShuffle: () => {
    const state = get();
    const newShuffle = !state.player.shuffle;
    
    // Rebuild queue
    if (state.player.currentPlaylist && state.player.currentSong) {
      let queue: SongDto[];
      let queueIndex: number;
      
      if (newShuffle) {
        const otherSongs = state.player.currentPlaylist.songs
          .filter(s => s.id !== state.player.currentSong!.id);
        const shuffled = [...otherSongs].sort(() => Math.random() - 0.5);
        queue = [state.player.currentSong, ...shuffled];
        queueIndex = 0;
      } else {
        queue = state.player.currentPlaylist.songs;
        queueIndex = queue.findIndex(s => s.id === state.player.currentSong!.id);
      }
      
      set({
        player: { ...state.player, shuffle: newShuffle },
        queue,
        queueIndex,
      });
    } else {
      set(state => ({
        player: { ...state.player, shuffle: newShuffle }
      }));
    }
  },

  setRepeatMode: (mode: RepeatMode) => {
    set(state => ({
      player: { ...state.player, repeatMode: mode }
    }));
  },

  setPlayerLoading: (loading: boolean) => {
    set(state => ({
      player: { ...state.player, isLoading: loading }
    }));
  },

  setCurrentTime: (time: number) => {
    set(state => ({
      player: { ...state.player, currentTime: time }
    }));
  },

  setDuration: (duration: number) => {
    set(state => ({
      player: { ...state.player, duration }
    }));
  },

  // ==========================================================================
  // Equalizer Actions
  // ==========================================================================

  fetchEqualizer: async () => {
    // Don't fetch from backend - equalizer settings are local preferences
    // They are already persisted in localStorage via Zustand persist
    // Only fetch if we somehow have no bands set (all zeros check)
    const currentBands = useMusicStore.getState().equalizer.bands;
    const hasCustomBands = currentBands.some(b => b !== 0);
    
    if (hasCustomBands) {
      console.log('🎛️ Using persisted EQ settings, skipping backend fetch');
      return;
    }
    
    // Only fetch from backend as fallback if bands are all zeros
    try {
      const settings = await musicApi.getEqualizer();
      // Only apply if current bands are all zeros
      if (!hasCustomBands) {
        set({ equalizer: settings });
      }
    } catch {
      // Keep current settings on error
      console.log('🎛️ Backend fetch failed, keeping current EQ settings');
    }
  },

  updateEqualizer: async (settings: EqualizerSettingsDto) => {
    try {
      await musicApi.updateEqualizer(settings);
      set({ equalizer: settings });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to update equalizer' });
    }
  },

  setEqualizerBand: (band: number, value: number) => {
    set(state => {
      const newBands = [...state.equalizer.bands];
      newBands[band] = Math.max(-12, Math.min(12, value));
      console.log(`🎚️ Setting band ${band} to ${newBands[band]}dB`);
      return {
        equalizer: {
          ...state.equalizer,
          bands: newBands,
          preset: undefined, // Clear preset when manually adjusting
        }
      };
    });
  },

  loadEqualizerPreset: async (preset: string) => {
    try {
      const settings = await musicApi.getEqualizerPreset(preset);
      set({ equalizer: settings });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to load preset' });
    }
  },

  // ==========================================================================
  // Spectrum
  // ==========================================================================

  setSpectrumData: (data: number[]) => {
    set({ spectrumData: data });
  },

  // ==========================================================================
  // Audio Filters
  // ==========================================================================

  toggleHighpass: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, highpassEnabled: !state.audioFilters.highpassEnabled }
    }));
  },

  toggleLowpass: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, lowpassEnabled: !state.audioFilters.lowpassEnabled }
    }));
  },

  setHighpassFreq: (freq: number) => {
    set(state => ({
      audioFilters: { ...state.audioFilters, highpassFreq: freq }
    }));
  },

  setLowpassFreq: (freq: number) => {
    set(state => ({
      audioFilters: { ...state.audioFilters, lowpassFreq: freq }
    }));
  },

  toggleLoudness: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, loudnessEnabled: !state.audioFilters.loudnessEnabled }
    }));
  },

  toggleBassBoost: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, bassBoostEnabled: !state.audioFilters.bassBoostEnabled }
    }));
  },

  toggleStereoWide: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, stereoWideEnabled: !state.audioFilters.stereoWideEnabled }
    }));
  },

  toggleVocalEnhancer: () => {
    set(state => ({
      audioFilters: { ...state.audioFilters, vocalEnhancerEnabled: !state.audioFilters.vocalEnhancerEnabled }
    }));
  },

  // ==========================================================================
  // Search
  // ==========================================================================

  searchYouTube: async (query: string) => {
    if (!query.trim()) {
      set({ searchResults: [] });
      return;
    }
    
    set({ isSearching: true, error: null });
    try {
      const results = await musicApi.searchYouTube(query);
      set({ searchResults: results, isSearching: false });
    } catch (error) {
      set({ 
        error: error instanceof Error ? error.message : 'Search failed',
        isSearching: false 
      });
    }
  },

  clearSearch: () => {
    set({ searchResults: [] });
  },

  // ==========================================================================
  // Error Handling
  // ==========================================================================

  setError: (error: string | null) => {
    set({ error });
  },

  clearError: () => {
    set({ error: null });
  },
}),
    {
      name: 'music-settings',
      version: 5, // Force fresh state
      storage: createJSONStorage(() => localStorage),
      partialize: (state) => ({
        // Only persist equalizer and filter settings, volume
        equalizer: state.equalizer,
        audioFilters: state.audioFilters,
        player: {
          volume: state.player.volume,
          isMuted: state.player.isMuted,
          shuffle: state.player.shuffle,
          repeatMode: state.player.repeatMode,
        },
      }),
      onRehydrateStorage: () => (state, error) => {
        if (error) {
          console.error('🎵 Error rehydrating music settings:', error);
          return;
        }
        console.log('🎵 Music settings rehydrated:', {
          equalizerEnabled: state?.equalizer?.enabled,
          equalizerBands: state?.equalizer?.bands,
          hpf: state?.audioFilters?.highpassEnabled,
          lpf: state?.audioFilters?.lowpassEnabled,
          volume: state?.player?.volume,
        });
      },
    }
  )
);

// Expose hydration state check
export const useHasHydrated = () => {
  const [hasHydrated, setHasHydrated] = useState(
    useMusicStore.persist.hasHydrated()
  );

  useEffect(() => {
    const unsub = useMusicStore.persist.onFinishHydration(() => {
      setHasHydrated(true);
    });
    return () => unsub();
  }, []);

  return hasHydrated;
};

// =============================================================================
// Utility Functions
// =============================================================================

export function formatDuration(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

export function formatDurationLong(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins} min`;
}

// Equalizer frequency labels
export const EQUALIZER_FREQUENCIES = [
  '32', '64', '125', '250', '500', '1K', '2K', '4K', '8K', '16K',
  '20', '45', '90', '180', '350', '700'
];

export const EQUALIZER_PRESETS = [
  { name: 'flat', label: 'Flat' },
  { name: 'bass_boost', label: 'Bass Boost' },
  { name: 'treble_boost', label: 'Treble Boost' },
  { name: 'vocal', label: 'Vocal' },
  { name: 'rock', label: 'Rock' },
  { name: 'electronic', label: 'Electronic' },
  { name: 'acoustic', label: 'Acoustic' },
];
