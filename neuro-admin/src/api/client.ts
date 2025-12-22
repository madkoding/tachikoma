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
  importance_score: number;
  access_count: number;
  vector?: number[];
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
  edges_by_type: Record<string, number>;
  avg_connections: number;
}

export interface SystemHealth {
  status: string;
  services: {
    database: string;
    llm: string;
    search: string;
  };
  version: string;
  uptime_seconds: number;
}

// Graph API
export const graphApi = {
  getGraph: async (_limit?: number): Promise<GraphData> => {
    const response = await api.get('/admin/graph/export');
    // Transform export format to GraphData format
    return {
      nodes: response.data.nodes,
      edges: response.data.edges.map((e: { from_id: string; to_id: string; relation_type: string; confidence: number }) => ({
        source: e.from_id,
        target: e.to_id,
        relation: e.relation_type,
        weight: e.confidence,
      })),
    };
  },

  getStats: async (): Promise<GraphStats> => {
    const response = await api.get('/admin/graph/stats');
    return response.data;
  },
};

// Memory API
export const memoryApi = {
  getAll: async (limit = 100, offset = 0): Promise<Memory[]> => {
    const response = await api.get('/memories', { params: { limit, per_page: limit, page: Math.floor(offset / limit) + 1 } });
    // Handle paginated response
    return response.data.data || response.data;
  },

  getById: async (id: string): Promise<Memory> => {
    const response = await api.get(`/memories/${id}`);
    return response.data;
  },

  search: async (query: string, limit = 10): Promise<Memory[]> => {
    const response = await api.post('/memories/search', { query, limit });
    // Response is array of {memory, similarity}
    return response.data.map((item: { memory: Memory }) => item.memory || item);
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

// Model info
export interface ModelInfo {
  id: string;
  name: string;
  size_bytes?: number;
  parameters?: number;
  context_length?: number;
  is_embedding_model: boolean;
}

// System API
export const systemApi = {
  getHealth: async (): Promise<SystemHealth> => {
    const response = await api.get('/health');
    return response.data;
  },

  getModels: async (): Promise<ModelInfo[]> => {
    const response = await api.get('/models');
    return response.data;
  },

  getVram: async (): Promise<{ total_mb: number; used_mb: number; free_mb: number }> => {
    const response = await api.get('/system/vram');
    return response.data;
  },
};

export default api;
