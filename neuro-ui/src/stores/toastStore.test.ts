import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useToastStore } from './toastStore'

describe('toastStore', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    useToastStore.setState({ toasts: [] })
  })

  it('adds a toast and returns its id', () => {
    const id = useToastStore.getState().toast('hello', 'success')
    const toasts = useToastStore.getState().toasts
    expect(toasts).toHaveLength(1)
    expect(toasts[0].id).toBe(id)
    expect(toasts[0].message).toBe('hello')
    expect(toasts[0].type).toBe('success')
  })

  it('defaults type to info and supports action label', () => {
    useToastStore.getState().toast('msg', undefined, { actionLabel: 'Undo' })
    expect(useToastStore.getState().toasts[0].type).toBe('info')
    expect(useToastStore.getState().toasts[0].actionLabel).toBe('Undo')
  })

  it('auto-dismisses after the duration', () => {
    useToastStore.getState().toast('bye', 'info')
    expect(useToastStore.getState().toasts).toHaveLength(1)
    vi.advanceTimersByTime(4100)
    expect(useToastStore.getState().toasts).toHaveLength(0)
  })

  it('updates progress on a toast', () => {
    const id = useToastStore.getState().toast('downloading')
    useToastStore.getState().updateProgress(id, 50)
    expect(useToastStore.getState().toasts[0].progress).toBe(50)
  })

  it('dismisses a single toast and clears all', () => {
    useToastStore.getState().toast('a')
    useToastStore.getState().toast('b')
    useToastStore.getState().dismiss(useToastStore.getState().toasts[0].id)
    expect(useToastStore.getState().toasts).toHaveLength(1)
    useToastStore.getState().clear()
    expect(useToastStore.getState().toasts).toHaveLength(0)
  })
})
