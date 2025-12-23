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

export type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error' | 'disabled';

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
 * Get the SSE endpoint URL based on current location
 * When accessing remotely, we need to connect directly to the backend
 * since Vite proxy doesn't handle SSE well
 */
function getSSEEndpoint(): string {
  const hostname = window.location.hostname;
  const isLocalhost = hostname === 'localhost' || hostname === '127.0.0.1';
  
  if (isLocalhost) {
    // Local development - can use proxy
    return '/api/admin/graph/events';
  } else {
    // Remote access - connect directly to backend on port 3000
    // Use same hostname but backend port
    return `http://${hostname}:3000/api/admin/graph/events`;
  }
}

// Maximum retry attempts before giving up
const MAX_RETRIES = 5;
// Initial retry delay in ms
const INITIAL_RETRY_DELAY = 2000;
// Maximum retry delay in ms
const MAX_RETRY_DELAY = 30000;

/**
 * Hook to subscribe to real-time graph events via Server-Sent Events (SSE)
 * 
 * This replaces polling with real-time updates from the backend.
 * The connection automatically reconnects on failure with exponential backoff.
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
  const retryCountRef = useRef(0);
  const hasLoggedDisabledRef = useRef(false);

  const connect = useCallback(() => {
    // Don't connect if disabled
    if (!enabled) {
      setStatus('disabled');
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

    // Check if we've exceeded max retries
    if (retryCountRef.current >= MAX_RETRIES) {
      if (!hasLoggedDisabledRef.current) {
        console.warn('[SSE] Max retries exceeded. Real-time updates disabled. Graph will still work with manual refresh.');
        hasLoggedDisabledRef.current = true;
      }
      setStatus('disabled');
      return;
    }

    setStatus('connecting');
    const endpoint = getSSEEndpoint();
    
    // Only log on first attempt
    if (retryCountRef.current === 0) {
      console.log(`[SSE] Connecting to graph events via ${endpoint}...`);
    }
    
    const eventSource = new EventSource(endpoint);
    eventSourceRef.current = eventSource;

    eventSource.onopen = () => {
      console.log('[SSE] Connected to graph events');
      setStatus('connected');
      retryCountRef.current = 0; // Reset retry count on successful connection
      hasLoggedDisabledRef.current = false;
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

    eventSource.onerror = () => {
      setStatus('error');
      
      // Close the failed connection
      eventSource.close();
      eventSourceRef.current = null;
      
      retryCountRef.current++;
      
      // Check if we should give up
      if (retryCountRef.current >= MAX_RETRIES) {
        if (!hasLoggedDisabledRef.current) {
          console.warn('[SSE] Could not connect to graph events. Real-time updates disabled.');
          hasLoggedDisabledRef.current = true;
        }
        setStatus('disabled');
        onError?.(new Error('SSE connection failed after max retries'));
        return;
      }
      
      // Calculate exponential backoff delay
      const baseDelay = INITIAL_RETRY_DELAY * Math.pow(2, retryCountRef.current);
      const delay = Math.min(baseDelay, MAX_RETRY_DELAY);
      
      // Schedule reconnect with backoff
      reconnectTimeoutRef.current = globalThis.setTimeout(() => {
        connect();
      }, delay);
    };
  }, [enabled, onMemoryCreated, onMemoryUpdated, onMemoryDeleted, onRelationCreated, onError]);

  const reconnect = useCallback(() => {
    // Reset retry counters for manual reconnect
    retryCountRef.current = 0;
    hasLoggedDisabledRef.current = false;
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
