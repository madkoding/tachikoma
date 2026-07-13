import api from './client';

export interface PlaylistDto {
  id: string;
  name: string;
  description?: string;
  cover_url?: string;
  is_suggestions: boolean;
  is_favorites: boolean;
  last_suggestions_update?: string;
  shuffle: boolean;
  repeat_mode: 'off' | 'one' | 'all';
  song_count: number;
  total_duration: number;
  created_at: string;
  updated_at: string;
}

export interface SongDto {
  id: string;
  playlist_id: string;
  youtube_id: string;
  youtube_url: string;
  title: string;
  artist?: string;
  album?: string;
  duration: number;
  cover_url?: string;
  thumbnail_url?: string;
  song_order: number;
  play_count: number;
  is_liked: boolean;
  last_played?: string;
  created_at: string;
}

export interface PlaylistWithSongsDto {
  id: string;
  name: string;
  description?: string;
  cover_url?: string;
  is_suggestions: boolean;
  is_favorites: boolean;
  last_suggestions_update?: string;
  shuffle: boolean;
  repeat_mode: 'off' | 'one' | 'all';
  song_count: number;
  total_duration: number;
  created_at: string;
  updated_at: string;
  songs: SongDto[];
}

export interface CreatePlaylistRequest {
  name: string;
  description?: string;
  cover_url?: string;
}

export interface UpdatePlaylistRequest {
  name?: string;
  description?: string;
  cover_url?: string;
  shuffle?: boolean;
  repeat_mode?: 'off' | 'one' | 'all';
}

export interface CreateSongRequest {
  youtube_url: string;
  title?: string;
  artist?: string;
  album?: string;
  cover_url?: string;
}

export interface UpdateSongRequest {
  title?: string;
  artist?: string;
  album?: string;
  cover_url?: string;
  song_order?: number;
  is_liked?: boolean;
}

export interface EqualizerSettingsDto {
  enabled: boolean;
  preset?: string;
  bands: number[];
}

export interface YouTubeSearchResultDto {
  video_id: string;
  title: string;
  channel: string;
  duration: number;
  thumbnail: string;
  view_count?: number;
}

export type MetadataSource = 'music_brainz' | 'llm_inference' | 'original';

export interface EnrichedSearchResultDto {
  video_id: string;
  original_title: string;
  title: string;
  artist?: string;
  album?: string;
  channel?: string;
  duration: number;
  thumbnail: string;
  view_count?: number;
  source: MetadataSource;
}

export interface EnrichMetadataRequest {
  title: string;
  channel?: string;
}

export interface EnrichMetadataResponse {
  title: string;
  artist?: string;
  album?: string;
  source: MetadataSource;
}

export interface YouTubeMetadataDto {
  id: string;
  title: string;
  uploader?: string;
  duration: number;
  thumbnail?: string;
  description?: string;
}

export interface CoverArtResultDto {
  url: string;
  source: string;
  width?: number;
  height?: number;
}

export interface StreamInfoDto {
  song_id: string;
  stream_url: string;
  format: string;
  bitrate: number;
  sample_rate: number;
}

export const musicApi = {
  listPlaylists: async (includeSongs = false): Promise<PlaylistDto[]> => {
    const response = await api.get<PlaylistDto[]>('/music/playlists', {
      params: { include_songs: includeSongs }
    });
    return response.data;
  },

  getPlaylist: async (id: string): Promise<PlaylistWithSongsDto> => {
    const response = await api.get<PlaylistWithSongsDto>(`/music/playlists/${id}`);
    return response.data;
  },

  createPlaylist: async (request: CreatePlaylistRequest): Promise<PlaylistDto> => {
    const response = await api.post<PlaylistDto>('/music/playlists', request);
    return response.data;
  },

  updatePlaylist: async (id: string, request: UpdatePlaylistRequest): Promise<PlaylistDto> => {
    const response = await api.patch<PlaylistDto>(`/music/playlists/${id}`, request);
    return response.data;
  },

  deletePlaylist: async (id: string): Promise<void> => {
    await api.delete(`/music/playlists/${id}`);
  },

  addSong: async (playlistId: string, request: CreateSongRequest): Promise<SongDto> => {
    const response = await api.post<SongDto>(`/music/playlists/${playlistId}/songs`, request);
    return response.data;
  },

  updateSong: async (playlistId: string, songId: string, request: UpdateSongRequest): Promise<SongDto> => {
    const response = await api.patch<SongDto>(`/music/playlists/${playlistId}/songs/${songId}`, request);
    return response.data;
  },

  deleteSong: async (playlistId: string, songId: string): Promise<void> => {
    await api.delete(`/music/playlists/${playlistId}/songs/${songId}`);
  },

  reorderSongs: async (playlistId: string, songIds: string[]): Promise<void> => {
    await api.post(`/music/playlists/${playlistId}/reorder`, { song_ids: songIds });
  },

  getStreamUrl: (songId: string): string => {
    const API_BASE_URL = import.meta.env.VITE_API_URL || '/api';
    return `${API_BASE_URL}/music/stream/${songId}`;
  },

  downloadSong: async (songId: string): Promise<Blob> => {
    const response = await api.get(`/music/download/${songId}`, {
      responseType: 'blob',
      headers: {
        'Accept': 'audio/ogg, audio/mpeg, audio/*',
      },
    });
    return response.data;
  },

  getStreamInfo: async (songId: string): Promise<StreamInfoDto> => {
    const response = await api.get<StreamInfoDto>(`/music/stream/${songId}/info`);
    return response.data;
  },

  searchYouTube: async (query: string, limit = 10): Promise<YouTubeSearchResultDto[]> => {
    const response = await api.get<YouTubeSearchResultDto[]>('/music/youtube/search', {
      params: { q: query, limit }
    });
    return response.data;
  },

  searchYouTubeEnriched: async (query: string, limit = 10): Promise<EnrichedSearchResultDto[]> => {
    const response = await api.get<EnrichedSearchResultDto[]>('/music/youtube/search/enriched', {
      params: { q: query, limit }
    });
    return response.data;
  },

  enrichMetadata: async (request: EnrichMetadataRequest): Promise<EnrichMetadataResponse> => {
    const response = await api.post<EnrichMetadataResponse>('/music/youtube/enrich', request);
    return response.data;
  },

  addMultipleSongs: async (playlistId: string, requests: CreateSongRequest[]): Promise<SongDto[]> => {
    const results: SongDto[] = [];
    for (const request of requests) {
      const song = await musicApi.addSong(playlistId, request);
      results.push(song);
    }
    return results;
  },

  getYouTubeMetadata: async (url: string): Promise<YouTubeMetadataDto> => {
    const response = await api.get<YouTubeMetadataDto>('/music/youtube/metadata', {
      params: { url }
    });
    return response.data;
  },

  searchCover: async (title: string, artist?: string): Promise<CoverArtResultDto | null> => {
    const response = await api.get<CoverArtResultDto | null>('/music/covers/search', {
      params: { title, artist }
    });
    return response.data;
  },

  getEqualizer: async (): Promise<EqualizerSettingsDto> => {
    const response = await api.get<EqualizerSettingsDto>('/music/equalizer');
    return response.data;
  },

  updateEqualizer: async (settings: EqualizerSettingsDto): Promise<EqualizerSettingsDto> => {
    const response = await api.put<EqualizerSettingsDto>('/music/equalizer', settings);
    return response.data;
  },

  getEqualizerPreset: async (name: string): Promise<EqualizerSettingsDto> => {
    const response = await api.get<EqualizerSettingsDto>('/music/equalizer/preset', {
      params: { name }
    });
    return response.data;
  },

  getListeningHistory: async (limit = 50): Promise<SongDto[]> => {
    const response = await api.get<SongDto[]>('/music/history', { params: { limit } });
    return response.data;
  },

  getMostPlayed: async (limit = 20): Promise<SongDto[]> => {
    const response = await api.get<SongDto[]>('/music/stats/most-played', { params: { limit } });
    return response.data;
  },

  initSpecialPlaylists: async (): Promise<PlaylistDto[]> => {
    const response = await api.post<PlaylistDto[]>('/music/init-special-playlists');
    return response.data;
  },

  toggleSongLike: async (songId: string): Promise<SongDto> => {
    const response = await api.post<SongDto>(`/music/songs/${songId}/toggle-like`);
    return response.data;
  },

  fetchSongCover: async (songId: string): Promise<SongDto> => {
    const response = await api.post<SongDto>(`/music/songs/${songId}/fetch-cover`);
    return response.data;
  },

  getLikedSongs: async (): Promise<SongDto[]> => {
    const response = await api.get<SongDto[]>('/music/songs/liked');
    return response.data;
  },

  refreshSuggestions: async (): Promise<PlaylistWithSongsDto> => {
    const response = await api.post<PlaylistWithSongsDto>('/music/playlists/suggestions/refresh');
    return response.data;
  },
};
