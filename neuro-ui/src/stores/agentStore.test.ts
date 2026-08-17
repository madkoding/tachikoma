import { describe, it, expect, beforeEach } from 'vitest'
import { AGENT_TEMPLATES, useAgentStore, getActiveTemplate } from './agentStore'

describe('agentStore', () => {
  beforeEach(() => {
    useAgentStore.setState({ activeAgentId: null })
  })

  it('has 5 templates', () => {
    expect(AGENT_TEMPLATES).toHaveLength(5)
  })

  it('every template has a system prompt and unique id', () => {
    const ids = new Set(AGENT_TEMPLATES.map((t) => t.id))
    expect(ids.size).toBe(AGENT_TEMPLATES.length)
    for (const t of AGENT_TEMPLATES) {
      expect(t.systemPrompt.length).toBeGreaterThan(0)
      expect(t.name.length).toBeGreaterThan(0)
    }
  })

  it('sets and clears the active agent', () => {
    useAgentStore.getState().setActiveAgent('code-reviewer')
    expect(getActiveTemplate()?.id).toBe('code-reviewer')
    useAgentStore.getState().setActiveAgent(null)
    expect(getActiveTemplate()).toBeNull()
  })

  it('returns null for an unknown agent id', () => {
    useAgentStore.getState().setActiveAgent('does-not-exist')
    expect(getActiveTemplate()).toBeNull()
  })
})
