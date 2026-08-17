import api from './client';

export interface HardwareProfile {
  cpu_cores: number;
  ram_gb: number;
  disk_free_gb: number;
  gpu_vendor: string;
  gpu_model: string;
  vram_gb: number;
  recommended_model_size: string;
}

export const systemApi = {
  hardware: async (): Promise<HardwareProfile> => {
    const response = await api.get<HardwareProfile>('/system/hardware');
    return response.data;
  },
};
