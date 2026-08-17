import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface TachikomaPlugin {
  id: string;
  name: string;
  version: string;
  description: string;
  type: 'tool' | 'memory_connector' | 'model_adapter';
  /** Tool names this plugin contributes to agents. */
  tools: string[];
}

// Built-in plugins discoverable in the marketplace. `ponytail:` static list for
// now; a real registry sync lands when the backend exposes a `/plugins` endpoint.
export const PLUGIN_CATALOG: TachikomaPlugin[] = [
  {
    id: 'hello-tool',
    name: 'Hello Tool',
    version: '0.1.0',
    description: 'Un plugin de ejemplo que añade una herramienta de saludo.',
    type: 'tool',
    tools: ['say_hello'],
  },
  {
    id: 'weather',
    name: 'Weather Check',
    version: '0.2.0',
    description: 'Consulta el clima de una ciudad (necesita red).',
    type: 'tool',
    tools: ['get_weather'],
  },
  {
    id: 'pinecone-memory',
    name: 'Pinecone Memory',
    version: '0.1.0',
    description: 'Backend de memoria alternativo usando Pinecone.',
    type: 'memory_connector',
    tools: [],
  },
];

interface PluginState {
  installed: string[];
  install: (id: string) => void;
  uninstall: (id: string) => void;
  isInstalled: (id: string) => boolean;
}

export const usePluginStore = create<PluginState>()(
  persist(
    (set, get) => ({
      installed: [],
      install: (id) =>
        set((s) => (s.installed.includes(id) ? s : { installed: [...s.installed, id] })),
      uninstall: (id) => set((s) => ({ installed: s.installed.filter((x) => x !== id) })),
      isInstalled: (id) => get().installed.includes(id),
    }),
    { name: 'tachikoma-plugins' }
  )
);
