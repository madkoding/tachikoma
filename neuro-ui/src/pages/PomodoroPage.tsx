import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  usePomodoroStore,
  formatTime,
  getSessionTypeLabel,
  SessionType,
} from '../stores/pomodoroStore';
import TypewriterText from '../components/common/TypewriterText';

// =============================================================================
// Timer Circle (Cyberpunk)
// =============================================================================

function TimerCircle({ remainingSeconds, totalSeconds, sessionType }: { remainingSeconds: number; totalSeconds: number; sessionType: SessionType }) {
  const progress = totalSeconds > 0 ? (remainingSeconds / totalSeconds) * 100 : 100;
  const circumference = 2 * Math.PI * 140;
  const strokeDashoffset = circumference - (progress / 100) * circumference;

  const getColor = () => {
    switch (sessionType) {
      case 'work': return { stroke: '#ef4444', glow: '#ef444480' };
      case 'short_break': return { stroke: '#00ff9f', glow: '#00ff9f80' };
      case 'long_break': return { stroke: '#00d4ff', glow: '#00d4ff80' };
    }
  };

  const colors = getColor();

  return (
    <div className="relative w-64 h-64 sm:w-80 sm:h-80 flex items-center justify-center">
      <svg className="absolute w-full h-full -rotate-90" viewBox="0 0 300 300">
        <circle cx="150" cy="150" r="140" fill="none" stroke="currentColor" strokeWidth="4" className="text-cyber-cyan/10" />
        <circle cx="150" cy="150" r="140" fill="none" stroke={colors.stroke} strokeWidth="6" strokeLinecap="round" strokeDasharray={circumference} strokeDashoffset={strokeDashoffset} className="transition-all duration-1000 ease-linear" style={{ filter: `drop-shadow(0 0 12px ${colors.glow})` }} />
      </svg>
      <div className="text-center z-10">
        <div className="text-5xl sm:text-6xl font-mono font-bold text-cyber-cyan" style={{ textShadow: `0 0 20px ${colors.glow}` }}>{formatTime(remainingSeconds)}</div>
        <div className="text-sm sm:text-base font-mono text-cyber-cyan/50 mt-2 uppercase tracking-wider">{getSessionTypeLabel(sessionType)}</div>
      </div>
    </div>
  );
}

// =============================================================================
// Control Buttons (Cyberpunk)
// =============================================================================

function ControlButtons() {
  const { activeSession, isLoading, startSession, pauseSession, resumeSession, completeSession, cancelSession, getNextSessionType } = usePomodoroStore();
  const { t } = useTranslation();

  if (!activeSession || activeSession.status === 'completed' || activeSession.status === 'cancelled') {
    return (
      <div className="flex flex-col items-center gap-4">
        <button onClick={() => startSession('work')} disabled={isLoading} className="px-8 py-4 bg-gradient-to-r from-red-500 to-red-600 hover:from-red-600 hover:to-red-700 text-white rounded-xl font-cyber font-semibold text-lg transition-all disabled:opacity-50 shadow-lg shadow-red-500/30 hover:shadow-red-500/50">🍅 {t('pomodoro.startWork', 'Start Work Session')}</button>
        <div className="flex gap-3">
          <button onClick={() => startSession('short_break')} disabled={isLoading} className="px-4 py-2 bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white rounded-lg font-mono font-medium transition-all disabled:opacity-50 shadow-lg shadow-emerald-500/20">☕ {t('pomodoro.shortBreak', 'Short Break')}</button>
          <button onClick={() => startSession('long_break')} disabled={isLoading} className="px-4 py-2 bg-gradient-to-r from-cyber-cyan to-blue-500 hover:from-blue-500 hover:to-blue-600 text-white rounded-lg font-mono font-medium transition-all disabled:opacity-50 shadow-lg shadow-cyber-cyan/20">🌴 {t('pomodoro.longBreak', 'Long Break')}</button>
        </div>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-4">
      <div className="flex flex-wrap justify-center gap-3">
        {activeSession.status === 'running' ? (
          <button onClick={pauseSession} disabled={isLoading} className="px-6 py-3 bg-gradient-to-r from-amber-500 to-amber-600 hover:from-amber-600 hover:to-amber-700 text-white rounded-xl font-cyber font-semibold transition-all disabled:opacity-50 shadow-lg shadow-amber-500/30">⏸️ {t('pomodoro.pause', 'Pause')}</button>
        ) : (
          <button onClick={resumeSession} disabled={isLoading} className="px-6 py-3 bg-gradient-to-r from-emerald-500 to-emerald-600 hover:from-emerald-600 hover:to-emerald-700 text-white rounded-xl font-cyber font-semibold transition-all disabled:opacity-50 shadow-lg shadow-emerald-500/30">▶️ {t('pomodoro.resume', 'Resume')}</button>
        )}
        <button onClick={completeSession} disabled={isLoading} className="px-6 py-3 bg-gradient-to-r from-green-500 to-green-600 hover:from-green-600 hover:to-green-700 text-white rounded-xl font-cyber font-semibold transition-all disabled:opacity-50 shadow-lg shadow-green-500/30">✅ {t('pomodoro.complete', 'Complete')}</button>
        <button onClick={cancelSession} disabled={isLoading} className="px-6 py-3 bg-cyber-surface border border-cyber-cyan/30 text-cyber-cyan/70 hover:text-cyber-cyan hover:border-cyber-cyan/50 rounded-xl font-cyber font-semibold transition-all disabled:opacity-50">❌ {t('pomodoro.cancel', 'Cancel')}</button>
      </div>
      {activeSession.sessionType === 'work' && (
        <p className="text-sm text-cyber-cyan/40 font-mono">{t('pomodoro.nextBreak', 'Next: {{type}}', { type: getSessionTypeLabel(getNextSessionType()) })}</p>
      )}
    </div>
  );
}

// =============================================================================
// Stats Card (Cyberpunk)
// =============================================================================

function StatsCard() {
  const { todayStats, completedWorkSessions, settings } = usePomodoroStore();
  const { t } = useTranslation();

  const sessionsUntilLongBreak = settings.sessionsUntilLongBreak - (completedWorkSessions % settings.sessionsUntilLongBreak);

  const stats = [
    { label: t('pomodoro.sessionsCompleted', 'Sessions'), value: todayStats?.completedSessions ?? 0, color: '#ef4444' },
    { label: t('pomodoro.workMinutes', 'Work Minutes'), value: todayStats?.totalWorkMinutes ?? 0, color: '#00d4ff' },
    { label: t('pomodoro.breakMinutes', 'Break Minutes'), value: todayStats?.totalBreakMinutes ?? 0, color: '#00ff9f' },
    { label: t('pomodoro.untilLongBreak', 'Until Long Break'), value: sessionsUntilLongBreak, color: '#a855f7' },
  ];

  return (
    <div className="bg-cyber-surface border border-cyber-cyan/20 rounded-xl p-4 sm:p-6">
      <h3 className="text-sm font-cyber font-semibold text-cyber-cyan mb-4 flex items-center gap-2">📊 {t('pomodoro.todayStats', "Today's Stats")}</h3>
      <div className="grid grid-cols-2 sm:grid-cols-4 gap-3 sm:gap-4">
        {stats.map((stat) => (
          <div key={stat.label} className="text-center p-3 bg-cyber-bg rounded-lg border border-cyber-cyan/10">
            <div className="text-2xl sm:text-3xl font-mono font-bold" style={{ color: stat.color, textShadow: `0 0 10px ${stat.color}50` }}>{stat.value}</div>
            <div className="text-xs text-cyber-cyan/50 font-mono mt-1">{stat.label}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// Settings Panel (Cyberpunk)
// =============================================================================

function SettingsPanel() {
  const { settings, updateSettings, isLoading } = usePomodoroStore();
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [localSettings, setLocalSettings] = useState(settings);

  useEffect(() => { setLocalSettings(settings); }, [settings]);

  const handleSave = async () => { await updateSettings(localSettings); setIsOpen(false); };

  if (!isOpen) {
    return <button onClick={() => setIsOpen(true)} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors font-mono text-sm">⚙️ {t('pomodoro.settings', 'Settings')}</button>;
  }

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-md shadow-2xl shadow-cyber-cyan/10 max-h-[90vh] overflow-auto">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between sticky top-0 bg-cyber-surface">
          <h3 className="text-lg font-cyber font-semibold text-cyber-cyan">⚙️ {t('pomodoro.settingsTitle', 'Pomodoro Settings')}</h3>
          <button onClick={() => setIsOpen(false)} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>
        <div className="p-4 space-y-4">
          {[
            { key: 'workDurationMinutes', label: t('pomodoro.workDuration', 'Work Duration (minutes)'), min: 1, max: 120, default: 25 },
            { key: 'shortBreakMinutes', label: t('pomodoro.shortBreakDuration', 'Short Break (minutes)'), min: 1, max: 30, default: 5 },
            { key: 'longBreakMinutes', label: t('pomodoro.longBreakDuration', 'Long Break (minutes)'), min: 1, max: 60, default: 15 },
            { key: 'sessionsUntilLongBreak', label: t('pomodoro.sessionsUntilLong', 'Sessions Until Long Break'), min: 1, max: 10, default: 4 },
          ].map((field) => (
            <div key={field.key}>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{field.label}</label>
              <input type="number" min={field.min} max={field.max} value={(localSettings as Record<string, number>)[field.key]} onChange={(e) => setLocalSettings({ ...localSettings, [field.key]: parseInt(e.target.value) || field.default })} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
            </div>
          ))}
          <div className="space-y-3 pt-2">
            {[
              { key: 'autoStartBreaks', label: t('pomodoro.autoStartBreaks', 'Auto-start breaks') },
              { key: 'autoStartWork', label: t('pomodoro.autoStartWork', 'Auto-start work sessions') },
            ].map((toggle) => (
              <div key={toggle.key} className="flex items-center justify-between">
                <span className="text-sm text-cyber-cyan/70 font-mono">{toggle.label}</span>
                <button onClick={() => setLocalSettings({ ...localSettings, [toggle.key]: !(localSettings as Record<string, boolean>)[toggle.key] })} className={`w-12 h-6 rounded-full transition-colors ${(localSettings as Record<string, boolean>)[toggle.key] ? 'bg-cyber-cyan' : 'bg-cyber-cyan/20'}`}>
                  <div className={`w-5 h-5 bg-cyber-bg rounded-full shadow transition-transform ${(localSettings as Record<string, boolean>)[toggle.key] ? 'translate-x-6' : 'translate-x-0.5'}`} />
                </button>
              </div>
            ))}
          </div>
          <div className="flex gap-2 pt-4">
            <button onClick={() => setIsOpen(false)} className="flex-1 px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">{t('common.cancel', 'Cancel')}</button>
            <button onClick={handleSave} disabled={isLoading} className="flex-1 cyber-button disabled:opacity-50">{t('common.save', 'Save')}</button>
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Main Pomodoro Page (Cyberpunk)
// =============================================================================

export default function PomodoroPage() {
  const { activeSession, remainingSeconds, settings, isLoading, fetchActiveSession, fetchSettings, fetchTodayStats } = usePomodoroStore();
  const { t } = useTranslation();

  useEffect(() => {
    fetchSettings();
    fetchActiveSession();
    fetchTodayStats();
    if (Notification.permission === 'default') Notification.requestPermission();
  }, [fetchSettings, fetchActiveSession, fetchTodayStats]);

  const getTotalSeconds = () => activeSession ? activeSession.durationMinutes * 60 : settings.workDurationMinutes * 60;
  const currentSessionType: SessionType = activeSession?.sessionType ?? 'work';

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between">
          <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan"><TypewriterText text={t('nav.pomodoro', 'Pomodoro')} speed={30} /></h1>
          <SettingsPanel />
        </div>
      </header>

      <div className="flex-1 flex flex-col items-center justify-center p-4 sm:p-8 gap-6 sm:gap-8 relative">
        {isLoading && !activeSession ? (
          <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" />
        ) : (
          <>
            <TimerCircle remainingSeconds={activeSession?.status === 'running' || activeSession?.status === 'paused' ? remainingSeconds : getTotalSeconds()} totalSeconds={getTotalSeconds()} sessionType={currentSessionType} />
            {activeSession?.taskDescription && (
              <div className="text-center max-w-md px-4 py-2 bg-cyber-surface/50 rounded-lg border border-cyber-cyan/20">
                <p className="text-cyber-cyan/60 font-mono text-sm italic">"{activeSession.taskDescription}"</p>
              </div>
            )}
            <ControlButtons />
          </>
        )}
      </div>

      <div className="p-3 sm:p-4 relative z-10">
        <StatsCard />
      </div>
    </div>
  );
}
