import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  useCalendarStore,
  CalendarEvent,
  EventType,
} from '../stores/calendarStore';
import TypewriterText from '../components/common/TypewriterText';

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
            <p className="text-xs font-mono text-cyber-cyan/50 mt-0.5">
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

function CalendarGrid({ currentDate, events, onSelectDate, onSelectEvent }: { currentDate: Date; events: CalendarEvent[]; onSelectDate: (date: Date) => void; onSelectEvent: (event: CalendarEvent) => void }) {
  const today = new Date(); today.setHours(0, 0, 0, 0);
  const year = currentDate.getFullYear();
  const month = currentDate.getMonth();
  const firstDayOfMonth = new Date(year, month, 1);
  const lastDayOfMonth = new Date(year, month + 1, 0);
  const startDay = firstDayOfMonth.getDay();
  const daysInMonth = lastDayOfMonth.getDate();

  const days: (Date | null)[] = [];
  for (let i = 0; i < startDay; i++) days.push(null);
  for (let day = 1; day <= daysInMonth; day++) days.push(new Date(year, month, day));

  const getEventsForDate = (date: Date) => events.filter((event) => {
    const eventDate = new Date(event.startTime);
    return eventDate.getFullYear() === date.getFullYear() && eventDate.getMonth() === date.getMonth() && eventDate.getDate() === date.getDate();
  });

  const WEEKDAYS = ['Dom', 'Lun', 'Mar', 'Mié', 'Jue', 'Vie', 'Sáb'];

  return (
    <div className="flex-1 flex flex-col overflow-hidden">
      <div className="grid grid-cols-7 border-b border-cyber-cyan/20">
        {WEEKDAYS.map((day) => (
          <div key={day} className="p-1 sm:p-2 text-center text-xs font-mono text-cyber-cyan/50 uppercase tracking-wider">{day}</div>
        ))}
      </div>

      <div className="flex-1 grid grid-cols-7 auto-rows-fr overflow-hidden">
        {days.map((date, index) => {
          if (!date) return <div key={index} className="border-b border-r border-cyber-cyan/10 bg-cyber-bg/50" />;
          const dayEvents = getEventsForDate(date);
          const isToday = date.getTime() === today.getTime();
          const isWeekend = date.getDay() === 0 || date.getDay() === 6;

          return (
            <button key={index} onClick={() => onSelectDate(date)} className={`border-b border-r border-cyber-cyan/10 p-1 sm:p-2 text-left transition-colors hover:bg-cyber-cyan/10 flex flex-col overflow-hidden ${isWeekend ? 'bg-cyber-cyan/5' : 'bg-cyber-surface/50'}`}>
              <span className={`text-xs sm:text-sm font-mono w-5 h-5 sm:w-7 sm:h-7 flex items-center justify-center rounded-full mb-1 ${isToday ? 'bg-cyber-cyan text-cyber-bg font-bold' : 'text-cyber-cyan/70'}`}>{date.getDate()}</span>
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

  return (
    <div className="w-full h-full md:w-72 lg:w-80 border-l border-cyber-cyan/20 bg-cyber-surface flex flex-col">
      <div className="p-3 sm:p-4 border-b border-cyber-cyan/20">
        <div className="flex items-center justify-between mb-2">
          <h2 className="font-cyber font-bold text-cyber-cyan text-sm sm:text-base">{isToday ? t('calendar.today', 'Today') : displayDate.toLocaleDateString()}</h2>
          <button onClick={onNewEvent} className="p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
            <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" /></svg>
          </button>
        </div>
        <p className="text-xs font-mono text-cyber-cyan/50">{displayDate.toLocaleDateString(undefined, { weekday: 'long', year: 'numeric', month: 'long', day: 'numeric' })}</p>
      </div>
      <div className="flex-1 overflow-auto p-3 sm:p-4 space-y-2">
        {dayEvents.length === 0 ? (
          <div className="text-center py-8">
            <span className="text-4xl mb-4 block">📅</span>
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

  useEffect(() => { loadEvents(); }, [loadEvents]);

  const handlePrevMonth = () => { const d = new Date(currentMonth); d.setMonth(d.getMonth() - 1); setCurrentMonth(d); };
  const handleNextMonth = () => { const d = new Date(currentMonth); d.setMonth(d.getMonth() + 1); setCurrentMonth(d); };
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
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 sm:gap-4">
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan hidden sm:block"><TypewriterText text={t('nav.calendar', 'Calendar')} speed={30} /></h1>
            <div className="flex items-center gap-1 sm:gap-2">
              <button onClick={handlePrevMonth} className="p-1.5 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                <svg className="w-4 h-4 sm:w-5 sm:h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" /></svg>
              </button>
              <span className="font-cyber text-cyber-cyan text-sm sm:text-lg min-w-[100px] sm:min-w-[180px] text-center">{currentMonth.toLocaleDateString(undefined, { month: 'long', year: 'numeric' })}</span>
              <button onClick={handleNextMonth} className="p-1.5 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">
                <svg className="w-4 h-4 sm:w-5 sm:h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" /></svg>
              </button>
            </div>
          </div>
          <div className="flex items-center gap-2">
            <button onClick={handleToday} className="px-2 sm:px-3 py-1.5 text-xs sm:text-sm font-mono text-cyber-cyan/70 hover:text-cyber-cyan border border-cyber-cyan/30 hover:border-cyber-cyan/50 rounded-lg transition-all">{t('calendar.today', 'Today')}</button>
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
          <CalendarGrid currentDate={currentMonth} events={events} onSelectDate={handleSelectDate} onSelectEvent={handleSelectEvent} />
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
    </div>
  );
}
