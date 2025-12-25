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
    onError: (error: string) => void,
    onToolExecuted?: (tools: string[]) => void
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
                  } else if (parsed.type === 'tool_executed') {
                    if (onToolExecuted) {
                      onToolExecuted(parsed.tools);
                    }
                  } else if (parsed.type === 'thinking') {
                    // Tool is being executed - UI can show a "thinking" indicator
                    console.log('🔧 Tool executing:', parsed.message);
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

// =============================================================================
// Pomodoro API Types
// =============================================================================

export type PomodoroSessionStatus = 'running' | 'paused' | 'completed' | 'cancelled';
export type PomodoroSessionType = 'work' | 'short_break' | 'long_break';

export interface PomodoroSessionDto {
  id: string;
  session_type: PomodoroSessionType;
  status: PomodoroSessionStatus;
  duration_minutes: number;
  elapsed_seconds: number;
  started_at: string;
  paused_at?: string;
  completed_at?: string;
  task_description?: string;
  created_at: string;
  updated_at: string;
}

export interface PomodoroSettingsDto {
  work_duration_minutes: number;
  short_break_minutes: number;
  long_break_minutes: number;
  sessions_until_long_break: number;
  auto_start_breaks: boolean;
  auto_start_work: boolean;
}

export interface PomodoroStatsDto {
  date: string;
  total_sessions: number;
  completed_sessions: number;
  total_work_minutes: number;
  total_break_minutes: number;
}

export interface StartSessionRequest {
  session_type: PomodoroSessionType;
  task_description?: string;
  duration_minutes?: number;
}

export interface UpdateSettingsRequest {
  work_duration_minutes?: number;
  short_break_minutes?: number;
  long_break_minutes?: number;
  sessions_until_long_break?: number;
  auto_start_breaks?: boolean;
  auto_start_work?: boolean;
}

export const pomodoroApi = {
  // Get active session (if any)
  getActiveSession: async (): Promise<PomodoroSessionDto | null> => {
    try {
      const response = await api.get<PomodoroSessionDto>('/pomodoro/sessions/active');
      return response.data;
    } catch (error) {
      // 404 means no active session
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  // Start a new session
  startSession: async (request: StartSessionRequest): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/start', request);
    return response.data;
  },

  // Pause current session
  pauseSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/pause');
    return response.data;
  },

  // Resume paused session
  resumeSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/resume');
    return response.data;
  },

  // Complete session (mark as done)
  completeSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/complete');
    return response.data;
  },

  // Cancel session
  cancelSession: async (): Promise<void> => {
    await api.post('/pomodoro/sessions/cancel');
  },

  // Get session history
  getHistory: async (limit = 20): Promise<PomodoroSessionDto[]> => {
    const response = await api.get<PomodoroSessionDto[]>('/pomodoro/sessions/history', {
      params: { limit }
    });
    return response.data;
  },

  // Get settings
  getSettings: async (): Promise<PomodoroSettingsDto> => {
    const response = await api.get<PomodoroSettingsDto>('/pomodoro/settings');
    return response.data;
  },

  // Update settings
  updateSettings: async (request: UpdateSettingsRequest): Promise<PomodoroSettingsDto> => {
    const response = await api.put<PomodoroSettingsDto>('/pomodoro/settings', request);
    return response.data;
  },

  // Get daily stats
  getDailyStats: async (date?: string): Promise<PomodoroStatsDto> => {
    const response = await api.get<PomodoroStatsDto>('/pomodoro/stats/daily', {
      params: date ? { date } : undefined
    });
    return response.data;
  },

  // Get weekly stats
  getWeeklyStats: async (): Promise<PomodoroStatsDto[]> => {
    const response = await api.get<PomodoroStatsDto[]>('/pomodoro/stats/weekly');
    return response.data;
  },
};

// =============================================================================
// Kanban API Types
// =============================================================================

export interface KanbanCardDto {
  id: string;
  column_id: string;
  title: string;
  description?: string;
  color?: string;
  labels: string[];
  due_date?: string;
  order: number;
  created_at: string;
  updated_at: string;
}

export interface KanbanColumnDto {
  id: string;
  board_id: string;
  name: string;
  color?: string;
  wip_limit?: number;
  order: number;
  cards: KanbanCardDto[];
  created_at: string;
  updated_at: string;
}

export interface KanbanBoardDto {
  id: string;
  name: string;
  description?: string;
  color?: string;
  is_archived: boolean;
  columns: KanbanColumnDto[];
  created_at: string;
  updated_at: string;
}

export interface KanbanBoardSummaryDto {
  id: string;
  name: string;
  description?: string;
  color?: string;
  is_archived: boolean;
  column_count: number;
  card_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateBoardRequest {
  name: string;
  description?: string;
  color?: string;
  with_default_columns?: boolean;
}

export interface UpdateBoardRequest {
  name?: string;
  description?: string;
  color?: string;
  is_archived?: boolean;
}

export interface CreateColumnRequest {
  name: string;
  color?: string;
  wip_limit?: number;
}

export interface UpdateColumnRequest {
  name?: string;
  color?: string;
  wip_limit?: number;
}

export interface CreateCardRequest {
  title: string;
  description?: string;
  color?: string;
  labels?: string[];
  due_date?: string;
}

export interface UpdateCardRequest {
  title?: string;
  description?: string;
  color?: string;
  labels?: string[];
  due_date?: string;
}

export interface MoveCardRequest {
  target_column_id: string;
  target_order: number;
}

export const kanbanApi = {
  // List all boards (summaries)
  listBoards: async (): Promise<KanbanBoardSummaryDto[]> => {
    const response = await api.get<KanbanBoardSummaryDto[]>('/kanban/boards');
    return response.data;
  },

  // Get a single board with all columns and cards
  getBoard: async (boardId: string): Promise<KanbanBoardDto> => {
    const response = await api.get<KanbanBoardDto>(`/kanban/boards/${boardId}`);
    return response.data;
  },

  // Create a new board
  createBoard: async (request: CreateBoardRequest): Promise<KanbanBoardDto> => {
    const response = await api.post<KanbanBoardDto>('/kanban/boards', request);
    return response.data;
  },

  // Update a board
  updateBoard: async (boardId: string, request: UpdateBoardRequest): Promise<KanbanBoardDto> => {
    const response = await api.patch<KanbanBoardDto>(`/kanban/boards/${boardId}`, request);
    return response.data;
  },

  // Delete a board
  deleteBoard: async (boardId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}`);
  },

  // Create a column
  createColumn: async (boardId: string, request: CreateColumnRequest): Promise<KanbanColumnDto> => {
    const response = await api.post<KanbanColumnDto>(`/kanban/boards/${boardId}/columns`, request);
    return response.data;
  },

  // Update a column
  updateColumn: async (boardId: string, columnId: string, request: UpdateColumnRequest): Promise<KanbanColumnDto> => {
    const response = await api.patch<KanbanColumnDto>(`/kanban/boards/${boardId}/columns/${columnId}`, request);
    return response.data;
  },

  // Delete a column
  deleteColumn: async (boardId: string, columnId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}/columns/${columnId}`);
  },

  // Create a card
  createCard: async (boardId: string, columnId: string, request: CreateCardRequest): Promise<KanbanCardDto> => {
    const response = await api.post<KanbanCardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards`, request);
    return response.data;
  },

  // Update a card
  updateCard: async (boardId: string, columnId: string, cardId: string, request: UpdateCardRequest): Promise<KanbanCardDto> => {
    const response = await api.patch<KanbanCardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}`, request);
    return response.data;
  },

  // Delete a card
  deleteCard: async (boardId: string, columnId: string, cardId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}`);
  },

  // Move a card to another column or position
  moveCard: async (boardId: string, columnId: string, cardId: string, request: MoveCardRequest): Promise<KanbanBoardDto> => {
    const response = await api.put<KanbanBoardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}/move`, request);
    return response.data;
  },
};

// =============================================================================
// Calendar API Types
// =============================================================================

export type EventType = 'event' | 'task' | 'reminder' | 'birthday' | 'holiday';

export interface ReminderDto {
  id: string;
  event_id: string;
  remind_at: string;
  message?: string;
  is_sent: boolean;
  created_at: string;
}

export interface CalendarEventDto {
  id: string;
  title: string;
  description?: string;
  start_time: string;
  end_time?: string;
  all_day: boolean;
  location?: string;
  color?: string;
  event_type: EventType;
  recurrence_rule?: string;
  reminders: ReminderDto[];
  created_at: string;
  updated_at: string;
}

export interface CreateEventRequest {
  title: string;
  description?: string;
  start_time: string;
  end_time?: string;
  all_day?: boolean;
  location?: string;
  color?: string;
  event_type?: EventType;
  recurrence_rule?: string;
}

export interface UpdateEventRequest {
  title?: string;
  description?: string;
  start_time?: string;
  end_time?: string;
  all_day?: boolean;
  location?: string;
  color?: string;
  event_type?: EventType;
  recurrence_rule?: string;
}

export interface CreateReminderRequest {
  remind_at: string;
  message?: string;
}

export const calendarApi = {
  // List all events
  listEvents: async (from?: string, to?: string): Promise<CalendarEventDto[]> => {
    const response = await api.get<{ events: CalendarEventDto[]; total: number }>('/calendar/events', {
      params: { from, to }
    });
    return response.data.events || [];
  },

  // Get today's events
  getTodayEvents: async (): Promise<CalendarEventDto[]> => {
    const response = await api.get<{ events: CalendarEventDto[]; total: number } | CalendarEventDto[]>('/calendar/events/today');
    return Array.isArray(response.data) ? response.data : (response.data.events || []);
  },

  // Get a single event
  getEvent: async (id: string): Promise<CalendarEventDto> => {
    const response = await api.get<CalendarEventDto>(`/calendar/events/${id}`);
    return response.data;
  },

  // Create a new event
  createEvent: async (request: CreateEventRequest): Promise<CalendarEventDto> => {
    const response = await api.post<CalendarEventDto>('/calendar/events', request);
    return response.data;
  },

  // Update an event
  updateEvent: async (id: string, request: UpdateEventRequest): Promise<CalendarEventDto> => {
    const response = await api.patch<CalendarEventDto>(`/calendar/events/${id}`, request);
    return response.data;
  },

  // Delete an event
  deleteEvent: async (id: string): Promise<void> => {
    await api.delete(`/calendar/events/${id}`);
  },

  // Get upcoming reminders
  getReminders: async (): Promise<ReminderDto[]> => {
    const response = await api.get<{ reminders: ReminderDto[] } | ReminderDto[]>('/calendar/reminders');
    return Array.isArray(response.data) ? response.data : (response.data.reminders || []);
  },

  // Add a reminder to an event
  addReminder: async (eventId: string, request: CreateReminderRequest): Promise<ReminderDto> => {
    const response = await api.post<ReminderDto>(`/calendar/events/${eventId}/reminders`, request);
    return response.data;
  },

  // Delete a reminder
  deleteReminder: async (eventId: string, reminderId: string): Promise<void> => {
    await api.delete(`/calendar/events/${eventId}/reminders/${reminderId}`);
  },
};

// =============================================================================
// Notes API Types
// =============================================================================

export interface NoteFolderDto {
  id: string;
  name: string;
  color?: string;
  parent_id?: string;
  note_count: number;
  created_at: string;
  updated_at: string;
}

export interface NoteDto {
  id: string;
  title: string;
  content: string;
  folder_id?: string;
  tags: string[];
  color?: string;
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateNoteRequest {
  title: string;
  content?: string;
  folder_id?: string;
  tags?: string[];
  color?: string;
}

export interface UpdateNoteRequest {
  title?: string;
  content?: string;
  folder_id?: string;
  tags?: string[];
  color?: string;
  is_pinned?: boolean;
  is_archived?: boolean;
}

export interface CreateFolderRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateFolderRequest {
  name?: string;
  color?: string;
  parent_id?: string;
}

export const notesApi = {
  // List all notes
  listNotes: async (folderId?: string, includeArchived = false): Promise<NoteDto[]> => {
    const response = await api.get<{ notes: NoteDto[]; total: number }>('/notes', {
      params: { folder_id: folderId, include_archived: includeArchived }
    });
    return response.data.notes || [];
  },

  // Search notes
  searchNotes: async (query: string): Promise<NoteDto[]> => {
    const response = await api.get<{ notes: NoteDto[]; total: number }>('/notes/search', {
      params: { q: query }
    });
    return response.data.notes || [];
  },

  // Get a single note
  getNote: async (id: string): Promise<NoteDto> => {
    const response = await api.get<NoteDto>(`/notes/${id}`);
    return response.data;
  },

  // Create a new note
  createNote: async (request: CreateNoteRequest): Promise<NoteDto> => {
    const response = await api.post<NoteDto>('/notes', request);
    return response.data;
  },

  // Update a note
  updateNote: async (id: string, request: UpdateNoteRequest): Promise<NoteDto> => {
    const response = await api.patch<NoteDto>(`/notes/${id}`, request);
    return response.data;
  },

  // Delete a note
  deleteNote: async (id: string): Promise<void> => {
    await api.delete(`/notes/${id}`);
  },

  // List all folders
  listFolders: async (): Promise<NoteFolderDto[]> => {
    const response = await api.get<{ folders: NoteFolderDto[] } | NoteFolderDto[]>('/notes/folders');
    return Array.isArray(response.data) ? response.data : (response.data.folders || []);
  },

  // Create a folder
  createFolder: async (request: CreateFolderRequest): Promise<NoteFolderDto> => {
    const response = await api.post<NoteFolderDto>('/notes/folders', request);
    return response.data;
  },

  // Update a folder
  updateFolder: async (id: string, request: UpdateFolderRequest): Promise<NoteFolderDto> => {
    const response = await api.patch<NoteFolderDto>(`/notes/folders/${id}`, request);
    return response.data;
  },

  // Delete a folder
  deleteFolder: async (id: string): Promise<void> => {
    await api.delete(`/notes/folders/${id}`);
  },
};

// =============================================================================
// Docs API Types
// =============================================================================

export type DocType = 'text' | 'markdown' | 'code' | 'spreadsheet' | 'pdf';

export interface DocFolderDto {
  id: string;
  name: string;
  color?: string;
  parent_id?: string;
  doc_count: number;
  created_at: string;
  updated_at: string;
}

export interface DocumentDto {
  id: string;
  title: string;
  content: string;
  folder_id?: string;
  doc_type: DocType;
  mime_type?: string;
  size_bytes: number;
  is_starred: boolean;
  is_shared: boolean;
  shared_with: string[];
  created_at: string;
  updated_at: string;
}

export interface StorageStatsDto {
  total_documents: number;
  total_size_bytes: number;
  by_type: Record<DocType, number>;
}

export interface CreateDocRequest {
  title: string;
  content?: string;
  folder_id?: string;
  doc_type?: DocType;
  mime_type?: string;
}

export interface UpdateDocRequest {
  title?: string;
  content?: string;
  folder_id?: string;
  doc_type?: DocType;
  is_starred?: boolean;
  is_shared?: boolean;
  shared_with?: string[];
}

export interface CreateDocFolderRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateDocFolderRequest {
  name?: string;
  color?: string;
  parent_id?: string;
}

export const docsApi = {
  // List all documents
  listDocs: async (folderId?: string): Promise<DocumentDto[]> => {
    const response = await api.get<{ documents: DocumentDto[]; total: number }>('/docs', {
      params: { folder_id: folderId }
    });
    return response.data.documents || [];
  },

  // Search documents
  searchDocs: async (query: string): Promise<DocumentDto[]> => {
    const response = await api.get<{ documents: DocumentDto[]; total: number }>('/docs/search', {
      params: { q: query }
    });
    return response.data.documents || [];
  },

  // Get storage stats
  getStats: async (): Promise<StorageStatsDto> => {
    const response = await api.get<StorageStatsDto>('/docs/stats');
    return response.data;
  },

  // Get a single document
  getDoc: async (id: string): Promise<DocumentDto> => {
    const response = await api.get<DocumentDto>(`/docs/${id}`);
    return response.data;
  },

  // Create a new document
  createDoc: async (request: CreateDocRequest): Promise<DocumentDto> => {
    const response = await api.post<DocumentDto>('/docs', request);
    return response.data;
  },

  // Update a document
  updateDoc: async (id: string, request: UpdateDocRequest): Promise<DocumentDto> => {
    const response = await api.patch<DocumentDto>(`/docs/${id}`, request);
    return response.data;
  },

  // Delete a document
  deleteDoc: async (id: string): Promise<void> => {
    await api.delete(`/docs/${id}`);
  },

  // List all folders
  listFolders: async (): Promise<DocFolderDto[]> => {
    const response = await api.get<{ folders: DocFolderDto[] } | DocFolderDto[]>('/docs/folders');
    return Array.isArray(response.data) ? response.data : (response.data.folders || []);
  },

  // Create a folder
  createFolder: async (request: CreateDocFolderRequest): Promise<DocFolderDto> => {
    const response = await api.post<DocFolderDto>('/docs/folders', request);
    return response.data;
  },

  // Update a folder
  updateFolder: async (id: string, request: UpdateDocFolderRequest): Promise<DocFolderDto> => {
    const response = await api.patch<DocFolderDto>(`/docs/folders/${id}`, request);
    return response.data;
  },

  // Delete a folder
  deleteFolder: async (id: string): Promise<void> => {
    await api.delete(`/docs/folders/${id}`);
  },
};

// =============================================================================
// Images API Types
// =============================================================================

export type ImageSource = 'generated' | 'uploaded' | 'external';

export interface AlbumDto {
  id: string;
  name: string;
  description?: string;
  cover_image_id?: string;
  image_count: number;
  created_at: string;
  updated_at: string;
}

export interface ImageDto {
  id: string;
  title: string;
  description?: string;
  url: string;
  thumbnail_url?: string;
  width: number;
  height: number;
  size_bytes: number;
  source: ImageSource;
  prompt?: string;
  negative_prompt?: string;
  model?: string;
  seed?: number;
  steps?: number;
  cfg_scale?: number;
  tags: string[];
  album_id?: string;
  is_favorite: boolean;
  created_at: string;
}

export interface GenerateImageRequest {
  prompt: string;
  negative_prompt?: string;
  width?: number;
  height?: number;
  steps?: number;
  cfg_scale?: number;
  seed?: number;
  style?: string;
}

export interface ImageStyleDto {
  id: string;
  name: string;
  description: string;
  prompt_modifier: string;
}

export interface CreateAlbumRequest {
  name: string;
  description?: string;
}

export interface UpdateAlbumRequest {
  name?: string;
  description?: string;
  cover_image_id?: string;
}

export interface UpdateImageRequest {
  title?: string;
  description?: string;
  tags?: string[];
  album_id?: string;
  is_favorite?: boolean;
}

export const imagesApi = {
  // List all images
  listImages: async (albumId?: string, favoritesOnly = false): Promise<ImageDto[]> => {
    const response = await api.get<{ images: ImageDto[]; total: number; has_more: boolean }>('/images', {
      params: { album_id: albumId, favorites_only: favoritesOnly }
    });
    return response.data.images || [];
  },

  // Get a single image
  getImage: async (id: string): Promise<ImageDto> => {
    const response = await api.get<ImageDto>(`/images/${id}`);
    return response.data;
  },

  // Generate a new image
  generateImage: async (request: GenerateImageRequest): Promise<ImageDto> => {
    const response = await api.post<ImageDto>('/images/generate', request);
    return response.data;
  },

  // Update an image
  updateImage: async (id: string, request: UpdateImageRequest): Promise<ImageDto> => {
    const response = await api.patch<ImageDto>(`/images/${id}`, request);
    return response.data;
  },

  // Delete an image
  deleteImage: async (id: string): Promise<void> => {
    await api.delete(`/images/${id}`);
  },

  // Toggle favorite
  toggleFavorite: async (id: string): Promise<ImageDto> => {
    const response = await api.post<ImageDto>(`/images/${id}/favorite`);
    return response.data;
  },

  // Get available styles
  getStyles: async (): Promise<ImageStyleDto[]> => {
    const response = await api.get<{ styles: ImageStyleDto[] } | ImageStyleDto[]>('/images/styles');
    return Array.isArray(response.data) ? response.data : (response.data.styles || []);
  },

  // List all albums
  listAlbums: async (): Promise<AlbumDto[]> => {
    const response = await api.get<{ albums: AlbumDto[] } | AlbumDto[]>('/images/albums');
    return Array.isArray(response.data) ? response.data : (response.data.albums || []);
  },

  // Get a single album
  getAlbum: async (id: string): Promise<AlbumDto> => {
    const response = await api.get<AlbumDto>(`/images/albums/${id}`);
    return response.data;
  },

  // Create an album
  createAlbum: async (request: CreateAlbumRequest): Promise<AlbumDto> => {
    const response = await api.post<AlbumDto>('/images/albums', request);
    return response.data;
  },

  // Update an album
  updateAlbum: async (id: string, request: UpdateAlbumRequest): Promise<AlbumDto> => {
    const response = await api.patch<AlbumDto>(`/images/albums/${id}`, request);
    return response.data;
  },

  // Delete an album
  deleteAlbum: async (id: string): Promise<void> => {
    await api.delete(`/images/albums/${id}`);
  },
};

export default api;
