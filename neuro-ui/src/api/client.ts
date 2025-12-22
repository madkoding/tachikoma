import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
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
