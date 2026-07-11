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
    return {
      nodes: response.data.nodes,
      edges: response.data.edges.map((e: { from_id: string; to_id: string; relation: string; confidence: number }) => ({
        source: e.from_id,
        target: e.to_id,
        relation: e.relation,
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
  update: async (id: string, memory: Partial<Memory>): Promise<Memory> => {
    const response = await api.patch(`/memories/${id}`, memory);
    return response.data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/memories/${id}`);
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
};

export default api;
