import { useEffect, useRef, useCallback, useState } from 'react';

// Event types from backend
export interface MemoryEventData {
  id: string;
  content: string;
  memory_type: string;
  created_at: string;
}

export interface RelationEventData {
  source: string;
  target: string;
  relation: string;
  weight: number;
}

// Discriminated union for all event types
type MemoryEvent =
  | { type: 'Created'; data: MemoryEventData }
  | { type: 'Updated'; data: MemoryEventData }
  | { type: 'Deleted'; data: { id: string } }
  | { type: 'RelationCreated'; data: RelationEventData }
  | { type: 'Heartbeat' };

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface UseGraphEventsOptions {
  onMemoryCreated?: (data: MemoryEventData) => void;
  onMemoryUpdated?: (data: MemoryEventData) => void;
  onMemoryDeleted?: (id: string) => void;
  onRelationCreated?: (data: RelationEventData) => void;
  onError?: (error: Error) => void;
  enabled?: boolean;
}

interface UseGraphEventsReturn {
  status: ConnectionStatus;
  reconnect: () => void;
}

/**
 * Hook to subscribe to real-time graph events via Server-Sent Events (SSE)
 * 
 * This replaces polling with real-time updates from the backend.
 * The connection automatically reconnects on failure.
 */
export function useGraphEvents({
  onMemoryCreated,
  onMemoryUpdated,
  onMemoryDeleted,
  onRelationCreated,
  onError,
  enabled = true,
}: UseGraphEventsOptions): UseGraphEventsReturn {
  const eventSourceRef = useRef<EventSource | null>(null);
  const reconnectTimeoutRef = useRef<number | null>(null);
  const [status, setStatus] = useState<ConnectionStatus>('connecting');

  const connect = useCallback(() => {
    // Don't connect if disabled
    if (!enabled) {
      setStatus('disconnected');
      return;
    }
    
    // Close existing connection
    if (eventSourceRef.current) {
      eventSourceRef.current.close();
    }
    
    // Clear any pending reconnect
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    setStatus('connecting');
    console.log('[SSE] Connecting to graph events...');
    const eventSource = new EventSource('/api/admin/graph/events');
    eventSourceRef.current = eventSource;

    eventSource.onopen = () => {
      console.log('[SSE] Connected to graph events');
      setStatus('connected');
    };

    eventSource.onmessage = (event) => {
      try {
        // SSE sends "heartbeat" as plain text for keep-alive
        if (event.data === 'heartbeat' || event.data === '') {
          return;
        }
        
        const parsed = JSON.parse(event.data) as MemoryEvent;
        
        switch (parsed.type) {
          case 'Created':
            console.log('[SSE] Memory created:', parsed.data);
            onMemoryCreated?.(parsed.data);
            break;
          case 'Updated':
            console.log('[SSE] Memory updated:', parsed.data);
            onMemoryUpdated?.(parsed.data);
            break;
          case 'Deleted':
            console.log('[SSE] Memory deleted:', parsed.data.id);
            onMemoryDeleted?.(parsed.data.id);
            break;
          case 'RelationCreated':
            console.log('[SSE] Relation created:', parsed.data);
            onRelationCreated?.(parsed.data);
            break;
          case 'Heartbeat':
            // Ignore heartbeats
            break;
        }
      } catch (e) {
        // Not all messages are JSON (e.g., heartbeat)
        if (event.data && event.data !== 'heartbeat') {
          console.warn('[SSE] Failed to parse event:', event.data, e);
        }
      }
    };

    eventSource.onerror = (error) => {
      console.error('[SSE] Connection error:', error);
      setStatus('error');
      onError?.(new Error('SSE connection error'));
      
      // Close and schedule reconnect
      eventSource.close();
      eventSourceRef.current = null;
      
      // Reconnect after 5 seconds
      reconnectTimeoutRef.current = globalThis.setTimeout(() => {
        console.log('[SSE] Attempting to reconnect...');
        connect();
      }, 5000);
    };
  }, [enabled, onMemoryCreated, onMemoryUpdated, onMemoryDeleted, onRelationCreated, onError]);

  const reconnect = useCallback(() => {
    connect();
  }, [connect]);

  // Initial connection
  useEffect(() => {
    connect();
    
    return () => {
      // Cleanup on unmount
      if (eventSourceRef.current) {
        eventSourceRef.current.close();
        eventSourceRef.current = null;
      }
      if (reconnectTimeoutRef.current) {
        clearTimeout(reconnectTimeoutRef.current);
        reconnectTimeoutRef.current = null;
      }
    };
  }, [connect]);

  return {
    status,
    reconnect,
  };
}
