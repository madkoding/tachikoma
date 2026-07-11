import { describe, it, expect, beforeEach } from 'vitest'
import { usePerformanceStore } from './performanceStore'

describe('performanceStore', () => {
  beforeEach(() => {
    usePerformanceStore.setState({
      level: 'high',
      autoDetect: true,
      settings: usePerformanceStore.getState().settings,
      currentFPS: 60,
      fpsHistory: [],
    })
  })

  it('has default level high with high preset settings', () => {
    const state = usePerformanceStore.getState()
    expect(state.level).toBe('high')
    expect(state.settings.enableSpectrumAnalyzer).toBe(true)
    expect(state.settings.spectrumBarCount).toBe(32)
  })

  it('setLevel applies the matching preset and disables autoDetect', () => {
    usePerformanceStore.getState().setLevel('minimal')
    const state = usePerformanceStore.getState()
    expect(state.level).toBe('minimal')
    expect(state.settings.enableSpectrumAnalyzer).toBe(false)
    expect(state.settings.spectrumBarCount).toBe(8)
    expect(state.autoDetect).toBe(false)
  })

  it('setLevel medium disables blur but keeps glow', () => {
    usePerformanceStore.getState().setLevel('medium')
    const { settings } = usePerformanceStore.getState()
    expect(settings.enableBlurEffects).toBe(false)
    expect(settings.enableGlowEffects).toBe(true)
  })

  it('updateFPS averages over history and keeps last 30', () => {
    const { updateFPS } = usePerformanceStore.getState()
    updateFPS(60)
    updateFPS(30)
    const state = usePerformanceStore.getState()
    expect(state.fpsHistory).toHaveLength(2)
    expect(state.currentFPS).toBe(45)
  })

  it('updateFPS keeps at most 30 readings', () => {
    const { updateFPS } = usePerformanceStore.getState()
    for (let i = 0; i < 35; i++) updateFPS(60)
    expect(usePerformanceStore.getState().fpsHistory).toHaveLength(30)
  })

  it('getSettings returns the current settings object', () => {
    usePerformanceStore.getState().setLevel('low')
    const settings = usePerformanceStore.getState().getSettings()
    expect(settings.enableGlowEffects).toBe(false)
    expect(settings.spectrumFPS).toBe(15)
  })
})