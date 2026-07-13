import api from './client';

export type EventType = 'event' | 'task' | 'reminder' | 'birthday' | 'holiday';

export interface ReminderDto {
  id: string;
  event_id: string;
  remind_at: string;
  message?: string;
  is_sent: boolean;
  created_at: string;
}

export interface CalendarEventDto {
  id: string;
  title: string;
  description?: string;
  start_time: string;
  end_time?: string;
  all_day: boolean;
  location?: string;
  color?: string;
  event_type: EventType;
  recurrence_rule?: string;
  reminders: ReminderDto[];
  created_at: string;
  updated_at: string;
}

export interface CreateEventRequest {
  title: string;
  description?: string;
  start_time: string;
  end_time?: string;
  all_day?: boolean;
  location?: string;
  color?: string;
  event_type?: EventType;
  recurrence_rule?: string;
}

export interface UpdateEventRequest {
  title?: string;
  description?: string;
  start_time?: string;
  end_time?: string;
  all_day?: boolean;
  location?: string;
  color?: string;
  event_type?: EventType;
  recurrence_rule?: string;
}

export interface CreateReminderRequest {
  remind_at: string;
  message?: string;
}

export const calendarApi = {
  listEvents: async (from?: string, to?: string): Promise<CalendarEventDto[]> => {
    const response = await api.get<{ events: CalendarEventDto[]; total: number }>('/calendar/events', {
      params: { from, to }
    });
    return response.data.events || [];
  },

  getTodayEvents: async (): Promise<CalendarEventDto[]> => {
    const response = await api.get<{ events: CalendarEventDto[]; total: number } | CalendarEventDto[]>('/calendar/events/today');
    return Array.isArray(response.data) ? response.data : (response.data.events || []);
  },

  getEvent: async (id: string): Promise<CalendarEventDto> => {
    const response = await api.get<CalendarEventDto>(`/calendar/events/${id}`);
    return response.data;
  },

  createEvent: async (request: CreateEventRequest): Promise<CalendarEventDto> => {
    const response = await api.post<CalendarEventDto>('/calendar/events', request);
    return response.data;
  },

  updateEvent: async (id: string, request: UpdateEventRequest): Promise<CalendarEventDto> => {
    const response = await api.patch<CalendarEventDto>(`/calendar/events/${id}`, request);
    return response.data;
  },

  deleteEvent: async (id: string): Promise<void> => {
    await api.delete(`/calendar/events/${id}`);
  },

  getReminders: async (): Promise<ReminderDto[]> => {
    const response = await api.get<{ reminders: ReminderDto[] } | ReminderDto[]>('/calendar/reminders');
    return Array.isArray(response.data) ? response.data : (response.data.reminders || []);
  },

  addReminder: async (eventId: string, request: CreateReminderRequest): Promise<ReminderDto> => {
    const response = await api.post<ReminderDto>(`/calendar/events/${eventId}/reminders`, request);
    return response.data;
  },

  deleteReminder: async (eventId: string, reminderId: string): Promise<void> => {
    await api.delete(`/calendar/events/${eventId}/reminders/${reminderId}`);
  },
};
