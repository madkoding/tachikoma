/**
 * =============================================================================
 * useMusicEvents Hook
 * =============================================================================
 * Server-Sent Events (SSE) hook for real-time music updates.
 * Connects to /api/music/events and dispatches updates to the music store.
 * Falls back to polling if SSE connection fails.
 * =============================================================================
 */

import { useEffect, useRef, useCallback, useState } from 'react';
import { useMusicStore } from '../stores/musicStore';

// Event types from backend
export type MusicEventType =
  | 'PlaylistCreated'
  | 'PlaylistUpdated'
  | 'PlaylistDeleted'
  | 'SongAdded'
  | 'SongRemoved'
  | 'SongUpdated'
  | 'SongLiked'
  | 'DownloadStarted'
  | 'DownloadProgress'
  | 'DownloadComplete'
  | 'DownloadFailed'
  | 'Heartbeat';

export interface PlaylistEventData {
  id: string;
  name: string;
  description?: string;
  cover_url?: string;
  song_count: number;
  total_duration: number;
}

export interface SongEventData {
  id: string;
  playlist_id: string;
  youtube_id: string;
  title: string;
  artist?: string;
  album?: string;
  duration: number;
  cover_url?: string;
  song_order: number;
}

export interface MusicEvent {
  type: MusicEventType;
  data?: PlaylistEventData | SongEventData | Record<string, unknown>;
}

export interface UseMusicEventsOptions {
  /** Playlist ID to watch for updates (optional - watches all if not provided) */
  playlistId?: string;
  /** Enable debug logging */
  debug?: boolean;
  /** Auto-reconnect on disconnect */
  autoReconnect?: boolean;
  /** Reconnect delay in ms */
  reconnectDelay?: number;
  /** Callback when a song is added */
  onSongAdded?: (playlistId: string, song: SongEventData) => void;
  /** Callback when a playlist is created */
  onPlaylistCreated?: (playlist: PlaylistEventData) => void;
  /** Callback when a playlist is updated */
  onPlaylistUpdated?: (playlist: PlaylistEventData) => void;
  /** Callback when a song is removed */
  onSongRemoved?: (playlistId: string, songId: string) => void;
}

export interface UseMusicEventsReturn {
  /** Whether SSE is currently connected */
  isConnected: boolean;
  /** Whether using polling fallback */
  isPolling: boolean;
  /** Last error message */
  error: string | null;
  /** Manually reconnect */
  reconnect: () => void;
  /** Disconnect and stop listening */
  disconnect: () => void;
}

export function useMusicEvents(options: UseMusicEventsOptions = {}): UseMusicEventsReturn {
  const {
    playlistId,
    debug = false,
    autoReconnect = true,
    reconnectDelay = 3000,
    onSongAdded,
    onPlaylistCreated,
    onPlaylistUpdated,
    onSongRemoved,
  } = options;

  const [isConnected, setIsConnected] = useState(false);
  const [isPolling, setIsPolling] = useState(false);
  const [error, setError] = useState<string | null>(null);
  
  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const pollingIntervalRef = useRef<ReturnType<typeof setInterval> | null>(null);
  const mountedRef = useRef(true);

  const log = useCallback((...args: unknown[]) => {
    if (debug) {
      console.log('🎵 [useMusicEvents]', ...args);
    }
  }, [debug]);

  // Store actions
  const { 
    fetchPlaylistDetail, 
    fetchPlaylists,
    currentPlaylistDetail,
  } = useMusicStore();

  // Handle incoming events
  const handleEvent = useCallback((event: MusicEvent) => {
    log('Received event:', event.type, event.data);

    switch (event.type) {
      case 'PlaylistCreated': {
        const playlist = event.data as PlaylistEventData;
        onPlaylistCreated?.(playlist);
        // Refresh playlists list
        fetchPlaylists();
        break;
      }

      case 'PlaylistUpdated': {
        const playlist = event.data as PlaylistEventData;
        onPlaylistUpdated?.(playlist);
        // If we're viewing this playlist, refresh it
        if (currentPlaylistDetail?.id === playlist.id) {
          fetchPlaylistDetail(playlist.id);
        }
        // Also refresh the list
        fetchPlaylists();
        break;
      }

      case 'PlaylistDeleted': {
        const { id } = event.data as { id: string };
        // Refresh playlists list
        fetchPlaylists();
        log('Playlist deleted:', id);
        break;
      }

      case 'SongAdded': {
        const { playlist_id, song } = event.data as { playlist_id: string; song: SongEventData };
        onSongAdded?.(playlist_id, song);
        
        // If we're watching this playlist or watching all, refresh
        if (!playlistId || playlistId === playlist_id) {
          fetchPlaylistDetail(playlist_id);
        }
        // Also refresh playlists (song count changed)
        fetchPlaylists();
        break;
      }

      case 'SongRemoved': {
        const { playlist_id, song_id } = event.data as { playlist_id: string; song_id: string };
        onSongRemoved?.(playlist_id, song_id);
        
        if (!playlistId || playlistId === playlist_id) {
          fetchPlaylistDetail(playlist_id);
        }
        fetchPlaylists();
        break;
      }

      case 'SongUpdated': {
        const song = event.data as SongEventData;
        if (!playlistId || playlistId === song.playlist_id) {
          fetchPlaylistDetail(song.playlist_id);
        }
        break;
      }

      case 'SongLiked': {
        const { song_id } = event.data as { song_id: string; is_liked: boolean };
        log('Song liked status changed:', song_id);
        // Optionally refresh current playlist
        if (currentPlaylistDetail) {
          fetchPlaylistDetail(currentPlaylistDetail.id);
        }
        break;
      }

      case 'Heartbeat':
        log('Heartbeat received');
        break;

      default:
        log('Unknown event type:', event.type);
    }
  }, [
    log, 
    playlistId, 
    currentPlaylistDetail, 
    fetchPlaylistDetail, 
    fetchPlaylists,
    onSongAdded, 
    onPlaylistCreated, 
    onPlaylistUpdated, 
    onSongRemoved
  ]);

  // Start polling fallback
  const startPolling = useCallback(() => {
    if (pollingIntervalRef.current) return;
    
    log('Starting polling fallback');
    setIsPolling(true);

    pollingIntervalRef.current = setInterval(() => {
      if (playlistId) {
        fetchPlaylistDetail(playlistId);
      }
      fetchPlaylists();
    }, 2000);
  }, [playlistId, fetchPlaylistDetail, fetchPlaylists, log]);

  // Stop polling
  const stopPolling = useCallback(() => {
    if (pollingIntervalRef.current) {
      clearInterval(pollingIntervalRef.current);
      pollingIntervalRef.current = null;
    }
    setIsPolling(false);
  }, []);

  // Connect to SSE
  const connect = useCallback(() => {
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
    }

    log('Connecting to SSE...');
    setError(null);

    try {
      const eventSource = new EventSource('/api/music/events');
      eventSourceRef.current = eventSource;

      eventSource.onopen = () => {
        if (!mountedRef.current) return;
        log('SSE connected');
        setIsConnected(true);
        setError(null);
        stopPolling(); // Stop polling when SSE works
      };

      eventSource.onmessage = (event) => {
        if (!mountedRef.current) return;
        try {
          const data = JSON.parse(event.data);
          handleEvent(data);
        } catch (e) {
          log('Failed to parse SSE message:', e);
        }
      };

      eventSource.onerror = (e) => {
        if (!mountedRef.current) return;
        log('SSE error:', e);
        setIsConnected(false);
        setError('Connection lost');
        
        eventSource.close();
        eventSourceRef.current = null;

        // Start polling as fallback
        startPolling();

        // Schedule reconnect
        if (autoReconnect) {
          reconnectTimeoutRef.current = setTimeout(() => {
            if (mountedRef.current) {
              connect();
            }
          }, reconnectDelay);
        }
      };
    } catch (e) {
      log('Failed to create EventSource:', e);
      setError('Failed to connect');
      startPolling();
    }
  }, [log, handleEvent, autoReconnect, reconnectDelay, startPolling, stopPolling]);

  // Disconnect
  const disconnect = useCallback(() => {
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    if (eventSourceRef.current) {
      eventSourceRef.current.close();
      eventSourceRef.current = null;
    }

    stopPolling();
    setIsConnected(false);
    log('Disconnected');
  }, [stopPolling, log]);

  // Reconnect
  const reconnect = useCallback(() => {
    disconnect();
    connect();
  }, [disconnect, connect]);

  // Connect on mount
  useEffect(() => {
    mountedRef.current = true;
    connect();

    return () => {
      mountedRef.current = false;
      disconnect();
    };
  }, [connect, disconnect]);

  return {
    isConnected,
    isPolling,
    error,
    reconnect,
    disconnect,
  };
}
