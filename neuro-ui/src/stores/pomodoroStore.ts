import { create } from 'zustand';
import {
  pomodoroApi,
  PomodoroSessionDto,
  PomodoroSettingsDto,
  PomodoroStatsDto,
  PomodoroSessionType,
  PomodoroSessionStatus,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export type SessionType = PomodoroSessionType;
export type SessionStatus = PomodoroSessionStatus;

export interface PomodoroSession {
  id: string;
  sessionType: SessionType;
  status: SessionStatus;
  durationMinutes: number;
  elapsedSeconds: number;
  startedAt: Date;
  pausedAt?: Date;
  completedAt?: Date;
  taskDescription?: string;
  createdAt: Date;
  updatedAt: Date;
}

export interface PomodoroSettings {
  workDurationMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  sessionsUntilLongBreak: number;
  autoStartBreaks: boolean;
  autoStartWork: boolean;
}

export interface PomodoroStats {
  date: string;
  totalSessions: number;
  completedSessions: number;
  totalWorkMinutes: number;
  totalBreakMinutes: number;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function sessionDtoToModel(dto: PomodoroSessionDto): PomodoroSession {
  return {
    id: dto.id,
    sessionType: dto.session_type,
    status: dto.status,
    durationMinutes: dto.duration_minutes,
    elapsedSeconds: dto.elapsed_seconds,
    startedAt: new Date(dto.started_at),
    pausedAt: dto.paused_at ? new Date(dto.paused_at) : undefined,
    completedAt: dto.completed_at ? new Date(dto.completed_at) : undefined,
    taskDescription: dto.task_description,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function settingsDtoToModel(dto: PomodoroSettingsDto): PomodoroSettings {
  return {
    workDurationMinutes: dto.work_duration_minutes,
    shortBreakMinutes: dto.short_break_minutes,
    longBreakMinutes: dto.long_break_minutes,
    sessionsUntilLongBreak: dto.sessions_until_long_break,
    autoStartBreaks: dto.auto_start_breaks,
    autoStartWork: dto.auto_start_work,
  };
}

function statsDtoToModel(dto: PomodoroStatsDto): PomodoroStats {
  return {
    date: dto.date,
    totalSessions: dto.total_sessions,
    completedSessions: dto.completed_sessions,
    totalWorkMinutes: dto.total_work_minutes,
    totalBreakMinutes: dto.total_break_minutes,
  };
}

// =============================================================================
// Store State & Actions
// =============================================================================

interface PomodoroState {
  // State
  activeSession: PomodoroSession | null;
  settings: PomodoroSettings;
  todayStats: PomodoroStats | null;
  weeklyStats: PomodoroStats[];
  history: PomodoroSession[];
  isLoading: boolean;
  error: string | null;
  
  // Timer state (local, calculated from activeSession)
  remainingSeconds: number;
  timerIntervalId: number | null;
  
  // Current session count (for long break calculation)
  completedWorkSessions: number;
  
  // Actions
  fetchActiveSession: () => Promise<void>;
  fetchSettings: () => Promise<void>;
  fetchTodayStats: () => Promise<void>;
  fetchWeeklyStats: () => Promise<void>;
  fetchHistory: () => Promise<void>;
  
  startSession: (type: SessionType, taskDescription?: string) => Promise<void>;
  pauseSession: () => Promise<void>;
  resumeSession: () => Promise<void>;
  completeSession: () => Promise<void>;
  cancelSession: () => Promise<void>;
  
  updateSettings: (settings: Partial<PomodoroSettings>) => Promise<void>;
  
  // Local timer management
  startTimer: () => void;
  stopTimer: () => void;
  tick: () => void;
  
  // Helpers
  calculateRemainingSeconds: (session: PomodoroSession) => number;
  getNextSessionType: () => SessionType;
}

// Default settings (will be overwritten by API)
const DEFAULT_SETTINGS: PomodoroSettings = {
  workDurationMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  sessionsUntilLongBreak: 4,
  autoStartBreaks: false,
  autoStartWork: false,
};

export const usePomodoroStore = create<PomodoroState>((set, get) => ({
  // Initial state
  activeSession: null,
  settings: DEFAULT_SETTINGS,
  todayStats: null,
  weeklyStats: [],
  history: [],
  isLoading: false,
  error: null,
  remainingSeconds: 0,
  timerIntervalId: null,
  completedWorkSessions: 0,
  
  // ==========================================================================
  // Helper Functions
  // ==========================================================================
  
  calculateRemainingSeconds: (session: PomodoroSession): number => {
    const totalSeconds = session.durationMinutes * 60;
    
    if (session.status === 'running') {
      // Calculate elapsed time since start
      const now = new Date();
      const startTime = session.startedAt;
      const elapsedSinceStart = Math.floor((now.getTime() - startTime.getTime()) / 1000);
      // Add any previously elapsed seconds (from pauses)
      const totalElapsed = session.elapsedSeconds + elapsedSinceStart;
      return Math.max(0, totalSeconds - totalElapsed);
    } else if (session.status === 'paused') {
      return Math.max(0, totalSeconds - session.elapsedSeconds);
    }
    
    return 0;
  },
  
  getNextSessionType: (): SessionType => {
    const { completedWorkSessions, settings } = get();
    
    // If we just completed a work session
    if ((completedWorkSessions + 1) % settings.sessionsUntilLongBreak === 0) {
      return 'long_break';
    }
    return 'short_break';
  },
  
  // ==========================================================================
  // Timer Management
  // ==========================================================================
  
  startTimer: () => {
    const { timerIntervalId } = get();
    
    // Clear existing interval if any
    if (timerIntervalId !== null) {
      window.clearInterval(timerIntervalId);
    }
    
    // Start new interval
    const intervalId = window.setInterval(() => {
      get().tick();
    }, 1000);
    
    set({ timerIntervalId: intervalId });
  },
  
  stopTimer: () => {
    const { timerIntervalId } = get();
    if (timerIntervalId !== null) {
      window.clearInterval(timerIntervalId);
      set({ timerIntervalId: null });
    }
  },
  
  tick: () => {
    const { activeSession, remainingSeconds, completeSession, settings } = get();
    
    if (!activeSession || activeSession.status !== 'running') {
      get().stopTimer();
      return;
    }
    
    if (remainingSeconds <= 1) {
      // Timer completed
      get().stopTimer();
      completeSession();
      
      // Play notification sound
      try {
        const audio = new Audio('/sounds/bell.mp3');
        audio.play().catch(() => {});
      } catch {
        // Ignore audio errors
      }
      
      // Show browser notification
      if (Notification.permission === 'granted') {
        const title = activeSession.sessionType === 'work' 
          ? '🍅 Work session complete!' 
          : '☕ Break time over!';
        const body = activeSession.sessionType === 'work'
          ? 'Time for a break!'
          : 'Ready to get back to work?';
        new Notification(title, { body, icon: '/favicon.ico' });
      }
      
      // Auto-start next session if enabled
      if (activeSession.sessionType === 'work' && settings.autoStartBreaks) {
        const nextType = get().getNextSessionType();
        setTimeout(() => get().startSession(nextType), 2000);
      } else if (activeSession.sessionType !== 'work' && settings.autoStartWork) {
        setTimeout(() => get().startSession('work'), 2000);
      }
      
      return;
    }
    
    set({ remainingSeconds: remainingSeconds - 1 });
  },
  
  // ==========================================================================
  // API Actions
  // ==========================================================================
  
  fetchActiveSession: async () => {
    try {
      set({ isLoading: true, error: null });
      const dto = await pomodoroApi.getActiveSession();
      
      if (dto) {
        const session = sessionDtoToModel(dto);
        const remainingSeconds = get().calculateRemainingSeconds(session);
        
        set({ 
          activeSession: session,
          remainingSeconds,
        });
        
        // Start timer if session is running
        if (session.status === 'running') {
          get().startTimer();
        }
      } else {
        set({ activeSession: null, remainingSeconds: 0 });
      }
    } catch (error) {
      console.error('Error fetching active session:', error);
      set({ error: 'Failed to fetch active session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  fetchSettings: async () => {
    try {
      const dto = await pomodoroApi.getSettings();
      set({ settings: settingsDtoToModel(dto) });
    } catch (error) {
      console.error('Error fetching settings:', error);
      // Keep default settings on error
    }
  },
  
  fetchTodayStats: async () => {
    try {
      const dto = await pomodoroApi.getDailyStats();
      set({ todayStats: statsDtoToModel(dto) });
    } catch (error) {
      console.error('Error fetching today stats:', error);
    }
  },
  
  fetchWeeklyStats: async () => {
    try {
      const dtos = await pomodoroApi.getWeeklyStats();
      set({ weeklyStats: dtos.map(statsDtoToModel) });
    } catch (error) {
      console.error('Error fetching weekly stats:', error);
    }
  },
  
  fetchHistory: async () => {
    try {
      const dtos = await pomodoroApi.getHistory(20);
      set({ history: dtos.map(sessionDtoToModel) });
    } catch (error) {
      console.error('Error fetching history:', error);
    }
  },
  
  startSession: async (type: SessionType, taskDescription?: string) => {
    try {
      set({ isLoading: true, error: null });
      
      const { settings } = get();
      let durationMinutes: number;
      
      switch (type) {
        case 'work':
          durationMinutes = settings.workDurationMinutes;
          break;
        case 'short_break':
          durationMinutes = settings.shortBreakMinutes;
          break;
        case 'long_break':
          durationMinutes = settings.longBreakMinutes;
          break;
      }
      
      const dto = await pomodoroApi.startSession({
        session_type: type,
        task_description: taskDescription,
        duration_minutes: durationMinutes,
      });
      
      const session = sessionDtoToModel(dto);
      const remainingSeconds = durationMinutes * 60;
      
      set({ 
        activeSession: session,
        remainingSeconds,
      });
      
      get().startTimer();
    } catch (error) {
      console.error('Error starting session:', error);
      set({ error: 'Failed to start session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  pauseSession: async () => {
    try {
      set({ isLoading: true, error: null });
      get().stopTimer();
      
      const dto = await pomodoroApi.pauseSession();
      set({ 
        activeSession: sessionDtoToModel(dto),
      });
    } catch (error) {
      console.error('Error pausing session:', error);
      set({ error: 'Failed to pause session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  resumeSession: async () => {
    try {
      set({ isLoading: true, error: null });
      
      const dto = await pomodoroApi.resumeSession();
      const session = sessionDtoToModel(dto);
      const remainingSeconds = get().calculateRemainingSeconds(session);
      
      set({ 
        activeSession: session,
        remainingSeconds,
      });
      
      get().startTimer();
    } catch (error) {
      console.error('Error resuming session:', error);
      set({ error: 'Failed to resume session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  completeSession: async () => {
    try {
      set({ isLoading: true, error: null });
      get().stopTimer();
      
      const { activeSession, completedWorkSessions } = get();
      
      const dto = await pomodoroApi.completeSession();
      
      // Increment work session counter if we completed a work session
      if (activeSession?.sessionType === 'work') {
        set({ completedWorkSessions: completedWorkSessions + 1 });
      }
      
      set({ 
        activeSession: sessionDtoToModel(dto),
        remainingSeconds: 0,
      });
      
      // Refresh stats
      get().fetchTodayStats();
    } catch (error) {
      console.error('Error completing session:', error);
      set({ error: 'Failed to complete session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  cancelSession: async () => {
    try {
      set({ isLoading: true, error: null });
      get().stopTimer();
      
      await pomodoroApi.cancelSession();
      
      set({ 
        activeSession: null,
        remainingSeconds: 0,
      });
    } catch (error) {
      console.error('Error cancelling session:', error);
      set({ error: 'Failed to cancel session' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateSettings: async (newSettings: Partial<PomodoroSettings>) => {
    try {
      set({ isLoading: true, error: null });
      
      const dto = await pomodoroApi.updateSettings({
        work_duration_minutes: newSettings.workDurationMinutes,
        short_break_minutes: newSettings.shortBreakMinutes,
        long_break_minutes: newSettings.longBreakMinutes,
        sessions_until_long_break: newSettings.sessionsUntilLongBreak,
        auto_start_breaks: newSettings.autoStartBreaks,
        auto_start_work: newSettings.autoStartWork,
      });
      
      set({ settings: settingsDtoToModel(dto) });
    } catch (error) {
      console.error('Error updating settings:', error);
      set({ error: 'Failed to update settings' });
    } finally {
      set({ isLoading: false });
    }
  },
}));

// =============================================================================
// Utility Functions
// =============================================================================

export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

export function getSessionTypeLabel(type: SessionType): string {
  switch (type) {
    case 'work':
      return 'Work';
    case 'short_break':
      return 'Short Break';
    case 'long_break':
      return 'Long Break';
  }
}

export function getSessionTypeColor(type: SessionType): string {
  switch (type) {
    case 'work':
      return 'text-red-500';
    case 'short_break':
      return 'text-green-500';
    case 'long_break':
      return 'text-blue-500';
  }
}
