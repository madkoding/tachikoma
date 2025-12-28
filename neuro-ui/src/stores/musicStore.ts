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
  EnrichedSearchResultDto
} from '../api/client';
import { songCache } from '../services/songCache';

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
  searchResults: EnrichedSearchResultDto[];
  
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
  
  // New songs animation tracking
  newSongIds: Set<string>;
  pollingPlaylistId: string | null;
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
  bands: [0, 0, 0, 0, 0, 0, 0, 0], // 8 bands
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
  
  // Polling for new songs (fallback for SSE)
  startPolling: (playlistId: string) => void;
  stopPolling: () => void;
  clearNewSongIds: () => void;
  markSongAsSeen: (songId: string) => void;
  
  // SSE-based watching (preferred)
  startWatchingPlaylist: (playlistId: string) => void;
  stopWatchingPlaylist: () => void;
  
  // Special playlists (Me gusta / Sugerencias)
  initSpecialPlaylists: () => Promise<void>;
  toggleSongLike: (songId: string) => Promise<void>;
  fetchSongCover: (songId: string) => Promise<void>;
  refreshSuggestions: () => Promise<void>;
}

// Polling interval reference (outside store)
let pollingInterval: ReturnType<typeof setInterval> | null = null;

// SSE EventSource reference (outside store)
let sseEventSource: EventSource | null = null;
let sseWatchingPlaylistId: string | null = null;

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
  newSongIds: new Set<string>(),
  pollingPlaylistId: null,

  // ==========================================================================
  // Playlist Actions
  // ==========================================================================

  fetchPlaylists: async () => {
    set({ isLoadingPlaylists: true, error: null });
    try {
      // Initialize special playlists first (creates them if they don't exist)
      try {
        await musicApi.initSpecialPlaylists();
      } catch (e) {
        console.warn('Could not init special playlists:', e);
      }
      const playlists = await musicApi.listPlaylists();
      
      // Sort playlists: is_favorites first, is_suggestions second, then by created_at desc
      const sortedPlaylists = [...playlists].sort((a, b) => {
        // is_favorites always first
        if (a.is_favorites && !b.is_favorites) return -1;
        if (!a.is_favorites && b.is_favorites) return 1;
        // is_suggestions second
        if (a.is_suggestions && !b.is_suggestions) return -1;
        if (!a.is_suggestions && b.is_suggestions) return 1;
        // Rest by created_at descending (newest first)
        return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
      });
      
      set({ playlists: sortedPlaylists, isLoadingPlaylists: false });
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
      // Handle API error responses with message
      if (error && typeof error === 'object' && 'response' in error) {
        const axiosError = error as { response?: { status?: number; data?: { error?: string } } };
        const errorMessage = axiosError.response?.data?.error || 'Error al agregar canción';
        set({ error: errorMessage });
        throw new Error(errorMessage);
      }
      set({ error: error instanceof Error ? error.message : 'Error al agregar canción' });
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
      // Use enriched search to get proper metadata (artist, title, album)
      const results = await musicApi.searchYouTubeEnriched(query);
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

  // ==========================================================================
  // Polling for New Songs
  // ==========================================================================

  startPolling: (playlistId: string) => {
    console.log('🎵 Starting polling for playlist:', playlistId);
    // Stop any existing polling
    if (pollingInterval) {
      clearInterval(pollingInterval);
    }
    
    set({ pollingPlaylistId: playlistId });
    
    // Poll every 2 seconds
    pollingInterval = setInterval(async () => {
      const state = get();
      if (state.pollingPlaylistId !== playlistId) {
        return;
      }
      
      try {
        const playlist = await musicApi.getPlaylist(playlistId);
        const currentSongs = state.currentPlaylistDetail?.songs || [];
        const currentSongIds = new Set(currentSongs.map(s => s.id));
        
        // Find new songs
        const newSongs = playlist.songs.filter(s => !currentSongIds.has(s.id));
        
        console.log(`🎵 Polling: ${playlist.songs.length} songs in playlist, ${newSongs.length} new, cover: ${playlist.cover_url ? 'yes' : 'no'}`);
        
        if (newSongs.length > 0 || playlist.cover_url !== state.playlists.find(p => p.id === playlistId)?.cover_url) {
          console.log('🎵 New songs detected:', newSongs.map(s => s.title));
          // Add new song IDs to the set for animation
          const newIds = new Set(state.newSongIds);
          newSongs.forEach(s => newIds.add(s.id));
          
          set({ 
            currentPlaylistDetail: playlist,
            newSongIds: newIds,
            // Also update the playlist in the list (including cover_url)
            playlists: state.playlists.map(p => 
              p.id === playlistId 
                ? { ...p, song_count: playlist.song_count, total_duration: playlist.total_duration, cover_url: playlist.cover_url }
                : p
            ),
          });
        }
        
        // Stop polling if playlist has 10+ songs (likely done)
        if (playlist.songs.length >= 10) {
          console.log('🎵 Polling: Playlist complete, stopping polling');
          get().stopPolling();
        }
      } catch (error) {
        console.error('Polling error:', error);
      }
    }, 2000);
  },

  stopPolling: () => {
    if (pollingInterval) {
      clearInterval(pollingInterval);
      pollingInterval = null;
    }
    set({ pollingPlaylistId: null });
  },

  clearNewSongIds: () => {
    set({ newSongIds: new Set() });
  },

  markSongAsSeen: (songId: string) => {
    set(state => {
      const newIds = new Set(state.newSongIds);
      newIds.delete(songId);
      return { newSongIds: newIds };
    });
  },

  // ==========================================================================
  // SSE-based Watching (preferred over polling)
  // ==========================================================================

  startWatchingPlaylist: (playlistId: string) => {
    console.log('🎵 Starting to watch playlist via SSE:', playlistId);
    
    // Stop any existing watching
    get().stopWatchingPlaylist();
    
    sseWatchingPlaylistId = playlistId;
    set({ pollingPlaylistId: playlistId });
    
    // Try to connect to SSE
    try {
      sseEventSource = new EventSource('/api/music/events');
      
      sseEventSource.onopen = () => {
        console.log('🎵 SSE connected for playlist watching');
      };
      
      sseEventSource.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          console.log('🎵 SSE event received:', data.type);
          
          // Handle SongAdded event
          if (data.type === 'SongAdded') {
            const { playlist_id, song: rawSong } = data.data;
            const song = rawSong as SongDto;
            console.log('🎵 SSE: New song added:', song.title, 'to playlist:', playlist_id);
            
            // Add to newSongIds for animation
            set(state => {
              const newIds = new Set(state.newSongIds);
              newIds.add(song.id);
              return { newSongIds: newIds };
            });
            
            // Immediately update the state if watching this playlist
            if (playlist_id === sseWatchingPlaylistId) {
              set(state => {
                const detail = state.currentPlaylistDetail;
                if (detail && detail.id === playlist_id) {
                  // Check if song already exists to avoid duplicates
                  const existingSong = detail.songs.find(s => s.id === song.id);
                  if (existingSong) {
                    console.log('🎵 SSE: Song already exists, skipping');
                    return state;
                  }
                  
                  console.log('🎵 SSE: Adding song to currentPlaylistDetail');
                  return {
                    currentPlaylistDetail: {
                      ...detail,
                      songs: [...detail.songs, song],
                      song_count: detail.song_count + 1,
                      total_duration: detail.total_duration + (song.duration || 0),
                    }
                  };
                }
                return state;
              });
            }
            
            // Also refresh playlists list (for song count in sidebar)
            get().fetchPlaylists();
          }
          
          // Handle PlaylistUpdated event
          if (data.type === 'PlaylistUpdated') {
            const playlist = data.data as { id: string; name: string; description?: string; cover_url?: string; song_count: number; total_duration: number };
            console.log('🎵 SSE: Playlist updated:', playlist.id);
            
            // Update the playlist in the list
            set(state => {
              const detail = state.currentPlaylistDetail;
              return {
                playlists: state.playlists.map(p => 
                  p.id === playlist.id 
                    ? { ...p, name: playlist.name, description: playlist.description, cover_url: playlist.cover_url, song_count: playlist.song_count, total_duration: playlist.total_duration }
                    : p
                ),
                // Update currentPlaylistDetail metadata if it's the same playlist
                currentPlaylistDetail: detail && detail.id === playlist.id
                  ? { ...detail, name: playlist.name, description: playlist.description, cover_url: playlist.cover_url, song_count: playlist.song_count, total_duration: playlist.total_duration }
                  : detail,
              };
            });
          }
          
          // Handle PlaylistCreated - refresh list
          if (data.type === 'PlaylistCreated') {
            console.log('🎵 SSE: New playlist created');
            get().fetchPlaylists();
          }
          
          // Handle SongRemoved event
          if (data.type === 'SongRemoved') {
            const { playlist_id, song_id } = data.data as { playlist_id: string; song_id: string };
            console.log('🎵 SSE: Song removed:', song_id, 'from playlist:', playlist_id);
            
            if (playlist_id === sseWatchingPlaylistId) {
              set(state => {
                const detail = state.currentPlaylistDetail;
                if (detail && detail.id === playlist_id) {
                  const removedSong = detail.songs.find(s => s.id === song_id);
                  return {
                    currentPlaylistDetail: {
                      ...detail,
                      songs: detail.songs.filter(s => s.id !== song_id),
                      song_count: Math.max(0, detail.song_count - 1),
                      total_duration: Math.max(0, detail.total_duration - (removedSong?.duration || 0)),
                    }
                  };
                }
                return state;
              });
            }
            get().fetchPlaylists();
          }
          
          // Handle SongUpdated event
          if (data.type === 'SongUpdated') {
            const updatedSong = data.data as SongDto;
            console.log('🎵 SSE: Song updated:', updatedSong.id);
            
            set(state => {
              const detail = state.currentPlaylistDetail;
              if (detail && detail.id === updatedSong.playlist_id) {
                return {
                  currentPlaylistDetail: {
                    ...detail,
                    songs: detail.songs.map(s => 
                      s.id === updatedSong.id ? { ...s, ...updatedSong } : s
                    ),
                  }
                };
              }
              return state;
            });
          }
          
        } catch (e) {
          console.error('🎵 SSE: Failed to parse event:', e);
        }
      };
      
      sseEventSource.onerror = (e) => {
        console.log('🎵 SSE error, falling back to polling:', e);
        
        // Close SSE connection
        if (sseEventSource) {
          sseEventSource.close();
          sseEventSource = null;
        }
        
        // Fall back to polling
        if (sseWatchingPlaylistId) {
          console.log('🎵 Starting polling fallback for:', sseWatchingPlaylistId);
          get().startPolling(sseWatchingPlaylistId);
        }
      };
      
    } catch (e) {
      console.error('🎵 Failed to create EventSource, using polling:', e);
      get().startPolling(playlistId);
    }
  },

  stopWatchingPlaylist: () => {
    console.log('🎵 Stopping playlist watching');
    
    // Stop SSE
    if (sseEventSource) {
      sseEventSource.close();
      sseEventSource = null;
    }
    sseWatchingPlaylistId = null;
    
    // Stop polling
    get().stopPolling();
  },

  // ==========================================================================
  // Special Playlists (Me gusta / Sugerencias)
  // ==========================================================================

  initSpecialPlaylists: async () => {
    try {
      await musicApi.initSpecialPlaylists();
      // Refresh playlists to include the new special ones
      await get().fetchPlaylists();
    } catch (error) {
      console.error('Failed to init special playlists:', error);
    }
  },

  toggleSongLike: async (songId: string) => {
    console.log('🎵 toggleSongLike called with songId:', songId);
    try {
      const updated = await musicApi.toggleSongLike(songId);
      console.log('🎵 toggleSongLike response:', updated);
      
      set(state => {
        // Update in current playlist detail
        const updatedPlaylistDetail = state.currentPlaylistDetail 
          ? {
              ...state.currentPlaylistDetail,
              songs: state.currentPlaylistDetail.songs.map(s =>
                s.id === songId ? { ...s, is_liked: updated.is_liked } : s
              ),
            }
          : null;
        
        // Update current song if it's the one being toggled
        const updatedPlayer = state.player.currentSong?.id === songId
          ? {
              ...state.player,
              currentSong: { ...state.player.currentSong, is_liked: updated.is_liked },
            }
          : state.player;
        
        return {
          currentPlaylistDetail: updatedPlaylistDetail,
          player: updatedPlayer,
        };
      });
      
      // Refresh playlists to update "Me gusta" count
      await get().fetchPlaylists();
      console.log('🎵 toggleSongLike completed successfully, is_liked:', updated.is_liked);

      // If liked, download the song to cache in background
      if (updated.is_liked) {
        console.log('🎵 Starting background download for liked song:', songId);
        musicApi.downloadSong(songId)
          .then(async (blob) => {
            await songCache.put(songId, updated.youtube_id || songId, blob);
            console.log('🎵 Song cached successfully:', songId);
          })
          .catch(error => {
            console.warn('🎵 Failed to cache song (will stream instead):', error);
          });
      } else {
        // If unliked, remove from cache
        songCache.remove(songId)
          .then(() => console.log('🎵 Song removed from cache:', songId))
          .catch(error => console.warn('🎵 Failed to remove from cache:', error));
      }
    } catch (error) {
      console.error('🎵 toggleSongLike error:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to toggle like' });
      throw error;
    }
  },

  fetchSongCover: async (songId: string) => {
    try {
      const updated = await musicApi.fetchSongCover(songId);
      set(state => {
        // Update in current playlist detail
        const updatedPlaylistDetail = state.currentPlaylistDetail 
          ? {
              ...state.currentPlaylistDetail,
              songs: state.currentPlaylistDetail.songs.map(s =>
                s.id === songId ? { ...s, cover_url: updated.cover_url } : s
              ),
            }
          : null;
        
        // Update current song if it's the one being updated
        const updatedPlayer = state.player.currentSong?.id === songId
          ? {
              ...state.player,
              currentSong: { ...state.player.currentSong, cover_url: updated.cover_url },
            }
          : state.player;
        
        return {
          currentPlaylistDetail: updatedPlaylistDetail,
          player: updatedPlayer,
        };
      });
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to fetch cover' });
      throw error;
    }
  },

  refreshSuggestions: async () => {
    try {
      const updated = await musicApi.refreshSuggestions();
      set(state => ({
        playlists: state.playlists.map(p =>
          p.id === updated.id ? { ...p, ...updated } : p
        ),
        currentPlaylistDetail: state.currentPlaylistDetail?.id === updated.id
          ? updated
          : state.currentPlaylistDetail,
      }));
    } catch (error) {
      set({ error: error instanceof Error ? error.message : 'Failed to refresh suggestions' });
      throw error;
    }
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
// Optimized Selectors (prevent re-renders from spectrumData updates ~60/s)
// =============================================================================

// Shallow equality check for player state
const shallowEqualPlayer = (a: PlayerState, b: PlayerState) => 
  a.currentSong === b.currentSong &&
  a.currentPlaylist === b.currentPlaylist &&
  a.isPlaying === b.isPlaying &&
  a.currentTime === b.currentTime &&
  a.duration === b.duration &&
  a.volume === b.volume &&
  a.isMuted === b.isMuted &&
  a.shuffle === b.shuffle &&
  a.repeatMode === b.repeatMode &&
  a.isLoading === b.isLoading &&
  a.seekTo === b.seekTo;

// Player state only (most components need this)
export const usePlayerState = () => useMusicStore((state) => state.player, shallowEqualPlayer);

// Player actions (stable references, don't cause re-renders)
export const usePlayerActions = () => useMusicStore((state) => ({
  togglePlay: state.togglePlay,
  nextSong: state.nextSong,
  previousSong: state.previousSong,
  seek: state.seek,
  setVolume: state.setVolume,
  toggleMute: state.toggleMute,
  toggleShuffle: state.toggleShuffle,
  cycleRepeatMode: state.cycleRepeatMode,
  playSong: state.playSong,
}));

// Spectrum data only (for SpectrumAnalyzer - updates ~60/s)
export const useSpectrumData = () => useMusicStore((state) => state.spectrumData);

// Playlists only
export const usePlaylists = () => useMusicStore((state) => ({
  playlists: state.playlists,
  isLoadingPlaylists: state.isLoadingPlaylists,
  fetchPlaylists: state.fetchPlaylists,
}));

// Current playlist detail
export const useCurrentPlaylistDetail = () => useMusicStore((state) => ({
  currentPlaylistDetail: state.currentPlaylistDetail,
  fetchPlaylistDetail: state.fetchPlaylistDetail,
}));

// Equalizer state
export const useEqualizerState = () => useMusicStore((state) => ({
  equalizer: state.equalizer,
  audioFilters: state.audioFilters,
}));

// Search functionality
export const useSearchState = () => useMusicStore((state) => ({
  searchResults: state.searchResults,
  isSearching: state.isSearching,
  searchYouTube: state.searchYouTube,
  addSong: state.addSong,
  clearSearch: state.clearSearch,
}));

// =============================================================================
// Utility Functions
// =============================================================================

export function formatDuration(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return '0:00';
  }
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

export function formatDurationLong(seconds: number): string {
  if (!Number.isFinite(seconds) || seconds < 0) {
    return '0 min';
  }
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  
  if (hours > 0) {
    return `${hours}h ${mins}m`;
  }
  return `${mins} min`;
}

// Equalizer frequency labels (8 bands)
export const EQUALIZER_FREQUENCIES = [
  '60', '170', '310', '600', '1K', '3K', '6K', '12K'
];

// Equalizer frequency values in Hz (8 bands)
export const EQUALIZER_FREQ_VALUES = [60, 170, 310, 600, 1000, 3000, 6000, 12000];

export const EQUALIZER_PRESETS = [
  { name: 'flat', label: 'Flat' },
  { name: 'bass_boost', label: 'Bass Boost' },
  { name: 'treble_boost', label: 'Treble Boost' },
  { name: 'vocal', label: 'Vocal' },
  { name: 'rock', label: 'Rock' },
  { name: 'electronic', label: 'Electronic' },
  { name: 'acoustic', label: 'Acoustic' },
];
