import axios from 'axios';

// Use environment variable for API URL, fallback to relative path for local development
const API_BASE_URL = import.meta.env.VITE_API_URL || '/api';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 60000, // 60 seconds for LLM responses
  headers: {
    'Content-Type': 'application/json',
  },
});

// Request interceptor
api.interceptors.request.use(
  (config) => {
    // Add any auth headers here if needed
    return config;
  },
  (error) => {
    return Promise.reject(error);
  }
);

// Response interceptor
api.interceptors.response.use(
  (response) => response,
  (error) => {
    // Handle errors globally
    if (error.response) {
      // Server responded with error
      console.error('API Error:', error.response.data);
    } else if (error.request) {
      // Request made but no response
      console.error('Network Error:', error.message);
    }
    return Promise.reject(error);
  }
);

export interface ChatMessageRequest {
  message: string;
  conversation_id?: string;
  stream?: boolean;
}

export interface ChatMessageResponse {
  content: string;
  conversation_id: string;
  message_id: string;
  model: string;
  tokens_prompt: number;
  tokens_completion: number;
  processing_time_ms: number;
}

export interface ConversationDto {
  id: string;
  title: string;
  message_count: number;
  created_at: string;
  updated_at: string;
}

export interface ConversationWithMessagesDto {
  id: string;
  title: string;
  messages: ChatMessageDto[];
  created_at: string;
  updated_at: string;
}

export interface ChatMessageDto {
  id: string;
  role: string;
  content: string;
  model?: string;
  tokens_prompt?: number;
  tokens_completion?: number;
  created_at: string;
}

export interface StreamCompleteResponse {
  conversation_id: string;
  message_id: string;
  model: string;
  tokens_prompt: number;
  tokens_completion: number;
  processing_time_ms: number;
}

// Helper to calculate tokens per second
export function calculateTokensPerSecond(tokens: number, timeMs: number): number {
  if (timeMs <= 0) return 0;
  return Math.round((tokens / timeMs) * 1000 * 10) / 10;
}

export interface HealthResponse {
  status: string;
  services: {
    database: string;
    llm: string;
    search: string;
  };
  version: string;
  uptime_seconds: number;
}

// Chat API
export const chatApi = {
  sendMessage: async (request: ChatMessageRequest): Promise<ChatMessageResponse> => {
    const response = await api.post<ChatMessageResponse>('/chat', request);
    return response.data;
  },

  // Streaming message using Server-Sent Events
  sendMessageStream: (
    request: ChatMessageRequest,
    onChunk: (chunk: string) => void,
    onComplete: (response: StreamCompleteResponse) => void,
    onError: (error: string) => void
  ): (() => void) => {
    const controller = new AbortController();
    
    fetch('/api/chat/stream', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify(request),
      signal: controller.signal,
    })
      .then(async (response) => {
        if (!response.ok) {
          throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const reader = response.body?.getReader();
        if (!reader) {
          throw new Error('No reader available');
        }
        
        const decoder = new TextDecoder();
        let buffer = '';
        
        while (true) {
          const { done, value } = await reader.read();
          if (done) break;
          
          buffer += decoder.decode(value, { stream: true });
          
          // Parse SSE events
          const lines = buffer.split('\n');
          buffer = lines.pop() || '';
          
          for (const line of lines) {
            if (line.startsWith('data:')) {
              const data = line.slice(5).trim();
              if (data) {
                try {
                  const parsed = JSON.parse(data);
                  
                  if (parsed.type === 'chunk') {
                    onChunk(parsed.content);
                  } else if (parsed.type === 'done') {
                    onComplete({
                      conversation_id: parsed.conversation_id,
                      message_id: parsed.message_id,
                      model: parsed.model,
                      tokens_prompt: parsed.tokens_prompt,
                      tokens_completion: parsed.tokens_completion,
                      processing_time_ms: parsed.processing_time_ms,
                    });
                  } else if (parsed.type === 'error') {
                    onError(parsed.error);
                  }
                } catch {
                  // Ignore parse errors for incomplete data
                }
              }
            }
          }
        }
      })
      .catch((error) => {
        if (error.name !== 'AbortError') {
          onError(error.message);
        }
      });
    
    // Return cancel function
    return () => controller.abort();
  },

  getConversations: async (): Promise<ConversationDto[]> => {
    const response = await api.get<ConversationDto[]>('/chat/conversations');
    return response.data;
  },

  getConversation: async (id: string): Promise<ConversationWithMessagesDto> => {
    const response = await api.get<ConversationWithMessagesDto>(`/chat/conversations/${id}`);
    return response.data;
  },

  deleteConversation: async (id: string): Promise<void> => {
    await api.delete(`/chat/conversations/${id}`);
  },
};

// Health API
export const healthApi = {
  check: async (): Promise<HealthResponse> => {
    const response = await api.get<HealthResponse>('/health');
    return response.data;
  },
};

// =============================================================================
// Checklists API
// =============================================================================

export interface ChecklistItemDto {
  id: string;
  content: string;
  is_completed: boolean;
  completed_at?: string;
  order: number;
  created_at: string;
  updated_at?: string;
}

export interface ChecklistDto {
  id: string;
  title: string;
  description?: string;
  priority: number;
  due_date?: string;
  notification_interval?: number;
  is_archived: boolean;
  total_items: number;
  completed_items: number;
  created_at: string;
  updated_at: string;
}

export interface ChecklistWithItemsDto extends ChecklistDto {
  items: ChecklistItemDto[];
}

export interface PaginatedResponse<T> {
  data: T[];
  total: number;
  page: number;
  per_page: number;
  total_pages: number;
}

export interface CreateChecklistRequest {
  title: string;
  description?: string;
  priority?: number;
  due_date?: string;
  notification_interval?: number;
  items?: CreateChecklistItemRequest[];
}

export interface CreateChecklistItemRequest {
  content: string;
  is_completed?: boolean;
  order?: number;
}

export interface UpdateChecklistRequest {
  title?: string;
  description?: string;
  priority?: number;
  due_date?: string;
  notification_interval?: number;
  is_archived?: boolean;
}

export interface UpdateChecklistItemRequest {
  content?: string;
  is_completed?: boolean;
  order?: number;
}

export interface ImportMarkdownRequest {
  markdown: string;
  title?: string;
}

export const checklistsApi = {
  // List all checklists with pagination
  list: async (page = 1, perPage = 50, includeArchived = false): Promise<PaginatedResponse<ChecklistDto>> => {
    const response = await api.get<PaginatedResponse<ChecklistDto>>('/checklists', {
      params: { page, per_page: perPage, include_archived: includeArchived }
    });
    return response.data;
  },

  // Get a single checklist with its items
  get: async (id: string): Promise<ChecklistWithItemsDto> => {
    const response = await api.get<ChecklistWithItemsDto>(`/checklists/${id}`);
    return response.data;
  },

  // Create a new checklist
  create: async (request: CreateChecklistRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.post<ChecklistWithItemsDto>('/checklists', request);
    return response.data;
  },

  // Update a checklist
  update: async (id: string, request: UpdateChecklistRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.patch<ChecklistWithItemsDto>(`/checklists/${id}`, request);
    return response.data;
  },

  // Delete a checklist
  delete: async (id: string): Promise<void> => {
    await api.delete(`/checklists/${id}`);
  },

  // Add an item to a checklist
  addItem: async (checklistId: string, request: CreateChecklistItemRequest): Promise<ChecklistItemDto> => {
    const response = await api.post<ChecklistItemDto>(`/checklists/${checklistId}/items`, request);
    return response.data;
  },

  // Update an item
  updateItem: async (checklistId: string, itemId: string, request: UpdateChecklistItemRequest): Promise<ChecklistItemDto> => {
    const response = await api.patch<ChecklistItemDto>(`/checklists/${checklistId}/items/${itemId}`, request);
    return response.data;
  },

  // Delete an item
  deleteItem: async (checklistId: string, itemId: string): Promise<void> => {
    await api.delete(`/checklists/${checklistId}/items/${itemId}`);
  },

  // Toggle item completion
  toggleItem: async (checklistId: string, itemId: string): Promise<ChecklistItemDto> => {
    const response = await api.post<ChecklistItemDto>(`/checklists/${checklistId}/items/${itemId}/toggle`);
    return response.data;
  },

  // Import from markdown
  importMarkdown: async (request: ImportMarkdownRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.post<ChecklistWithItemsDto>('/checklists/import/markdown', request);
    return response.data;
  },
};

// =============================================================================
// Music API Types
// =============================================================================

export interface PlaylistDto {
  id: string;
  name: string;
  description?: string;
  cover_url?: string;
  is_suggestions: boolean;
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
  last_played?: string;
  created_at: string;
}

export interface PlaylistWithSongsDto {
  id: string;
  name: string;
  description?: string;
  cover_url?: string;
  is_suggestions: boolean;
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

// =============================================================================
// Music API
// =============================================================================

export const musicApi = {
  // Playlists
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

  // Songs
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

  // Streaming
  getStreamUrl: (songId: string): string => {
    return `${API_BASE_URL}/music/stream/${songId}`;
  },

  getStreamInfo: async (songId: string): Promise<StreamInfoDto> => {
    const response = await api.get<StreamInfoDto>(`/music/stream/${songId}/info`);
    return response.data;
  },

  // YouTube
  searchYouTube: async (query: string, limit = 10): Promise<YouTubeSearchResultDto[]> => {
    const response = await api.get<YouTubeSearchResultDto[]>('/music/youtube/search', {
      params: { q: query, limit }
    });
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

  // Cover Art
  searchCover: async (title: string, artist?: string): Promise<CoverArtResultDto | null> => {
    const response = await api.get<CoverArtResultDto | null>('/music/covers/search', {
      params: { title, artist }
    });
    return response.data;
  },

  // Equalizer
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

  // History
  getListeningHistory: async (limit = 50): Promise<SongDto[]> => {
    const response = await api.get<SongDto[]>('/music/history', { params: { limit } });
    return response.data;
  },

  getMostPlayed: async (limit = 20): Promise<SongDto[]> => {
    const response = await api.get<SongDto[]>('/music/stats/most-played', { params: { limit } });
    return response.data;
  },
};

export default api;
