import { describe, it, expect, vi, beforeEach } from 'vitest'

vi.mock('../api/client', () => {
  const checklistsApi = {
    list: vi.fn(),
    get: vi.fn(),
    create: vi.fn(),
    update: vi.fn(),
    delete: vi.fn(),
    addItem: vi.fn(),
    updateItem: vi.fn(),
    deleteItem: vi.fn(),
    toggleItem: vi.fn(),
    importMarkdown: vi.fn(),
  }
  return { checklistsApi }
})

import { useChecklistStore } from './checklistStore'
import { checklistsApi } from '../api/client'

function makeChecklistDto(id: string, title: string, opts: Partial<any> = {}) {
  const now = new Date().toISOString()
  return {
    id,
    title,
    description: null,
    priority: 3,
    due_date: null,
    notification_interval: null,
    is_archived: false,
    total_items: 0,
    completed_items: 0,
    created_at: now,
    updated_at: now,
    items: [],
    ...opts,
  }
}

function makeItemDto(id: string, content: string, opts: Partial<any> = {}) {
  const now = new Date().toISOString()
  return {
    id,
    content,
    is_completed: false,
    completed_at: null,
    order: 0,
    created_at: now,
    updated_at: now,
    ...opts,
  }
}

describe('checklistStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useChecklistStore.setState({
      checklists: [],
      selectedChecklistId: null,
      isLoading: false,
      error: null,
    })
  })

  it('fetchChecklists loads and orders checklists', async () => {
    vi.mocked(checklistsApi.list).mockResolvedValue({
      data: [
        makeChecklistDto('c1', 'First'),
        makeChecklistDto('c2', 'Second'),
      ],
      total: 2, page: 1, per_page: 100, total_pages: 1,
    } as any)

    await useChecklistStore.getState().fetchChecklists()
    const { checklists, isLoading } = useChecklistStore.getState()
    expect(isLoading).toBe(false)
    expect(checklists).toHaveLength(2)
    expect(checklists[0].id).toBe('c1')
    expect(checklists[0].order).toBe(0)
    expect(checklists[1].order).toBe(1)
  })

  it('createChecklist prepends the new checklist and reindexes order', async () => {
    useChecklistStore.setState({
      checklists: [{ ...makeChecklistDto('c1', 'Old'), order: 0, items: [] }] as any,
    })
    vi.mocked(checklistsApi.create).mockResolvedValue(makeChecklistDto('c2', 'New') as any)

    const created = await useChecklistStore.getState().createChecklist('New')
    const { checklists } = useChecklistStore.getState()
    expect(created.title).toBe('New')
    expect(checklists).toHaveLength(2)
    expect(checklists[0].id).toBe('c2')
    expect(checklists[0].order).toBe(0)
    expect(checklists[1].order).toBe(1)
  })

  it('deleteChecklist removes the checklist and clears selection if matching', async () => {
    useChecklistStore.setState({
      checklists: [
        { ...makeChecklistDto('c1', 'A'), order: 0, items: [] },
        { ...makeChecklistDto('c2', 'B'), order: 1, items: [] },
      ] as any,
      selectedChecklistId: 'c1',
    })
    vi.mocked(checklistsApi.delete).mockResolvedValue(undefined as any)

    await useChecklistStore.getState().deleteChecklist('c1')
    const { checklists, selectedChecklistId } = useChecklistStore.getState()
    expect(checklists).toHaveLength(1)
    expect(checklists[0].id).toBe('c2')
    expect(selectedChecklistId).toBeNull()
  })

  it('toggleItem updates the item and completedItems counter', async () => {
    const itemDto = makeItemDto('i1', 'Task', { is_completed: true, completed_at: new Date().toISOString() })
    useChecklistStore.setState({
      checklists: [{
        ...makeChecklistDto('c1', 'List'),
        order: 0,
        totalItems: 1,
        completedItems: 0,
        items: [{ ...makeItemDto('i1', 'Task'), order: 0 }] as any,
      }] as any,
    })
    vi.mocked(checklistsApi.toggleItem).mockResolvedValue(itemDto as any)

    await useChecklistStore.getState().toggleItem('c1', 'i1')
    const checklist = useChecklistStore.getState().checklists[0]
    expect(checklist.items[0].isCompleted).toBe(true)
    expect(checklist.completedItems).toBe(1)
  })

  it('reorderChecklists updates order based on provided array', () => {
    useChecklistStore.setState({
      checklists: [
        { ...makeChecklistDto('c1', 'A'), order: 0, items: [] },
        { ...makeChecklistDto('c2', 'B'), order: 1, items: [] },
      ] as any,
    })

    useChecklistStore.getState().reorderChecklists([
      useChecklistStore.getState().checklists[1],
      useChecklistStore.getState().checklists[0],
    ])
    const { checklists } = useChecklistStore.getState()
    expect(checklists[0].id).toBe('c2')
    expect(checklists[0].order).toBe(0)
    expect(checklists[1].id).toBe('c1')
    expect(checklists[1].order).toBe(1)
  })
})