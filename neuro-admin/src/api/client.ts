import axios from 'axios';

const api = axios.create({
  baseURL: '/api',
  timeout: 30000,
});

// Types
export interface Memory {
  id: string;
  content: string;
  memory_type: string;
  importance: number;
  embedding?: number[];
  metadata: Record<string, unknown>;
  created_at: string;
  updated_at: string;
}

export interface GraphEdge {
  source: string;
  target: string;
  relation: string;
  weight: number;
}

export interface GraphData {
  nodes: Memory[];
  edges: GraphEdge[];
}

export interface GraphStats {
  total_nodes: number;
  total_edges: number;
  nodes_by_type: Record<string, number>;
  edges_by_relation: Record<string, number>;
  avg_connections: number;
}

export interface SystemHealth {
  ollama: boolean;
  surrealdb: boolean;
  searxng: boolean;
  memory_usage_mb: number;
  uptime_seconds: number;
}

// Graph API
export const graphApi = {
  getGraph: async (limit?: number): Promise<GraphData> => {
    const params = limit ? { limit } : {};
    const response = await api.get('/admin/graph', { params });
    return response.data;
  },

  getStats: async (): Promise<GraphStats> => {
    const response = await api.get('/admin/graph/stats');
    return response.data;
  },
};

// Memory API
export const memoryApi = {
  getAll: async (limit = 100, offset = 0): Promise<Memory[]> => {
    const response = await api.get('/memories', { params: { limit, offset } });
    return response.data;
  },

  getById: async (id: string): Promise<Memory> => {
    const response = await api.get(`/memories/${id}`);
    return response.data;
  },

  search: async (query: string, limit = 10): Promise<Memory[]> => {
    const response = await api.get('/memories/search', { params: { query, limit } });
    return response.data;
  },

  create: async (memory: Partial<Memory>): Promise<Memory> => {
    const response = await api.post('/memories', memory);
    return response.data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/memories/${id}`);
  },

  getRelated: async (id: string): Promise<Memory[]> => {
    const response = await api.get(`/memories/${id}/related`);
    return response.data;
  },
};

// System API
export const systemApi = {
  getHealth: async (): Promise<SystemHealth> => {
    const response = await api.get('/system/health');
    return response.data;
  },

  getModels: async (): Promise<string[]> => {
    const response = await api.get('/system/models');
    return response.data.models;
  },

  getVram: async (): Promise<{ total_mb: number; used_mb: number; free_mb: number }> => {
    const response = await api.get('/system/vram');
    return response.data;
  },
};

export default api;
