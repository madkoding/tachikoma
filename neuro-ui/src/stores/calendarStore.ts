import { create } from 'zustand';
import {
  calendarApi,
  CalendarEventDto,
  ReminderDto,
  EventType,
  CreateEventRequest,
  UpdateEventRequest,
  CreateReminderRequest,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export type { EventType };

export interface Reminder {
  id: string;
  eventId: string;
  remindAt: Date;
  message?: string;
  isSent: boolean;
  createdAt: Date;
}

export interface CalendarEvent {
  id: string;
  title: string;
  description?: string;
  startTime: Date;
  endTime?: Date;
  allDay: boolean;
  location?: string;
  color?: string;
  eventType: EventType;
  recurrenceRule?: string;
  reminders: Reminder[];
  createdAt: Date;
  updatedAt: Date;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function reminderDtoToModel(dto: ReminderDto): Reminder {
  return {
    id: dto.id,
    eventId: dto.event_id,
    remindAt: new Date(dto.remind_at),
    message: dto.message,
    isSent: dto.is_sent,
    createdAt: new Date(dto.created_at),
  };
}

function eventDtoToModel(dto: CalendarEventDto): CalendarEvent {
  return {
    id: dto.id,
    title: dto.title,
    description: dto.description,
    startTime: new Date(dto.start_time),
    endTime: dto.end_time ? new Date(dto.end_time) : undefined,
    allDay: dto.all_day,
    location: dto.location,
    color: dto.color,
    eventType: dto.event_type,
    recurrenceRule: dto.recurrence_rule,
    reminders: dto.reminders.map(reminderDtoToModel),
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

// =============================================================================
// Store Interface
// =============================================================================

interface CalendarState {
  // Data
  events: CalendarEvent[];
  selectedEvent: CalendarEvent | null;
  upcomingReminders: Reminder[];
  
  // UI State
  isLoading: boolean;
  error: string | null;
  selectedDate: Date;
  currentMonth: Date;
  viewMode: 'month' | 'week' | 'day';
  
  // Actions
  loadEvents: (from?: Date, to?: Date) => Promise<void>;
  loadTodayEvents: () => Promise<void>;
  loadReminders: () => Promise<void>;
  selectEvent: (event: CalendarEvent | null) => void;
  selectDate: (date: Date) => void;
  setSelectedDate: (date: Date) => void;
  setCurrentMonth: (date: Date) => void;
  setViewMode: (mode: 'month' | 'week' | 'day') => void;
  
  createEvent: (data: CreateEventRequest) => Promise<CalendarEvent>;
  updateEvent: (id: string, data: UpdateEventRequest) => Promise<CalendarEvent>;
  deleteEvent: (id: string) => Promise<void>;
  
  addReminder: (eventId: string, data: CreateReminderRequest) => Promise<Reminder>;
  deleteReminder: (eventId: string, reminderId: string) => Promise<void>;
  
  clearError: () => void;
}

// =============================================================================
// Store Implementation
// =============================================================================

export const useCalendarStore = create<CalendarState>((set, get) => ({
  // Initial state
  events: [],
  selectedEvent: null,
  upcomingReminders: [],
  isLoading: false,
  error: null,
  selectedDate: new Date(),
  currentMonth: new Date(),
  viewMode: 'month',

  // Load events with optional date range
  loadEvents: async (from?: Date, to?: Date) => {
    set({ isLoading: true, error: null });
    try {
      const fromStr = from?.toISOString();
      const toStr = to?.toISOString();
      const dtos = await calendarApi.listEvents(fromStr, toStr);
      const events = dtos.map(eventDtoToModel);
      set({ events, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load events';
      set({ error: message, isLoading: false });
    }
  },

  // Load today's events
  loadTodayEvents: async () => {
    set({ isLoading: true, error: null });
    try {
      const dtos = await calendarApi.getTodayEvents();
      const events = dtos.map(eventDtoToModel);
      set({ events, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load today events';
      set({ error: message, isLoading: false });
    }
  },

  // Load upcoming reminders
  loadReminders: async () => {
    try {
      const dtos = await calendarApi.getReminders();
      const reminders = dtos.map(reminderDtoToModel);
      set({ upcomingReminders: reminders });
    } catch (error) {
      console.error('Failed to load reminders:', error);
    }
  },

  selectEvent: (event) => set({ selectedEvent: event }),
  selectDate: (date) => set({ selectedDate: date }),
  setSelectedDate: (date) => set({ selectedDate: date }),
  setCurrentMonth: (date) => set({ currentMonth: date }),
  setViewMode: (mode) => set({ viewMode: mode }),

  // Create a new event
  createEvent: async (data) => {
    set({ isLoading: true, error: null });
    try {
      const dto = await calendarApi.createEvent(data);
      const event = eventDtoToModel(dto);
      set((state) => ({
        events: [...state.events, event],
        isLoading: false,
      }));
      return event;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create event';
      set({ error: message, isLoading: false });
      throw error;
    }
  },

  // Update an event
  updateEvent: async (id, data) => {
    set({ isLoading: true, error: null });
    try {
      const dto = await calendarApi.updateEvent(id, data);
      const event = eventDtoToModel(dto);
      set((state) => ({
        events: state.events.map((e) => (e.id === id ? event : e)),
        selectedEvent: state.selectedEvent?.id === id ? event : state.selectedEvent,
        isLoading: false,
      }));
      return event;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update event';
      set({ error: message, isLoading: false });
      throw error;
    }
  },

  // Delete an event
  deleteEvent: async (id) => {
    set({ isLoading: true, error: null });
    try {
      await calendarApi.deleteEvent(id);
      set((state) => ({
        events: state.events.filter((e) => e.id !== id),
        selectedEvent: state.selectedEvent?.id === id ? null : state.selectedEvent,
        isLoading: false,
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete event';
      set({ error: message, isLoading: false });
      throw error;
    }
  },

  // Add reminder to event
  addReminder: async (eventId, data) => {
    try {
      const dto = await calendarApi.addReminder(eventId, data);
      const reminder = reminderDtoToModel(dto);
      
      // Update the event's reminders
      set((state) => ({
        events: state.events.map((e) => {
          if (e.id === eventId) {
            return { ...e, reminders: [...e.reminders, reminder] };
          }
          return e;
        }),
        selectedEvent: state.selectedEvent?.id === eventId
          ? { ...state.selectedEvent, reminders: [...state.selectedEvent.reminders, reminder] }
          : state.selectedEvent,
      }));
      
      // Refresh upcoming reminders
      get().loadReminders();
      
      return reminder;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to add reminder';
      set({ error: message });
      throw error;
    }
  },

  // Delete reminder from event
  deleteReminder: async (eventId, reminderId) => {
    try {
      await calendarApi.deleteReminder(eventId, reminderId);
      
      set((state) => ({
        events: state.events.map((e) => {
          if (e.id === eventId) {
            return { ...e, reminders: e.reminders.filter((r) => r.id !== reminderId) };
          }
          return e;
        }),
        selectedEvent: state.selectedEvent?.id === eventId
          ? {
              ...state.selectedEvent,
              reminders: state.selectedEvent.reminders.filter((r) => r.id !== reminderId),
            }
          : state.selectedEvent,
        upcomingReminders: state.upcomingReminders.filter((r) => r.id !== reminderId),
      }));
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete reminder';
      set({ error: message });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));
