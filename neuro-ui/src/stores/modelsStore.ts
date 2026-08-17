import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { MODEL_CATALOG, OPENAI_CATALOG, OLLAMA_CLOUD_CATALOG, ModelCatalogEntry, modelApi } from '../api/models';
import { useToastStore } from './toastStore';

export type LlmProvider = 'ollama' | 'openai' | 'ollama_cloud';

interface ModelsState {
  /** ollama_name of installed models. */
  installed: string[];
  /** ollama_name -> 0..100 download progress (simulated). */
  downloading: Record<string, number>;
  /** Actively used model (chat). */
  activeModel: string | null;
  /** Detected LLM provider. */
  provider: LlmProvider;
  /** Detect the provider from the backend health endpoint. */
  detectProvider: () => Promise<void>;
  install: (entry: ModelCatalogEntry) => Promise<void>;
  setActive: (name: string) => void;
  isInstalled: (name: string) => boolean;
  compatible: (entry: ModelCatalogEntry, vramGb: number) => boolean;
}

export const useModelsStore = create<ModelsState>()(
  persist(
    (set, get) => ({
      installed: [],
      downloading: {},
      activeModel: null,
      provider: 'ollama',

      detectProvider: async () => {
        try {
          const health = await modelApi.health();
          const provider: LlmProvider =
            health.provider === 'openai' || health.provider === 'ollama_cloud'
              ? health.provider
              : 'ollama';
          set({ provider });
        } catch {
          // Backend unreachable; keep the current provider.
        }
      },

      install: async (entry) => {
        if (get().installed.includes(entry.ollama_name)) return;
        const toastId = useToastStore.getState().toast(
          `Descargando ${entry.name}...`,
          'info'
        );
        // Simulate progress; the backend pull is synchronous.
        let p = 0;
        const timer = setInterval(() => {
          p = Math.min(100, p + 8);
          useToastStore.getState().updateProgress(toastId, p);
        }, 400);

        try {
          await modelApi.pull(entry.ollama_name);
          clearInterval(timer);
          useToastStore.getState().updateProgress(toastId, 100);
          setTimeout(() => useToastStore.getState().dismiss(toastId), 600);
          useToastStore.getState().toast(`Modelo ${entry.name} instalado`, 'success');
          set((s) => ({ installed: [...s.installed, entry.ollama_name] }));
        } catch (e) {
          clearInterval(timer);
          useToastStore.getState().dismiss(toastId);
          useToastStore
            .getState()
            .toast(`Error descargando ${entry.name}: ${(e as Error)?.message ?? 'desconocido'}`, 'error');
        }
      },

      setActive: (name) => set({ activeModel: name }),
      isInstalled: (name) => get().installed.includes(name),
      compatible: (entry, vramGb) => vramGb >= entry.vram_required_gb,
    }),
    {
      name: 'tachikoma-models',
      partialize: (s) => ({ installed: s.installed, activeModel: s.activeModel }),
    }
  )
);

export { MODEL_CATALOG, OPENAI_CATALOG, OLLAMA_CLOUD_CATALOG };
export type { ModelCatalogEntry };
