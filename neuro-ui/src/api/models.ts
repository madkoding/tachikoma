import api from './client';

export interface ModelCatalogEntry {
  id: string;
  name: string;
  /** Ollama tag used for pull (e.g. "qwen2.5:7b"). */
  ollama_name: string;
  description: string;
  icon: string;
  size_bytes: number;
  vram_required_gb: number;
  /** small | medium | large */
  size_tier: string;
  license: string;
  tags: string[];
  /** True when this is an embedding/vector model for GraphRAG. */
  is_embedding: boolean;
}

export const OPENAI_CATALOG: ModelCatalogEntry[] = [
  { id: 'gpt-4o-mini', name: 'GPT-4o mini', ollama_name: 'gpt-4o-mini', description: 'Rápido y económico de OpenAI.', icon: '⚡', size_bytes: 0, vram_required_gb: 0, size_tier: 'small', license: 'OpenAI', tags: ['chat', 'fast'], is_embedding: false },
  { id: 'gpt-4o', name: 'GPT-4o', ollama_name: 'gpt-4o', description: 'Modelo general de alta calidad.', icon: '🟢', size_bytes: 0, vram_required_gb: 0, size_tier: 'medium', license: 'OpenAI', tags: ['chat'], is_embedding: false },
  { id: 'gpt-4.1', name: 'GPT-4.1', ollama_name: 'gpt-4.1', description: 'Última generación de OpenAI.', icon: '🔵', size_bytes: 0, vram_required_gb: 0, size_tier: 'large', license: 'OpenAI', tags: ['chat', 'powerful'], is_embedding: false },
  { id: 'claude-3-5-sonnet', name: 'Claude 3.5 Sonnet', ollama_name: 'claude-3-5-sonnet', description: 'Alta calidad de Anthropic (vía OpenRouter).', icon: '🟣', size_bytes: 0, vram_required_gb: 0, size_tier: 'large', license: 'Anthropic', tags: ['chat', 'powerful'], is_embedding: false },
  { id: 'llama-3.3-70b', name: 'Llama 3.3 70B', ollama_name: 'llama-3.3-70b', description: 'Open source potente (vía Groq/OpenRouter).', icon: '🟡', size_bytes: 0, vram_required_gb: 0, size_tier: 'large', license: 'Llama 3.3', tags: ['chat', 'powerful'], is_embedding: false },
  { id: 'text-embedding-3-small', name: 'Text Embedding 3 Small', ollama_name: 'text-embedding-3-small', description: 'Embeddings para GraphRAG (memoria semántica).', icon: '🧠', size_bytes: 0, vram_required_gb: 0, size_tier: 'small', license: 'OpenAI', tags: ['embed'], is_embedding: true },
];

export const OLLAMA_CLOUD_CATALOG: ModelCatalogEntry[] = [
  { id: 'llama3.2-3b', name: 'Llama 3.2 3B', ollama_name: 'llama3.2:3b', description: 'Compacto de Meta, buena comprensión.', icon: '🟡', size_bytes: 0, vram_required_gb: 0, size_tier: 'small', license: 'Llama 3.2', tags: ['chat'], is_embedding: false },
  { id: 'qwen2.5-7b', name: 'Qwen 2.5 7B', ollama_name: 'qwen2.5:7b', description: 'Buen equilibrio calidad/velocidad.', icon: '🔵', size_bytes: 0, vram_required_gb: 0, size_tier: 'medium', license: 'Apache 2.0', tags: ['chat', 'balanced'], is_embedding: false },
  { id: 'llama3.3-70b', name: 'Llama 3.3 70B', ollama_name: 'llama3.3:70b', description: 'Potente, para tareas complejas.', icon: '🟣', size_bytes: 0, vram_required_gb: 0, size_tier: 'large', license: 'Llama 3.3', tags: ['chat', 'powerful'], is_embedding: false },
  { id: 'qwen2.5-coder-32b', name: 'Qwen 2.5 Coder 32B', ollama_name: 'qwen2.5-coder:32b', description: 'Alta calidad de código.', icon: '💻', size_bytes: 0, vram_required_gb: 0, size_tier: 'large', license: 'Apache 2.0', tags: ['chat', 'code', 'powerful'], is_embedding: false },
  { id: 'nomic-embed-text', name: 'Nomic Embed Text', ollama_name: 'nomic-embed-text', description: 'Embeddings para GraphRAG (memoria semántica).', icon: '🧠', size_bytes: 0, vram_required_gb: 0, size_tier: 'small', license: 'Apache 2.0', tags: ['embed'], is_embedding: true },
];

export const MODEL_CATALOG: ModelCatalogEntry[] = [
  { id: 'qwen2.5-0.5b', name: 'Qwen 2.5 0.5B', ollama_name: 'qwen2.5:0.5b', description: 'Ultra ligero, para CPU puras y respuestas rápidas.', icon: '🐤', size_bytes: 397_000_000, vram_required_gb: 1, size_tier: 'small', license: 'Apache 2.0', tags: ['chat', 'fast'], is_embedding: false },
  { id: 'qwen2.5-3b', name: 'Qwen 2.5 3B', ollama_name: 'qwen2.5:3b', description: 'Equilibrado, corre bien sin GPU dedicada.', icon: '🟢', size_bytes: 1_900_000_000, vram_required_gb: 3, size_tier: 'small', license: 'Apache 2.0', tags: ['chat'], is_embedding: false },
  { id: 'qwen2.5-7b', name: 'Qwen 2.5 7B', ollama_name: 'qwen2.5:7b', description: 'Buen equilibrio calidad/velocidad para la mayoría.', icon: '🔵', size_bytes: 4_700_000_000, vram_required_gb: 6, size_tier: 'medium', license: 'Apache 2.0', tags: ['chat', 'balanced'], is_embedding: false },
  { id: 'llama3.2-3b', name: 'Llama 3.2 3B', ollama_name: 'llama3.2:3b', description: 'Compacto de Meta, buena comprensión.', icon: '🟡', size_bytes: 2_000_000_000, vram_required_gb: 3, size_tier: 'small', license: 'Llama 3.2', tags: ['chat'], is_embedding: false },
  { id: 'mistral-7b', name: 'Mistral 7B', ollama_name: 'mistral:7b', description: 'Fuerte en razonamiento y multilingüe.', icon: '🟣', size_bytes: 4_100_000_000, vram_required_gb: 6, size_tier: 'medium', license: 'Apache 2.0', tags: ['chat'], is_embedding: false },
  { id: 'qwen2.5-coder-7b', name: 'Qwen 2.5 Coder 7B', ollama_name: 'qwen2.5-coder:7b', description: 'Especializado en código y tareas de programación.', icon: '💻', size_bytes: 4_700_000_000, vram_required_gb: 6, size_tier: 'medium', license: 'Apache 2.0', tags: ['chat', 'code'], is_embedding: false },
  { id: 'qwen2.5-coder-14b', name: 'Qwen 2.5 Coder 14B', ollama_name: 'qwen2.5-coder:14b', description: 'Alta calidad de código. Requiere buena GPU.', icon: '🔧', size_bytes: 9_000_000_000, vram_required_gb: 12, size_tier: 'large', license: 'Apache 2.0', tags: ['chat', 'code', 'powerful'], is_embedding: false },
  { id: 'nomic-embed-text', name: 'Nomic Embed Text', ollama_name: 'nomic-embed-text', description: 'Embeddings para GraphRAG (memoria semántica).', icon: '🧠', size_bytes: 274_000_000, vram_required_gb: 1, size_tier: 'small', license: 'Apache 2.0', tags: ['embed'], is_embedding: true },
];

export const modelApi = {
  pull: async (name: string): Promise<{ name: string; status: string }> => {
    const response = await api.post<{ name: string; status: string }>('/llm/models/pull', { name });
    return response.data;
  },
  health: async (): Promise<{ provider: string; provider_url: string; healthy: boolean }> => {
    const response = await api.get<{ provider: string; provider_url: string; healthy: boolean }>('/llm/health');
    return response.data;
  },
};
