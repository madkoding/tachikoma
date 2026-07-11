import { describe, it, expect, beforeEach } from 'vitest'
import { useThemeStore } from './themeStore'

describe('themeStore', () => {
  beforeEach(() => {
    useThemeStore.setState({ theme: 'system' })
  })

  it('has initial theme of system', () => {
    expect(useThemeStore.getState().theme).toBe('system')
  })

  it('setTheme updates the theme to dark', () => {
    useThemeStore.getState().setTheme('dark')
    expect(useThemeStore.getState().theme).toBe('dark')
  })

  it('setTheme updates the theme to light', () => {
    useThemeStore.getState().setTheme('light')
    expect(useThemeStore.getState().theme).toBe('light')
  })

  it('setTheme toggles between light and dark', () => {
    const { setTheme } = useThemeStore.getState()
    setTheme('dark')
    expect(useThemeStore.getState().theme).toBe('dark')
    setTheme('light')
    expect(useThemeStore.getState().theme).toBe('light')
  })
})