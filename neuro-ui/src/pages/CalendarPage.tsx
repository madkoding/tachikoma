import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  useCalendarStore,
  CalendarEvent,
  EventType,
} from '../stores/calendarStore';
import TypewriterText from '../components/common/TypewriterText';

// Types
type CalendarView = 'month' | 'week' | 'day' | 'agenda';
type WeekStart = 'sunday' | 'monday';

// =============================================================================
// Settings Modal Component
// =============================================================================

function SettingsModal({
  weekStart,
  onWeekStartChange,
  onClose,
}: {
  weekStart: WeekStart;
  onWeekStartChange: (start: WeekStart) => void;
  onClose: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-sm shadow-2xl shadow-cyber-cyan/10">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between">
          <h2 className="text-lg font-cyber font-semibold text-cyber-cyan">
            {t('calendar.settings', 'Settings')}
          </h2>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <div className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">
              {t('calendar.weekStartsOn', 'Week starts on')}
            </label>
            <div className="flex gap-2">
              <button
                type="button"
                onClick={() => onWeekStartChange('monday')}
                className={`flex-1 px-4 py-2.5 rounded-lg text-sm font-mono transition-all
                  ${weekStart === 'monday'
                    ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50'
                    : 'bg-cyber-bg text-cyber-cyan/50 border border-cyber-cyan/20 hover:border-cyber-cyan/40'}`}
              >
                {t('calendar.monday', 'Monday')}
              </button>
              <button
                type="button"
                onClick={() => onWeekStartChange('sunday')}
                className={`flex-1 px-4 py-2.5 rounded-lg text-sm font-mono transition-all
                  ${weekStart === 'sunday'
                    ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50'
                    : 'bg-cyber-bg text-cyber-cyan/50 border border-cyber-cyan/20 hover:border-cyber-cyan/40'}`}
              >
                {t('calendar.sunday', 'Sunday')}
              </button>
            </div>
          </div>
        </div>

        <div className="p-4 border-t border-cyber-cyan/20">
          <button onClick={onClose} className="w-full cyber-button">
            {t('common.done', 'Done')}
          </button>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Event Modal Component (Cyberpunk Style)
// =============================================================================

function EventModal({
  event,
  selectedDate,
  onClose,
  onSave,
  onDelete,
}: {
  event: CalendarEvent | null;
  selectedDate: Date | null;
  onClose: () => void;
  onSave: (data: {
    title: string;
    description?: string;
    start_time: string;
    end_time?: string;
    all_day?: boolean;
    location?: string;
    color?: string;
    event_type?: EventType;
  }) => void;
  onDelete?: () => void;
}) {
  const { t } = useTranslation();
  
  const getDefaultDateTime = () => {
    if (event?.startTime) {
      return new Date(event.startTime.getTime() - event.startTime.getTimezoneOffset() * 60000)
        .toISOString()
        .slice(0, 16);
    }
    if (selectedDate) {
      const date = new Date(selectedDate);
      date.setHours(9, 0, 0, 0);
      return new Date(date.getTime() - date.getTimezoneOffset() * 60000)
        .toISOString()
        .slice(0, 16);
    }
    return new Date().toISOString().slice(0, 16);
  };

  const [title, setTitle] = useState(event?.title || '');
  const [description, setDescription] = useState(event?.description || '');
  const [startTime, setStartTime] = useState(getDefaultDateTime());
  const [endTime, setEndTime] = useState(
    event?.endTime
      ? new Date(event.endTime.getTime() - event.endTime.getTimezoneOffset() * 60000)
          .toISOString()
          .slice(0, 16)
      : ''
  );
  const [allDay, setAllDay] = useState(event?.allDay || false);
  const [location, setLocation] = useState(event?.location || '');
  const [color, setColor] = useState(event?.color || '#00d4ff');
  const [eventType, setEventType] = useState<EventType>(event?.eventType || 'event');

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    onSave({
      title: title.trim(),
      description: description.trim() || undefined,
      start_time: new Date(startTime).toISOString(),
      end_time: endTime ? new Date(endTime).toISOString() : undefined,
      all_day: allDay,
      location: location.trim() || undefined,
      color,
      event_type: eventType,
    });
  };

  const EVENT_TYPES: { value: EventType; label: string; icon: string }[] = [
    { value: 'event', label: t('calendar.type.event', 'Event'), icon: '📅' },
    { value: 'task', label: t('calendar.type.task', 'Task'), icon: '✓' },
    { value: 'reminder', label: t('calendar.type.reminder', 'Reminder'), icon: '🔔' },
    { value: 'birthday', label: 'Birthday', icon: '🎂' },
    { value: 'holiday', label: 'Holiday', icon: '🎉' },
  ];

  const COLORS = ['#00d4ff', '#00ff9f', '#ff00ff', '#ff6b00', '#ffff00', '#ef4444', '#a855f7', '#ec4899', '#3b82f6', '#64748b'];

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-md shadow-2xl shadow-cyber-cyan/10 max-h-[90vh] overflow-auto">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between sticky top-0 bg-cyber-surface">
          <h2 className="text-lg font-cyber font-semibold text-cyber-cyan">
            {event ? t('calendar.editEvent', 'Edit Event') : t('calendar.newEvent', 'New Event')}
          </h2>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">
              {t('calendar.eventTitle', 'Title')} *
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none"
              placeholder={t('calendar.eventTitlePlaceholder', 'Enter event title...')}
              autoFocus
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">
              {t('calendar.eventType', 'Type')}
            </label>
            <div className="flex flex-wrap gap-2">
              {EVENT_TYPES.map((type) => (
                <button
                  key={type.value}
                  type="button"
                  onClick={() => setEventType(type.value)}
                  className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-all flex items-center gap-1.5
                    ${eventType === type.value
                      ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50'
                      : 'bg-cyber-bg text-cyber-cyan/50 border border-cyber-cyan/20 hover:border-cyber-cyan/40'}`}
                >
                  <span>{type.icon}</span>
                  <span className="hidden sm:inline">{type.label}</span>
                </button>
              ))}
            </div>
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">
                {t('calendar.startTime', 'Start')}
              </label>
              <input
                type="datetime-local"
                value={startTime}
                onChange={(e) => setStartTime(e.target.value)}
                className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm focus:border-cyber-cyan focus:outline-none"
              />
            </div>
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">
                {t('calendar.endTime', 'End')}
              </label>
              <input
                type="datetime-local"
                value={endTime}
                onChange={(e) => setEndTime(e.target.value)}
                className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm focus:border-cyber-cyan focus:outline-none"
              />
            </div>
          </div>

          <label className="flex items-center gap-3 cursor-pointer">
            <div className={`w-10 h-5 rounded-full transition-colors relative ${allDay ? 'bg-cyber-cyan' : 'bg-cyber-cyan/20'}`}>
              <div className={`absolute top-0.5 w-4 h-4 rounded-full bg-cyber-bg transition-transform ${allDay ? 'translate-x-5' : 'translate-x-0.5'}`} />
            </div>
            <span className="text-sm font-mono text-cyber-cyan/70">{t('calendar.allDay', 'All Day')}</span>
          </label>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">
              {t('calendar.location', 'Location')}
            </label>
            <input
              type="text"
              value={location}
              onChange={(e) => setLocation(e.target.value)}
              className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none"
              placeholder={t('calendar.locationPlaceholder', 'Add location...')}
            />
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">
              {t('calendar.color', 'Color')}
            </label>
            <div className="flex flex-wrap gap-2">
              {COLORS.map((c) => (
                <button
                  key={c}
                  type="button"
                  onClick={() => setColor(c)}
                  className={`w-7 h-7 rounded-lg transition-all ${color === c ? 'ring-2 ring-white ring-offset-2 ring-offset-cyber-surface scale-110' : 'hover:scale-110'}`}
                  style={{ backgroundColor: c }}
                />
              ))}
            </div>
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">
              {t('calendar.eventDescription', 'Description')}
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              rows={2}
              className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm resize-none focus:border-cyber-cyan focus:outline-none"
              placeholder={t('calendar.eventDescriptionPlaceholder', 'Optional description...')}
            />
          </div>

          <div className="flex flex-col sm:flex-row justify-between gap-2 pt-2">
            {event && onDelete && (
              <button type="button" onClick={onDelete} className="px-4 py-2 text-red-400 hover:bg-red-500/10 rounded-lg transition-colors font-mono text-sm">
                🗑️ {t('common.delete', 'Delete')}
              </button>
            )}
            <div className="flex gap-2 ml-auto">
              <button type="button" onClick={onClose} className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">
                {t('common.cancel', 'Cancel')}
              </button>
              <button type="submit" disabled={!title.trim()} className="cyber-button disabled:opacity-50">
                {t('common.save', 'Save')}
              </button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}

// =============================================================================
// Event Item Component
// =============================================================================

function EventItem({ event, compact = false, onClick }: { event: CalendarEvent; compact?: boolean; onClick: () => void }) {
  const typeIcons: Record<EventType, string> = { event: '📅', meeting: '👥', task: '✓', reminder: '🔔', birthday: '��', holiday: '🎉' };

  if (compact) {
    return (
      <button onClick={(e) => { e.stopPropagation(); onClick(); }} className="w-full text-left px-1.5 py-0.5 rounded text-xs truncate font-mono transition-all hover:scale-105" style={{ backgroundColor: `${event.color}30`, color: event.color }}>
        {typeIcons[event.eventType]} {event.title}
      </button>
    );
  }

  return (
    <button onClick={onClick} className="w-full text-left p-2 sm:p-3 rounded-lg border transition-all hover:scale-[1.02] group" style={{ backgroundColor: `${event.color}15`, borderColor: `${event.color}40` }}>
      <div className="flex items-start gap-2">
        <span className="text-lg">{typeIcons[event.eventType]}</span>
        <div className="flex-1 min-w-0">
          <h4 className="font-mono font-medium truncate text-sm" style={{ color: event.color }}>{event.title}</h4>
          {!event.allDay && (
            <p className="led-event-time mt-0.5">
              {event.startTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
              {event.endTime && ` - ${event.endTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`}
            </p>
          )}
          {event.location && <p className="text-xs font-mono text-cyber-cyan/40 mt-0.5 truncate">📍 {event.location}</p>}
        </div>
      </div>
    </button>
  );
}

// =============================================================================
// Calendar Grid Component
// =============================================================================

function CalendarGrid({ currentDate, events, weekStart, onSelectDate, onSelectEvent }: { currentDate: Date; events: CalendarEvent[]; weekStart: WeekStart; onSelectDate: (date: Date) => void; onSelectEvent: (event: CalendarEvent) => void }) {
  const today = new Date(); today.setHours(0, 0, 0, 0);
  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();
  const firstDayOfMonth = new Date(year, month, 1);
  const lastDayOfMonth = new Date(year, month + 1, 0);
  
  // Adjust start day based on week start preference
  let startDay = firstDayOfMonth.getDay();
  if (weekStart === 'monday') {
    startDay = startDay === 0 ? 6 : startDay - 1;
  }
  const daysInMonth = lastDayOfMonth.getDate();

  const days: (Date | null)[] = [];
  for (let i = 0; i < startDay; i++) days.push(null);
  for (let day = 1; day <= daysInMonth; day++) days.push(new Date(year, month, day));

  const getEventsForDate = (date: Date) => events.filter((event) => {
    const eventDate = new Date(event.startTime);
    return eventDate.getFullYear() === date.getFullYear() && eventDate.getMonth() === date.getMonth() && eventDate.getDate() === date.getDate();
  });

  const WEEKDAYS_SUNDAY = ['Dom', 'Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb'];
  const WEEKDAYS_MONDAY = ['Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb', 'Dom'];
  const WEEKDAYS = weekStart === 'monday' ? WEEKDAYS_MONDAY : WEEKDAYS_SUNDAY;

  // Check if a date is weekend based on week start preference
  const isWeekendDay = (date: Date) => {
    const day = date.getDay();
    return day === 0 || day === 6;
  };

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="grid grid-cols-7 border-b border-cyber-cyan/20 bg-cyber-surface/30 backdrop-blur-sm">
        {WEEKDAYS.map((day) => (
          <div key={day} className="p-1 sm:p-2 text-center led-weekday uppercase tracking-wider">{day}</div>
        ))}
      </div>

      <div className="flex-1 grid grid-cols-7 auto-rows-fr overflow-hidden">
        {days.map((date, index) => {
          if (!date) return <div key={index} className="border-b border-r border-cyber-cyan/15 bg-cyber-bg/20" />;
          const dayEvents = getEventsForDate(date);
          const isToday = date.getTime() === today.getTime();
          const isWeekend = isWeekendDay(date);

          return (
            <button key={index} onClick={() => onSelectDate(date)} className={`border-b border-r border-cyber-cyan/15 p-1 sm:p-2 text-left transition-colors hover:bg-cyber-cyan/15 flex flex-col overflow-hidden ${isWeekend ? 'bg-cyber-surface/20' : 'bg-cyber-bg/10'}`}>
              <span className={`text-xs sm:text-sm w-6 h-6 sm:w-8 sm:h-8 flex items-center justify-center mb-1 ${isToday ? 'led-calendar-day-today' : 'led-calendar-day'}`}>{date.getDate().toString().padStart(2, '0')}</span>
              <div className="flex-1 space-y-0.5 overflow-hidden">
                {dayEvents.slice(0, 2).map((event) => <EventItem key={event.id} event={event} compact onClick={() => onSelectEvent(event)} />)}
                {dayEvents.length > 2 && <span className="text-xs font-mono text-cyber-cyan/50 px-1">+{dayEvents.length - 2}</span>}
              </div>
            </button>
          );
        })}
      </div>
    </div>
  );
}

// =============================================================================
// Week View Component
// =============================================================================

function WeekView({ currentDate, events, weekStart, onSelectDate, onSelectEvent }: { currentDate: Date; events: CalendarEvent[]; weekStart: WeekStart; onSelectDate: (date: Date) => void; onSelectEvent: (event: CalendarEvent) => void }) {
  const { t } = useTranslation();
  const today = new Date(); today.setHours(0, 0, 0, 0);
  
  // Get start of current week
  const getWeekStart = (date: Date) => {
    const d = new Date(date);
    const day = d.getDay();
    const diff = weekStart === 'monday' ? (day === 0 ? -6 : 1 - day) : -day;
    d.setDate(d.getDate() + diff);
    d.setHours(0, 0, 0, 0);
    return d;
  };

  const weekStartDate = getWeekStart(currentDate);
  const weekDays: Date[] = [];
  for (let i = 0; i < 7; i++) {
    const d = new Date(weekStartDate);
    d.setDate(weekStartDate.getDate() + i);
    weekDays.push(d);
  }

  const HOURS = Array.from({ length: 24 }, (_, i) => i);

  const getEventsForDate = (date: Date) => events.filter((event) => {
    const eventDate = new Date(event.startTime);
    return eventDate.getFullYear() === date.getFullYear() && eventDate.getMonth() === date.getMonth() && eventDate.getDate() === date.getDate();
  });

  const WEEKDAYS_SUNDAY = ['Dom', 'Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb'];
  const WEEKDAYS_MONDAY = ['Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb', 'Dom'];
  const WEEKDAY_NAMES = weekStart === 'monday' ? WEEKDAYS_MONDAY : WEEKDAYS_SUNDAY;

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header with days */}
      <div className="grid grid-cols-8 border-b border-cyber-cyan/20 bg-cyber-surface/30 backdrop-blur-sm">
        <div className="p-2 text-center led-weekday" /> {/* Empty cell for time column */}
        {weekDays.map((date, index) => {
          const isToday = date.getTime() === today.getTime();
          return (
            <button
              key={index}
              onClick={() => onSelectDate(date)}
              className={`p-2 text-center border-l border-cyber-cyan/15 hover:bg-cyber-cyan/15 transition-colors ${isToday ? 'bg-cyber-cyan/10' : ''}`}
            >
              <span className="led-weekday block">{WEEKDAY_NAMES[index]}</span>
              <span className={`text-sm mt-1 inline-block ${isToday ? 'led-calendar-day-today' : 'led-calendar-day'}`}>
                {date.getDate().toString().padStart(2, '0')}
              </span>
            </button>
          );
        })}
      </div>

      {/* Time grid */}
      <div className="flex-1 overflow-auto">
        <div className="grid grid-cols-8 min-h-[1200px]">
          {/* Time column */}
          <div className="border-r border-cyber-cyan/20 bg-cyber-surface/20 backdrop-blur-sm">
            {HOURS.map((hour) => (
              <div key={hour} className="h-12 border-b border-cyber-cyan/15 pr-2 text-right">
                <span className="led-event-time">{hour.toString().padStart(2, '0')}:00</span>
              </div>
            ))}
          </div>

          {/* Day columns */}
          {weekDays.map((date, dayIndex) => {
            const dayEvents = getEventsForDate(date);
            return (
              <div key={dayIndex} className="relative border-r border-cyber-cyan/15 bg-cyber-bg/10">
                {HOURS.map((hour) => (
                  <div
                    key={hour}
                    onClick={() => {
                      const newDate = new Date(date);
                      newDate.setHours(hour, 0, 0, 0);
                      onSelectDate(newDate);
                    }}
                    className="h-12 border-b border-cyber-cyan/15 hover:bg-cyber-cyan/10 cursor-pointer"
                  />
                ))}
                {/* Events overlay */}
                {dayEvents.filter(e => !e.allDay).map((event) => {
                  const startHour = event.startTime.getHours() + event.startTime.getMinutes() / 60;
                  const endHour = event.endTime ? event.endTime.getHours() + event.endTime.getMinutes() / 60 : startHour + 1;
                  const top = startHour * 48;
                  const height = Math.max((endHour - startHour) * 48, 24);
                  return (
                    <button
                      key={event.id}
                      onClick={() => onSelectEvent(event)}
                      className="absolute left-0.5 right-0.5 rounded px-1 py-0.5 text-xs font-mono truncate hover:z-10 hover:scale-105 transition-transform"
                      style={{ top: `${top}px`, height: `${height}px`, backgroundColor: `${event.color}30`, borderLeft: `3px solid ${event.color}`, color: event.color }}
                    >
                      {event.title}
                    </button>
                  );
                })}
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Day View Component
// =============================================================================

function DayView({ currentDate, events, onSelectEvent, onNewEvent }: { currentDate: Date; events: CalendarEvent[]; onSelectEvent: (event: CalendarEvent) => void; onNewEvent: () => void }) {
  const { t } = useTranslation();
  const today = new Date(); today.setHours(0, 0, 0, 0);
  const isToday = currentDate.toDateString() === today.toDateString();

  const HOURS = Array.from({ length: 24 }, (_, i) => i);

  const dayEvents = events.filter((event) => {
    const eventDate = new Date(event.startTime);
    return eventDate.getFullYear() === currentDate.getFullYear() && eventDate.getMonth() === currentDate.getMonth() && eventDate.getDate() === currentDate.getDate();
  });

  const allDayEvents = dayEvents.filter(e => e.allDay);
  const timedEvents = dayEvents.filter(e => !e.allDay);

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      {/* Header */}
      <div className="p-4 border-b border-cyber-cyan/20 text-center bg-cyber-surface/30 backdrop-blur-sm">
        <span className="led-weekday block mb-2">{currentDate.toLocaleDateString(undefined, { weekday: 'long' }).toUpperCase()}</span>
        <span className={`text-2xl inline-block ${isToday ? 'led-calendar-day-today' : 'led-calendar-day'}`} style={{ padding: '8px 16px' }}>
          {currentDate.getDate().toString().padStart(2, '0')}
        </span>
      </div>

      {/* All day events */}
      {allDayEvents.length > 0 && (
        <div className="p-2 border-b border-cyber-cyan/20 space-y-1 bg-cyber-surface/20">
          <span className="text-xs font-mono text-cyber-cyan/50 uppercase">{t('calendar.allDay', 'All Day')}</span>
          {allDayEvents.map((event) => (
            <button
              key={event.id}
              onClick={() => onSelectEvent(event)}
              className="w-full text-left px-2 py-1 rounded text-sm font-mono truncate hover:scale-[1.02] transition-transform"
              style={{ backgroundColor: `${event.color}30`, color: event.color }}
            >
              {event.title}
            </button>
          ))}
        </div>
      )}

      {/* Time grid */}
      <div className="flex-1 overflow-auto">
        <div className="flex min-h-[1200px]">
          {/* Time column */}
          <div className="w-16 flex-shrink-0 border-r border-cyber-cyan/20 bg-cyber-surface/20 backdrop-blur-sm">
            {HOURS.map((hour) => (
              <div key={hour} className="h-12 border-b border-cyber-cyan/15 pr-2 text-right flex items-start justify-end">
                <span className="led-event-time">{hour.toString().padStart(2, '0')}:00</span>
              </div>
            ))}
          </div>

          {/* Day column */}
          <div className="flex-1 relative bg-cyber-bg/10">
            {HOURS.map((hour) => (
              <div
                key={hour}
                onClick={() => {
                  const newDate = new Date(currentDate);
                  newDate.setHours(hour, 0, 0, 0);
                  onNewEvent();
                }}
                className="h-12 border-b border-cyber-cyan/10 hover:bg-cyber-cyan/5 cursor-pointer"
              />
            ))}
            {/* Events overlay */}
            {timedEvents.map((event) => {
              const startHour = event.startTime.getHours() + event.startTime.getMinutes() / 60;
              const endHour = event.endTime ? event.endTime.getHours() + event.endTime.getMinutes() / 60 : startHour + 1;
              const top = startHour * 48;
              const height = Math.max((endHour - startHour) * 48, 24);
              return (
                <button
                  key={event.id}
                  onClick={() => onSelectEvent(event)}
                  className="absolute left-1 right-1 rounded px-2 py-1 text-sm font-mono hover:z-10 hover:scale-[1.02] transition-transform"
                  style={{ top: `${top}px`, height: `${height}px`, backgroundColor: `${event.color}30`, borderLeft: `4px solid ${event.color}`, color: event.color }}
                >
                  <div className="font-medium truncate">{event.title}</div>
                  <div className="led-event-time">
                    {event.startTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                    {event.endTime && ` - ${event.endTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`}
                  </div>
                </button>
              );
            })}
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Agenda View Component
// =============================================================================

function AgendaView({ events, onSelectEvent, onNewEvent }: { events: CalendarEvent[]; onSelectEvent: (event: CalendarEvent) => void; onNewEvent: () => void }) {
  const { t } = useTranslation();
  const today = new Date(); today.setHours(0, 0, 0, 0);

  // Get events for the next 30 days
  const upcomingEvents = events
    .filter((event) => new Date(event.startTime) >= today)
    .sort((a, b) => a.startTime.getTime() - b.startTime.getTime());

  // Group events by date
  const groupedEvents: { date: Date; events: CalendarEvent[] }[] = [];
  upcomingEvents.forEach((event) => {
    const eventDate = new Date(event.startTime);
    eventDate.setHours(0, 0, 0, 0);
    const existing = groupedEvents.find((g) => g.date.getTime() === eventDate.getTime());
    if (existing) {
      existing.events.push(event);
    } else {
      groupedEvents.push({ date: eventDate, events: [event] });
    }
  });

  const formatLedDate = (date: Date) => {
    const day = date.getDate().toString().padStart(2, '0');
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    return `${day}.${month}`;
  };

  const isToday = (date: Date) => date.getTime() === today.getTime();
  const isTomorrow = (date: Date) => {
    const tomorrow = new Date(today);
    tomorrow.setDate(tomorrow.getDate() + 1);
    return date.getTime() === tomorrow.getTime();
  };

  return (
    <div className="flex-1 overflow-auto p-4">
      {groupedEvents.length === 0 ? (
        <div className="text-center py-16">
          <p className="text-cyber-cyan/50 font-mono text-lg mb-4">{t('calendar.noUpcomingEvents', 'No upcoming events')}</p>
          <button onClick={onNewEvent} className="cyber-button">+ {t('calendar.newEvent', 'New Event')}</button>
        </div>
      ) : (
        <div className="space-y-6 max-w-2xl mx-auto">
          {groupedEvents.map(({ date, events: dateEvents }) => (
            <div key={date.getTime()}>
              {/* Date header */}
              <div className="flex items-center gap-3 mb-3">
                <span className={`${isToday(date) ? 'led-calendar-day-today' : 'led-calendar-day'}`} style={{ padding: '4px 8px' }}>
                  {formatLedDate(date)}
                </span>
                <span className="text-sm font-mono text-cyber-cyan/70">
                  {isToday(date) ? t('calendar.today', 'Today') : isTomorrow(date) ? t('calendar.tomorrow', 'Tomorrow') : date.toLocaleDateString(undefined, { weekday: 'long' })}
                </span>
                <div className="flex-1 h-px bg-cyber-cyan/20" />
              </div>

              {/* Events for this date */}
              <div className="space-y-2 pl-4">
                {dateEvents.map((event) => (
                  <button
                    key={event.id}
                    onClick={() => onSelectEvent(event)}
                    className="w-full text-left p-3 rounded-lg border transition-all hover:scale-[1.01] group"
                    style={{ backgroundColor: `${event.color}15`, borderColor: `${event.color}40` }}
                  >
                    <div className="flex items-start gap-3">
                      <div className="flex-1 min-w-0">
                        <h4 className="font-mono font-medium text-sm" style={{ color: event.color }}>{event.title}</h4>
                        {!event.allDay && (
                          <p className="led-event-time mt-1">
                            {event.startTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                            {event.endTime && ` - ${event.endTime.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}`}
                          </p>
                        )}
                        {event.allDay && <span className="text-xs font-mono text-cyber-cyan/50">{t('calendar.allDay', 'All Day')}</span>}
                        {event.location && <p className="text-xs font-mono text-cyber-cyan/40 mt-1">📍 {event.location}</p>}
                        {event.description && <p className="text-xs font-mono text-cyber-cyan/50 mt-1 line-clamp-2">{event.description}</p>}
                      </div>
                    </div>
                  </button>
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Sidebar Component
// =============================================================================

function EventsSidebar({ events, selectedDate, onSelectEvent, onNewEvent }: { events: CalendarEvent[]; selectedDate: Date | null; onSelectEvent: (event: CalendarEvent) => void; onNewEvent: () => void }) {
  const { t } = useTranslation();
  const displayDate = selectedDate || new Date();
  const dayEvents = events.filter((event) => {
    const eventDate = new Date(event.startTime);
    return eventDate.getFullYear() === displayDate.getFullYear() && eventDate.getMonth() === displayDate.getMonth() && eventDate.getDate() === displayDate.getDate();
  }).sort((a, b) => a.startTime.getTime() - b.startTime.getTime());

  const isToday = displayDate.toDateString() === new Date().toDateString();

  // Format date for LED display (DD/MM/YYYY)
  const formatLedDate = (date: Date) => {
    const day = date.getDate().toString().padStart(2, '0');
    const month = (date.getMonth() + 1).toString().padStart(2, '0');
    const year = date.getFullYear();
    return `${day}.${month}.${year}`;
  };

  return (
    <div className="w-full h-full md:w-72 lg:w-80 border-l border-cyber-cyan/20 bg-cyber-surface flex flex-col">
      <div className="p-3 sm:p-4 border-b border-cyber-cyan/20">
        <div className="flex items-center justify-between mb-3">
          <div>
            <p className="text-xs font-mono text-cyber-cyan/50 mb-1">{displayDate.toLocaleDateString(undefined, { weekday: 'long' })}</p>
            <div className="led-sidebar-date inline-block">{formatLedDate(displayDate)}</div>
          </div>
          <button onClick={onNewEvent} className="p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" /></svg>
          </button>
        </div>
      </div>
      <div className="flex-1 overflow-auto p-3 sm:p-4 space-y-2">
        {dayEvents.length === 0 ? (
          <div className="text-center py-8">
            <p className="text-cyber-cyan/50 font-mono text-sm">{t('calendar.noEventsToday', 'No events')}</p>
            <button onClick={onNewEvent} className="mt-4 cyber-button text-sm">+ {t('calendar.newEvent', 'New Event')}</button>
          </div>
        ) : dayEvents.map((event) => <EventItem key={event.id} event={event} onClick={() => onSelectEvent(event)} />)}
      </div>
    </div>
  );
}

// =============================================================================
// Main Calendar Page
// =============================================================================

export default function CalendarPage() {
  const { t } = useTranslation();
  const { events, selectedDate, currentMonth, isLoading, error, loadEvents, selectDate, setCurrentMonth, createEvent, updateEvent, deleteEvent, clearError } = useCalendarStore();

  const [showEventModal, setShowEventModal] = useState(false);
  const [editingEvent, setEditingEvent] = useState<CalendarEvent | null>(null);
  const [showSidebar, setShowSidebar] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [calendarView, setCalendarView] = useState<CalendarView>('month');
  const [weekStart, setWeekStart] = useState<WeekStart>(() => {
    const saved = localStorage.getItem('calendar-week-start');
    return (saved === 'sunday' || saved === 'monday') ? saved : 'monday';
  });

  const handleWeekStartChange = (start: WeekStart) => {
    setWeekStart(start);
    localStorage.setItem('calendar-week-start', start);
  };

  useEffect(() => { loadEvents(); }, [loadEvents]);

  const handlePrevMonth = () => { const d = new Date(currentMonth); d.setMonth(d.getMonth() - 1); setCurrentMonth(d); };
  const handleNextMonth = () => { const d = new Date(currentMonth); d.setMonth(d.getMonth() + 1); setCurrentMonth(d); };
  const handlePrevYear = () => { const d = new Date(currentMonth); d.setFullYear(d.getFullYear() - 1); setCurrentMonth(d); };
  const handleNextYear = () => { const d = new Date(currentMonth); d.setFullYear(d.getFullYear() + 1); setCurrentMonth(d); };
  const handleToday = () => { setCurrentMonth(new Date()); selectDate(new Date()); };
  const handleSelectDate = (date: Date) => { selectDate(date); setShowSidebar(true); };
  const handleSelectEvent = (event: CalendarEvent) => { setEditingEvent(event); setShowEventModal(true); };
  const handleNewEvent = () => { setEditingEvent(null); setShowEventModal(true); };

  const handleSaveEvent = async (data: Parameters<typeof createEvent>[0]) => {
    try {
      if (editingEvent) await updateEvent(editingEvent.id, data);
      else await createEvent(data);
      setShowEventModal(false); setEditingEvent(null);
    } catch (e) { console.error('Failed to save event:', e); }
  };

  const handleDeleteEvent = async () => {
    if (editingEvent && window.confirm(t('calendar.confirmDelete', 'Delete this event?'))) {
      try { await deleteEvent(editingEvent.id); setShowEventModal(false); setEditingEvent(null); } catch (e) { console.error('Failed to delete event:', e); }
    }
  };

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      {/* Curved background pattern */}
      <div className="fixed inset-0 pointer-events-none opacity-10">
        <svg className="w-full h-full" xmlns="http://www.w3.org/2000/svg">
          <defs>
            <pattern id="waves" x="0" y="0" width="200" height="200" patternUnits="userSpaceOnUse">
              <path d="M0 50 Q50 0 100 50 T200 50" fill="none" stroke="cyan" strokeWidth="0.5" opacity="0.5"/>
              <path d="M0 100 Q50 50 100 100 T200 100" fill="none" stroke="cyan" strokeWidth="0.5" opacity="0.3"/>
              <path d="M0 150 Q50 100 100 150 T200 150" fill="none" stroke="cyan" strokeWidth="0.5" opacity="0.2"/>
              <circle cx="100" cy="100" r="80" fill="none" stroke="cyan" strokeWidth="0.3" opacity="0.15"/>
              <circle cx="100" cy="100" r="40" fill="none" stroke="cyan" strokeWidth="0.3" opacity="0.1"/>
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#waves)"/>
        </svg>
      </div>

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 sm:gap-4">
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan hidden sm:block"><TypewriterText text={t('nav.calendar', 'Calendar')} speed={30} /></h1>
            <div className="flex items-center gap-2 sm:gap-4">
              {/* Month selector */}
              <div className="flex items-center gap-1">
                <button onClick={handlePrevMonth} className="p-1 sm:p-1.5 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                  <svg className="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" /></svg>
                </button>
                <span className="led-calendar-header min-w-[80px] sm:min-w-[120px] text-center text-sm sm:text-base">{currentMonth.toLocaleDateString(undefined, { month: 'short' }).toUpperCase()}</span>
                <button onClick={handleNextMonth} className="p-1 sm:p-1.5 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                  <svg className="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
                </button>
              </div>
              {/* Year selector */}
              <div className="flex items-center gap-1">
                <button onClick={handlePrevYear} className="p-1 sm:p-1.5 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                  <svg className="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" /></svg>
                </button>
                <span className="led-calendar-header min-w-[50px] sm:min-w-[70px] text-center text-sm sm:text-base">{currentMonth.getFullYear()}</span>
                <button onClick={handleNextYear} className="p-1 sm:p-1.5 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                  <svg className="w-3 h-3 sm:w-4 sm:h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
                </button>
              </div>
            </div>
          </div>
          <div className="flex items-center gap-2">
            {/* View selector buttons */}
            <div className="hidden sm:flex items-center border border-cyber-cyan/30 rounded-lg overflow-hidden">
              <button
                onClick={() => setCalendarView('month')}
                className={`px-2 py-1.5 text-xs font-mono transition-all ${calendarView === 'month' ? 'bg-cyber-cyan/20 text-cyber-cyan' : 'text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10'}`}
              >
                {t('calendar.month', 'Month')}
              </button>
              <button
                onClick={() => setCalendarView('week')}
                className={`px-2 py-1.5 text-xs font-mono transition-all border-l border-cyber-cyan/30 ${calendarView === 'week' ? 'bg-cyber-cyan/20 text-cyber-cyan' : 'text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10'}`}
              >
                {t('calendar.week', 'Week')}
              </button>
              <button
                onClick={() => setCalendarView('day')}
                className={`px-2 py-1.5 text-xs font-mono transition-all border-l border-cyber-cyan/30 ${calendarView === 'day' ? 'bg-cyber-cyan/20 text-cyber-cyan' : 'text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10'}`}
              >
                {t('calendar.day', 'Day')}
              </button>
              <button
                onClick={() => setCalendarView('agenda')}
                className={`px-2 py-1.5 text-xs font-mono transition-all border-l border-cyber-cyan/30 ${calendarView === 'agenda' ? 'bg-cyber-cyan/20 text-cyber-cyan' : 'text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10'}`}
              >
                {t('calendar.agenda', 'Agenda')}
              </button>
            </div>
            <button onClick={handleToday} className="px-2 sm:px-3 py-1.5 text-xs sm:text-sm font-mono text-cyber-cyan/70 hover:text-cyber-cyan border border-cyber-cyan/30 hover:border-cyber-cyan/50 rounded-lg transition-all">{t('calendar.today', 'Today')}</button>
            {/* Settings button */}
            <button onClick={() => setShowSettings(true)} className="p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all" title={t('calendar.settings', 'Settings')}>
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
              </svg>
            </button>
            <button onClick={() => setShowSidebar(!showSidebar)} className="md:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            <button onClick={handleNewEvent} className="cyber-button text-sm hidden sm:flex">+ {t('calendar.newEvent', 'New Event')}</button>
          </div>
        </div>
      </header>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-red-500/20 border border-red-500/50 text-red-400 rounded-lg flex justify-between items-center font-mono text-sm">
          <span>{error}</span><button onClick={clearError} className="text-red-400 hover:text-red-300">✕</button>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden relative">
        {isLoading && <div className="absolute inset-0 bg-cyber-bg/80 flex items-center justify-center z-20"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" /></div>}
        <div className={`flex-1 flex flex-col ${showSidebar ? 'hidden md:flex' : 'flex'}`}>
          {calendarView === 'month' && <CalendarGrid currentDate={currentMonth} events={events} weekStart={weekStart} onSelectDate={handleSelectDate} onSelectEvent={handleSelectEvent} />}
          {calendarView === 'week' && <WeekView currentDate={selectedDate || currentMonth} events={events} weekStart={weekStart} onSelectDate={handleSelectDate} onSelectEvent={handleSelectEvent} />}
          {calendarView === 'day' && <DayView currentDate={selectedDate || new Date()} events={events} onSelectEvent={handleSelectEvent} onNewEvent={handleNewEvent} />}
          {calendarView === 'agenda' && <AgendaView events={events} onSelectEvent={handleSelectEvent} onNewEvent={handleNewEvent} />}
        </div>
        <div className={`${showSidebar ? 'flex' : 'hidden md:flex'} absolute md:relative inset-0 md:inset-auto z-10`}>
          <button onClick={() => setShowSidebar(false)} className="md:hidden absolute inset-0 bg-black/50" />
          <div className="relative ml-auto h-full">
            <button onClick={() => setShowSidebar(false)} className="md:hidden absolute top-2 right-2 z-10 p-2 text-cyber-cyan/70 hover:text-cyber-cyan">✕</button>
            <EventsSidebar events={events} selectedDate={selectedDate} onSelectEvent={handleSelectEvent} onNewEvent={handleNewEvent} />
          </div>
        </div>
      </div>

      {showEventModal && <EventModal event={editingEvent} selectedDate={selectedDate} onClose={() => { setShowEventModal(false); setEditingEvent(null); }} onSave={handleSaveEvent} onDelete={editingEvent ? handleDeleteEvent : undefined} />}
      {showSettings && <SettingsModal weekStart={weekStart} onWeekStartChange={handleWeekStartChange} onClose={() => setShowSettings(false)} />}
    </div>
  );
}
