import { describe, it, expect, beforeEach } from 'vitest'
import { useOnboardingStore } from './onboardingStore'

describe('onboardingStore', () => {
  beforeEach(() => {
    useOnboardingStore.setState({ onboarded: false, modelSize: null, agentName: null })
  })

  it('starts not onboarded', () => {
    expect(useOnboardingStore.getState().onboarded).toBe(false)
  })

  it('completes with model size and agent name', () => {
    useOnboardingStore.getState().complete({ modelSize: 'medium', agentName: 'Ada' })
    const s = useOnboardingStore.getState()
    expect(s.onboarded).toBe(true)
    expect(s.modelSize).toBe('medium')
    expect(s.agentName).toBe('Ada')
  })

  it('keeps existing values when completing without opts', () => {
    useOnboardingStore.getState().complete({ modelSize: 'small', agentName: 'X' })
    useOnboardingStore.getState().complete()
    const s = useOnboardingStore.getState()
    expect(s.modelSize).toBe('small')
    expect(s.agentName).toBe('X')
  })

  it('resets to initial state', () => {
    useOnboardingStore.getState().complete({ modelSize: 'large', agentName: 'Y' })
    useOnboardingStore.getState().reset()
    const s = useOnboardingStore.getState()
    expect(s.onboarded).toBe(false)
    expect(s.modelSize).toBeNull()
    expect(s.agentName).toBeNull()
  })
})
