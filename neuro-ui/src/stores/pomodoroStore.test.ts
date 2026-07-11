import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

vi.mock('../api/client', () => {
  const pomodoroApi = {
    getActiveSession: vi.fn(),
    startSession: vi.fn(),
    pauseSession: vi.fn(),
    resumeSession: vi.fn(),
    completeSession: vi.fn(),
    cancelSession: vi.fn(),
    getSettings: vi.fn(),
    updateSettings: vi.fn(),
    getHistory: vi.fn(),
    getDailyStats: vi.fn(),
    getWeeklyStats: vi.fn(),
  }
  const isSessionRunning = (status: string) =>
    status === 'working' || status === 'short_break' || status === 'long_break'
  return { pomodoroApi, isSessionRunning }
})

import { usePomodoroStore } from './pomodoroStore'
import { pomodoroApi } from '../api/client'

function makeSessionDto(opts: Partial<any> = {}) {
  const now = new Date().toISOString()
  return {
    id: 's1',
    session_type: 'work',
    status: 'working',
    duration_minutes: 25,
    elapsed_seconds: 0,
    started_at: now,
    paused_at: null,
    completed_at: null,
    task_description: null,
    created_at: now,
    updated_at: now,
    ...opts,
  }
}

function makeSettingsDto(opts: Partial<any> = {}) {
  return {
    work_duration_minutes: 25,
    short_break_minutes: 5,
    long_break_minutes: 15,
    sessions_until_long_break: 4,
    auto_start_breaks: false,
    auto_start_work: false,
    ...opts,
  }
}

describe('pomodoroStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    usePomodoroStore.setState({
      activeSession: null,
      settings: {
        workDurationMinutes: 25,
        shortBreakMinutes: 5,
        longBreakMinutes: 15,
        sessionsUntilLongBreak: 4,
        autoStartBreaks: false,
        autoStartWork: false,
      },
      todayStats: null,
      weeklyStats: [],
      history: [],
      isLoading: false,
      error: null,
      remainingSeconds: 0,
      timerIntervalId: null,
      completedWorkSessions: 0,
    })
  })

  afterEach(() => {
    usePomodoroStore.getState().stopTimer()
  })

  it('startSession creates a running session and sets remainingSeconds', async () => {
    vi.mocked(pomodoroApi.startSession).mockResolvedValue(makeSessionDto() as any)
    await usePomodoroStore.getState().startSession('work', 'writing tests')
    const state = usePomodoroStore.getState()
    expect(state.activeSession).not.toBeNull()
    expect(state.activeSession?.sessionType).toBe('work')
    expect(state.remainingSeconds).toBe(25 * 60)
    expect(pomodoroApi.startSession).toHaveBeenCalledWith(expect.objectContaining({
      session_type: 'work',
      task_description: 'writing tests',
      duration_minutes: 25,
    }))
  })

  it('pauseSession stops the timer and updates activeSession to paused', async () => {
    vi.mocked(pomodoroApi.startSession).mockResolvedValue(makeSessionDto() as any)
    vi.mocked(pomodoroApi.pauseSession).mockResolvedValue(makeSessionDto({ status: 'paused' }) as any)
    await usePomodoroStore.getState().startSession('work')
    await usePomodoroStore.getState().pauseSession()
    const state = usePomodoroStore.getState()
    expect(state.activeSession?.status).toBe('paused')
    expect(state.timerIntervalId).toBeNull()
  })

  it('completeSession increments completedWorkSessions for work sessions', async () => {
    vi.mocked(pomodoroApi.startSession).mockResolvedValue(makeSessionDto() as any)
    vi.mocked(pomodoroApi.completeSession).mockResolvedValue(makeSessionDto({ status: 'completed' }) as any)
    vi.mocked(pomodoroApi.getDailyStats).mockResolvedValue({
      date: '2026-07-10',
      total_sessions: 1,
      completed_sessions: 1,
      total_work_minutes: 25,
      total_break_minutes: 0,
    } as any)

    await usePomodoroStore.getState().startSession('work')
    await usePomodoroStore.getState().completeSession()
    const state = usePomodoroStore.getState()
    expect(state.completedWorkSessions).toBe(1)
    expect(state.remainingSeconds).toBe(0)
  })

  it('cancelSession clears the active session', async () => {
    vi.mocked(pomodoroApi.startSession).mockResolvedValue(makeSessionDto() as any)
    vi.mocked(pomodoroApi.cancelSession).mockResolvedValue(undefined as any)
    await usePomodoroStore.getState().startSession('work')
    await usePomodoroStore.getState().cancelSession()
    const state = usePomodoroStore.getState()
    expect(state.activeSession).toBeNull()
    expect(state.remainingSeconds).toBe(0)
  })

  it('getNextSessionType returns long_break when session count is multiple of 4', () => {
    usePomodoroStore.setState({ completedWorkSessions: 3 } as any)
    expect(usePomodoroStore.getState().getNextSessionType()).toBe('long_break')
    usePomodoroStore.setState({ completedWorkSessions: 1 } as any)
    expect(usePomodoroStore.getState().getNextSessionType()).toBe('short_break')
  })

  it('tick decrements remainingSeconds while running', () => {
    usePomodoroStore.setState({
      activeSession: {
        id: 's1',
        sessionType: 'work',
        status: 'working',
        durationMinutes: 25,
        elapsedSeconds: 0,
        startedAt: new Date(),
        createdAt: new Date(),
        updatedAt: new Date(),
      },
      remainingSeconds: 60,
      timerIntervalId: 999,
    } as any)
    usePomodoroStore.getState().tick()
    expect(usePomodoroStore.getState().remainingSeconds).toBe(59)
  })
})