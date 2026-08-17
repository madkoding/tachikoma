import { describe, it, expect, beforeEach, vi } from 'vitest'
import { useModelsStore, MODEL_CATALOG } from './modelsStore'

vi.mock('../api/models', () => ({
  MODEL_CATALOG: [
    { id: 'a', name: 'A', ollama_name: 'a:1b', vram_required_gb: 4, size_tier: 'small', is_embedding: false },
    { id: 'b', name: 'B', ollama_name: 'b:7b', vram_required_gb: 8, size_tier: 'medium', is_embedding: false },
  ],
  OPENAI_CATALOG: [
    { id: 'gpt', name: 'GPT', ollama_name: 'gpt-4o-mini', vram_required_gb: 0, size_tier: 'small', is_embedding: false },
  ],
  OLLAMA_CLOUD_CATALOG: [
    { id: 'llama', name: 'Llama', ollama_name: 'llama3.3:70b', vram_required_gb: 0, size_tier: 'large', is_embedding: false },
  ],
  modelApi: {
    pull: vi.fn().mockResolvedValue({ name: 'x', status: 'ok' }),
    health: vi.fn().mockResolvedValue({ provider: 'ollama', provider_url: 'http://x', healthy: true }),
  },
}))

describe('modelsStore', () => {
  beforeEach(() => {
    useModelsStore.setState({ installed: [], downloading: {}, activeModel: null, provider: 'ollama' })
  })

  it('has a catalog', () => {
    expect(MODEL_CATALOG.length).toBeGreaterThan(0)
  })

  it('isInstalled reflects installed list', () => {
    useModelsStore.setState({ installed: ['a:1b'] })
    expect(useModelsStore.getState().isInstalled('a:1b')).toBe(true)
    expect(useModelsStore.getState().isInstalled('b:7b')).toBe(false)
  })

  it('compatible checks vram', () => {
    const s = useModelsStore.getState()
    expect(s.compatible(MODEL_CATALOG[0], 4)).toBe(true)
    expect(s.compatible(MODEL_CATALOG[1], 4)).toBe(false)
  })

  it('setActive updates active model', () => {
    useModelsStore.getState().setActive('b:7b')
    expect(useModelsStore.getState().activeModel).toBe('b:7b')
  })

  it('detectProvider sets openai when backend reports openai', async () => {
    const { modelApi } = await import('../api/models')
    ;(modelApi.health as ReturnType<typeof vi.fn>).mockResolvedValue({ provider: 'openai', provider_url: 'http://x', healthy: true })
    await useModelsStore.getState().detectProvider()
    expect(useModelsStore.getState().provider).toBe('openai')
  })

  it('detectProvider sets ollama_cloud when backend reports ollama_cloud', async () => {
    const { modelApi } = await import('../api/models')
    ;(modelApi.health as ReturnType<typeof vi.fn>).mockResolvedValue({ provider: 'ollama_cloud', provider_url: 'http://x', healthy: true })
    await useModelsStore.getState().detectProvider()
    expect(useModelsStore.getState().provider).toBe('ollama_cloud')
  })
})
