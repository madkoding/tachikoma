import { describe, it, expect, beforeEach } from 'vitest'
import { useChatStore, Message, Conversation } from './chatStore'

function makeMessage(id: string, role: Message['role'], content: string): Message {
  return { id, role, content, createdAt: new Date() }
}

function makeConversation(id: string, title: string, messages: Message[] = []): Conversation {
  const now = new Date()
  return { id, title, messages, createdAt: now, updatedAt: now }
}

describe('chatStore', () => {
  beforeEach(() => {
    useChatStore.setState({
      conversations: [],
      currentConversationId: null,
      isLoading: false,
      error: null,
    })
  })

  it('setCurrentConversation sets the id', () => {
    const { setCurrentConversation } = useChatStore.getState()
    setCurrentConversation('conv-1')
    expect(useChatStore.getState().currentConversationId).toBe('conv-1')
    setCurrentConversation(null)
    expect(useChatStore.getState().currentConversationId).toBeNull()
  })

  it('addConversation prepends a new conversation', () => {
    const { addConversation } = useChatStore.getState()
    addConversation(makeConversation('c1', 'First'))
    addConversation(makeConversation('c2', 'Second'))
    const { conversations } = useChatStore.getState()
    expect(conversations).toHaveLength(2)
    expect(conversations[0].id).toBe('c2')
  })

  it('addMessage appends to the matching conversation and updates updatedAt', () => {
    const { addConversation, addMessage } = useChatStore.getState()
    const conv = makeConversation('c1', 'Title')
    addConversation(conv)
    const before = useChatStore.getState().conversations[0].updatedAt
    addMessage('c1', makeMessage('m1', 'user', 'hello'))
    const { conversations } = useChatStore.getState()
    expect(conversations[0].messages).toHaveLength(1)
    expect(conversations[0].messages[0].content).toBe('hello')
    expect(conversations[0].updatedAt.getTime()).toBeGreaterThanOrEqual(before.getTime())
  })

  it('updateMessage applies updater to the right message', () => {
    const { addConversation, addMessage, updateMessage } = useChatStore.getState()
    addConversation(makeConversation('c1', 'Title'))
    addMessage('c1', makeMessage('m1', 'user', 'hello'))
    updateMessage('c1', 'm1', (msg) => ({ ...msg, content: 'edited' }))
    expect(useChatStore.getState().conversations[0].messages[0].content).toBe('edited')
  })

  it('deleteConversation removes it and clears currentConversationId if matching', () => {
    const { addConversation, setCurrentConversation, deleteConversation } = useChatStore.getState()
    addConversation(makeConversation('c1', 'A'))
    addConversation(makeConversation('c2', 'B'))
    setCurrentConversation('c1')
    deleteConversation('c1')
    const { conversations, currentConversationId } = useChatStore.getState()
    expect(conversations).toHaveLength(1)
    expect(conversations[0].id).toBe('c2')
    expect(currentConversationId).toBeNull()
  })
})