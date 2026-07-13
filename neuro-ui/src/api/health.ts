import api from './client';

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

export const healthApi = {
  check: async (): Promise<HealthResponse> => {
    const response = await api.get<HealthResponse>('/health');
    return response.data;
  },
};
