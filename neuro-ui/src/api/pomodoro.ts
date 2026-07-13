import axios from 'axios';
import api from './client';

export type PomodoroSessionStatus = 'working' | 'short_break' | 'long_break' | 'paused' | 'completed' | 'cancelled';
export type PomodoroSessionType = 'work' | 'short_break' | 'long_break';

export const isSessionRunning = (status: PomodoroSessionStatus): boolean => {
  return status === 'working' || status === 'short_break' || status === 'long_break';
};

export interface PomodoroSessionDto {
  id: string;
  session_type: PomodoroSessionType;
  status: PomodoroSessionStatus;
  duration_minutes: number;
  elapsed_seconds: number;
  started_at: string;
  paused_at?: string;
  completed_at?: string;
  task_description?: string;
  created_at: string;
  updated_at: string;
}

export interface PomodoroSettingsDto {
  work_duration_minutes: number;
  short_break_minutes: number;
  long_break_minutes: number;
  sessions_until_long_break: number;
  auto_start_breaks: boolean;
  auto_start_work: boolean;
}

export interface PomodoroStatsDto {
  date: string;
  total_sessions: number;
  completed_sessions: number;
  total_work_minutes: number;
  total_break_minutes: number;
}

export interface StartSessionRequest {
  session_type: PomodoroSessionType;
  task_description?: string;
  duration_minutes?: number;
}

export interface UpdateSettingsRequest {
  work_duration_minutes?: number;
  short_break_minutes?: number;
  long_break_minutes?: number;
  sessions_until_long_break?: number;
  auto_start_breaks?: boolean;
  auto_start_work?: boolean;
}

export const pomodoroApi = {
  getActiveSession: async (): Promise<PomodoroSessionDto | null> => {
    try {
      const response = await api.get<PomodoroSessionDto>('/pomodoro/sessions/active');
      return response.data;
    } catch (error) {
      if (axios.isAxiosError(error) && error.response?.status === 404) {
        return null;
      }
      throw error;
    }
  },

  startSession: async (request: StartSessionRequest): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/start', request);
    return response.data;
  },

  pauseSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/pause');
    return response.data;
  },

  resumeSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/resume');
    return response.data;
  },

  completeSession: async (): Promise<PomodoroSessionDto> => {
    const response = await api.post<PomodoroSessionDto>('/pomodoro/sessions/complete');
    return response.data;
  },

  cancelSession: async (): Promise<void> => {
    await api.post('/pomodoro/sessions/cancel');
  },

  getHistory: async (limit = 20): Promise<PomodoroSessionDto[]> => {
    const response = await api.get<PomodoroSessionDto[]>('/pomodoro/sessions/history', {
      params: { limit }
    });
    return response.data;
  },

  getSettings: async (): Promise<PomodoroSettingsDto> => {
    const response = await api.get<PomodoroSettingsDto>('/pomodoro/settings');
    return response.data;
  },

  updateSettings: async (request: UpdateSettingsRequest): Promise<PomodoroSettingsDto> => {
    const response = await api.put<PomodoroSettingsDto>('/pomodoro/settings', request);
    return response.data;
  },

  getDailyStats: async (date?: string): Promise<PomodoroStatsDto> => {
    const response = await api.get<PomodoroStatsDto>('/pomodoro/stats/daily', {
      params: date ? { date } : undefined
    });
    return response.data;
  },

  getWeeklyStats: async (): Promise<PomodoroStatsDto[]> => {
    const response = await api.get<PomodoroStatsDto[]>('/pomodoro/stats/weekly');
    return response.data;
  },
};
