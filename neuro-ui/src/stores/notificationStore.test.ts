import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useNotificationStore } from './notificationStore'

describe('notificationStore', () => {
  beforeEach(() => {
    useNotificationStore.setState({ activeNotifications: {} })
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('notifySection sets a timestamp for the section', () => {
    const { notifySection } = useNotificationStore.getState()
    notifySection('/checklists')
    const { activeNotifications } = useNotificationStore.getState()
    expect(activeNotifications['/checklists']).toBeDefined()
    expect(typeof activeNotifications['/checklists']).toBe('number')
  })

  it('clearNotification removes the section entry', () => {
    const { notifySection, clearNotification } = useNotificationStore.getState()
    notifySection('/music')
    expect(useNotificationStore.getState().activeNotifications['/music']).toBeDefined()
    clearNotification('/music')
    expect(useNotificationStore.getState().activeNotifications['/music']).toBeUndefined()
  })

  it('notifyFromTools maps known tool names to sections', () => {
    const { notifyFromTools } = useNotificationStore.getState()
    notifyFromTools(['create_checklist', 'add_checklist_item'])
    const { activeNotifications } = useNotificationStore.getState()
    expect(activeNotifications['/checklists']).toBeDefined()
    expect(Object.keys(activeNotifications)).toHaveLength(1)
  })

  it('notifyFromTools ignores unknown tool names', () => {
    const { notifyFromTools } = useNotificationStore.getState()
    notifyFromTools(['unknown_tool'])
    expect(Object.keys(useNotificationStore.getState().activeNotifications)).toHaveLength(0)
  })

  it('auto-clears notification after duration', () => {
    const { notifySection } = useNotificationStore.getState()
    notifySection('/checklists')
    expect(useNotificationStore.getState().activeNotifications['/checklists']).toBeDefined()
    vi.advanceTimersByTime(5000)
    expect(useNotificationStore.getState().activeNotifications['/checklists']).toBeUndefined()
  })
})