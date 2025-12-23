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

export default api;
