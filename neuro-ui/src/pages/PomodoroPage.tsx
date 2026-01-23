import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import {
  usePomodoroStore,
  formatTime,
  getSessionTypeLabel,
  SessionType,
} from '../stores/pomodoroStore';
import { isSessionRunning } from '../api/client';
import TypewriterText from '../components/common/TypewriterText';

// =============================================================================
// Timer Circle (Cyberpunk Enhanced)
// =============================================================================

function TimerCircle({ remainingSeconds, totalSeconds, sessionType, isRunning }: { remainingSeconds: number; totalSeconds: number; sessionType: SessionType; isRunning: boolean }) {
  const progress = totalSeconds > 0 ? (remainingSeconds / totalSeconds) * 100 : 100;
  const circumference = 2 * Math.PI * 140;
  const strokeDashoffset = circumference - (progress / 100) * circumference;

  const getColor = () => {
    switch (sessionType) {
      case 'work': return { stroke: '#ff0040', glow: '#ff004080', led: '#ff0040' };
      case 'short_break': return { stroke: '#00ff9f', glow: '#00ff9f80', led: '#00ff9f' };
      case 'long_break': return { stroke: '#00d4ff', glow: '#00d4ff80', led: '#00d4ff' };
    }
  };

  const colors = getColor();

  return (
    <div className="relative w-56 h-56 sm:w-64 sm:h-64 lg:w-80 lg:h-80 flex items-center justify-center flex-shrink-0">
      {/* Outer glow ring */}
      <div 
        className="absolute inset-0 rounded-full opacity-20 animate-pulse"
        style={{ background: `radial-gradient(circle, ${colors.glow} 0%, transparent 70%)` }} 
      />
      
      {/* Circuit pattern background */}
      <div className="absolute inset-4 rounded-full border border-cyber-cyan/10 overflow-hidden">
        <div 
          className="absolute inset-0 opacity-10"
          style={{
            backgroundImage: `linear-gradient(90deg, ${colors.stroke}20 1px, transparent 1px), linear-gradient(${colors.stroke}20 1px, transparent 1px)`,
            backgroundSize: '20px 20px',
          }}
        />
      </div>

      {/* Main SVG ring */}
      <svg className="absolute w-full h-full -rotate-90" viewBox="0 0 300 300">
        <circle cx="150" cy="150" r="140" fill="none" stroke={`${colors.stroke}15`} strokeWidth="8" strokeDasharray="4 8" />
        <circle 
          cx="150" cy="150" r="140" fill="none" stroke={colors.stroke} strokeWidth="10" strokeLinecap="round" 
          strokeDasharray={circumference} strokeDashoffset={strokeDashoffset} 
          className="transition-all duration-1000 ease-linear" 
          style={{ filter: `drop-shadow(0 0 8px ${colors.glow}) drop-shadow(0 0 20px ${colors.glow})` }} 
        />
        <circle cx="150" cy="150" r="120" fill="none" stroke={`${colors.stroke}30`} strokeWidth="1" />
      </svg>

      {/* Center content - ABSOLUTELY centered */}
      <div className="absolute inset-0 flex items-center justify-center">
        <div className="text-center">
          {/* Timer display with LED font */}
          <div 
            className="text-4xl sm:text-5xl lg:text-6xl led-time leading-none"
            style={{ 
              color: colors.led,
              textShadow: `0 0 20px ${colors.glow}, 0 0 40px ${colors.glow}, 0 0 60px ${colors.glow}`,
            }}
          >
            {formatTime(remainingSeconds)}
          </div>
          
          {/* Session type label */}
          <div 
            className="mt-3 px-3 py-1 rounded-full border font-mono text-[10px] sm:text-xs uppercase tracking-[0.2em] inline-block"
            style={{ 
              borderColor: `${colors.stroke}50`,
              color: colors.stroke,
              textShadow: `0 0 10px ${colors.glow}`,
              background: `linear-gradient(180deg, ${colors.stroke}10 0%, transparent 100%)`,
            }}
          >
            {getSessionTypeLabel(sessionType)}
          </div>
          
          {/* Status indicator */}
          <div className="flex items-center justify-center gap-2 mt-2">
            <div 
              className={`w-2 h-2 rounded-full ${isRunning ? 'animate-pulse' : ''}`}
              style={{ 
                backgroundColor: isRunning ? colors.stroke : `${colors.stroke}40`,
                boxShadow: isRunning ? `0 0 8px ${colors.glow}, 0 0 16px ${colors.glow}` : 'none',
              }}
            />
            <span className="text-[10px] font-mono uppercase tracking-wider" style={{ color: `${colors.stroke}80` }}>
              {isRunning ? 'ACTIVE' : 'STANDBY'}
            </span>
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Cyberpunk Button Component
// =============================================================================

function CyberButton({ 
  onClick, disabled, variant = 'primary', size = 'md', icon, children, loading = false,
}: { 
  onClick: () => void; disabled?: boolean;
  variant?: 'primary' | 'secondary' | 'danger' | 'success' | 'warning' | 'ghost';
  size?: 'sm' | 'md' | 'lg'; icon?: string; children: React.ReactNode; loading?: boolean;
}) {
  const variants = {
    primary: { bg: 'from-cyber-cyan to-blue-500', hover: 'hover:from-blue-400 hover:to-cyber-cyan', shadow: 'shadow-cyber-cyan/40 hover:shadow-cyber-cyan/60', border: 'border-cyber-cyan/50', text: 'text-black' },
    secondary: { bg: 'from-purple-500 to-pink-500', hover: 'hover:from-purple-400 hover:to-pink-400', shadow: 'shadow-purple-500/40 hover:shadow-purple-500/60', border: 'border-purple-500/50', text: 'text-white' },
    danger: { bg: 'from-red-500 to-rose-600', hover: 'hover:from-red-400 hover:to-rose-500', shadow: 'shadow-red-500/40 hover:shadow-red-500/60', border: 'border-red-500/50', text: 'text-white' },
    success: { bg: 'from-emerald-500 to-green-500', hover: 'hover:from-emerald-400 hover:to-green-400', shadow: 'shadow-emerald-500/40 hover:shadow-emerald-500/60', border: 'border-emerald-500/50', text: 'text-white' },
    warning: { bg: 'from-amber-500 to-orange-500', hover: 'hover:from-amber-400 hover:to-orange-400', shadow: 'shadow-amber-500/40 hover:shadow-amber-500/60', border: 'border-amber-500/50', text: 'text-black' },
    ghost: { bg: 'from-transparent to-transparent', hover: 'hover:from-cyber-cyan/10 hover:to-cyber-cyan/10', shadow: '', border: 'border-cyber-cyan/30 hover:border-cyber-cyan/60', text: 'text-cyber-cyan/70 hover:text-cyber-cyan' },
  };
  const sizes = { sm: 'px-3 py-1.5 text-xs', md: 'px-5 py-2.5 text-sm', lg: 'px-8 py-4 text-base' };
  const v = variants[variant];

  return (
    <button
      onClick={onClick}
      disabled={disabled || loading}
      className={`relative overflow-hidden ${sizes[size]} bg-gradient-to-r ${v.bg} ${v.hover} ${v.text} font-cyber font-semibold rounded-lg border ${v.border} shadow-lg ${v.shadow} transition-all duration-300 disabled:opacity-40 disabled:cursor-not-allowed active:scale-95 group`}
    >
      <span className="relative flex items-center justify-center gap-2">
        {loading ? <div className="w-4 h-4 border-2 border-current border-t-transparent rounded-full animate-spin" /> : icon ? <span>{icon}</span> : null}
        {children}
      </span>
    </button>
  );
}

// =============================================================================
// Control Buttons (Cyberpunk Enhanced)
// =============================================================================

function ControlButtons() {
  const store = usePomodoroStore();
  const { activeSession, isLoading } = store;
  const { t } = useTranslation();
  const [actionLoading, setActionLoading] = useState<string | null>(null);

  const handleStartSession = async (type: 'work' | 'short_break' | 'long_break') => {
    setActionLoading(type);
    try {
      await store.startSession(type);
    } catch (error) {
      console.error('Error starting session:', error);
    } finally {
      setActionLoading(null);
    }
  };

  const handlePause = async () => {
    setActionLoading('pause');
    try {
      await store.pauseSession();
    } catch (error) {
      console.error('Error pausing:', error);
    } finally {
      setActionLoading(null);
    }
  };

  const handleResume = async () => {
    setActionLoading('resume');
    try {
      await store.resumeSession();
    } catch (error) {
      console.error('Error resuming:', error);
    } finally {
      setActionLoading(null);
    }
  };

  const handleComplete = async () => {
    setActionLoading('complete');
    try {
      await store.completeSession();
    } catch (error) {
      console.error('Error completing:', error);
    } finally {
      setActionLoading(null);
    }
  };

  const handleCancel = async () => {
    setActionLoading('cancel');
    try {
      await store.cancelSession();
    } catch (error) {
      console.error('Error canceling:', error);
    } finally {
      setActionLoading(null);
    }
  };

  if (!activeSession || activeSession.status === 'completed' || activeSession.status === 'cancelled') {
    return (
      <div className="flex flex-col items-center gap-4">
        <CyberButton onClick={() => handleStartSession('work')} disabled={isLoading} variant="danger" size="lg" icon="🍅" loading={actionLoading === 'work'}>
          {t('pomodoro.startWork', 'START WORK SESSION')}
        </CyberButton>
        <div className="flex gap-3 flex-wrap justify-center">
          <CyberButton onClick={() => handleStartSession('short_break')} disabled={isLoading} variant="success" size="md" icon="☕" loading={actionLoading === 'short_break'}>
            {t('pomodoro.shortBreak', 'Short Break')}
          </CyberButton>
          <CyberButton onClick={() => handleStartSession('long_break')} disabled={isLoading} variant="primary" size="md" icon="🌴" loading={actionLoading === 'long_break'}>
            {t('pomodoro.longBreak', 'Long Break')}
          </CyberButton>
        </div>
        <p className="text-xs text-cyber-cyan/30 font-mono">
          Press <kbd className="px-1.5 py-0.5 bg-cyber-cyan/10 border border-cyber-cyan/20 rounded text-cyber-cyan/50">Space</kbd> to start
        </p>
      </div>
    );
  }

  return (
    <div className="flex flex-col items-center gap-4">
      <div className="flex flex-wrap justify-center gap-3">
        {isSessionRunning(activeSession.status) ? (
          <CyberButton onClick={handlePause} disabled={isLoading} variant="warning" size="md" icon="⏸️" loading={actionLoading === 'pause'}>
            {t('pomodoro.pause', 'PAUSE')}
          </CyberButton>
        ) : (
          <CyberButton onClick={handleResume} disabled={isLoading} variant="success" size="md" icon="▶️" loading={actionLoading === 'resume'}>
            {t('pomodoro.resume', 'RESUME')}
          </CyberButton>
        )}
        <CyberButton onClick={handleComplete} disabled={isLoading} variant="primary" size="md" icon="✅" loading={actionLoading === 'complete'}>
          {t('pomodoro.complete', 'COMPLETE')}
        </CyberButton>
        <CyberButton onClick={handleCancel} disabled={isLoading} variant="ghost" size="md" icon="❌" loading={actionLoading === 'cancel'}>
          {t('pomodoro.cancel', 'CANCEL')}
        </CyberButton>
      </div>
      {activeSession.sessionType === 'work' && (
        <div className="flex items-center gap-2 px-4 py-2 bg-cyber-surface/50 rounded-lg border border-cyber-cyan/10">
          <span className="text-xs text-cyber-cyan/40 font-mono">NEXT →</span>
          <span className="text-sm text-cyber-cyan/70 font-mono uppercase tracking-wide">
            {getSessionTypeLabel(store.getNextSessionType())}
          </span>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Stats Card (Cyberpunk Enhanced)
// =============================================================================

function StatsCard() {
  const { todayStats, completedWorkSessions, settings } = usePomodoroStore();
  const { t } = useTranslation();
  const sessionsUntilLongBreak = settings.sessionsUntilLongBreak - (completedWorkSessions % settings.sessionsUntilLongBreak);

  const stats = [
    { label: t('pomodoro.sessionsCompleted', 'Sessions'), value: todayStats?.completedSessions ?? 0, color: '#ff0040', icon: '🍅' },
    { label: t('pomodoro.workMinutes', 'Work Min'), value: todayStats?.totalWorkMinutes ?? 0, color: '#00d4ff', icon: '⏱️' },
    { label: t('pomodoro.breakMinutes', 'Break Min'), value: todayStats?.totalBreakMinutes ?? 0, color: '#00ff9f', icon: '☕' },
    { label: t('pomodoro.untilLongBreak', 'To Long Break'), value: sessionsUntilLongBreak, color: '#a855f7', icon: '🌴' },
  ];

  return (
    <div className="bg-cyber-surface/80 backdrop-blur-sm border border-cyber-cyan/20 rounded-xl p-4">
      <div className="flex items-center justify-between mb-3">
        <h3 className="text-sm font-cyber font-semibold text-cyber-cyan flex items-center gap-2">
          <span>📊</span> {t('pomodoro.todayStats', "TODAY'S STATS")}
        </h3>
        <div className="text-xs font-mono text-cyber-cyan/30">
          {new Date().toLocaleDateString('en-US', { weekday: 'short', month: 'short', day: 'numeric' })}
        </div>
      </div>
      <div className="grid grid-cols-4 gap-2">
        {stats.map((stat) => (
          <div key={stat.label} className="relative group text-center p-2 bg-black/40 rounded-lg border border-cyber-cyan/10">
            <div className="text-sm mb-0.5">{stat.icon}</div>
            <div className="text-xl sm:text-2xl led-time" style={{ color: stat.color, textShadow: `0 0 10px ${stat.color}80` }}>
              {String(stat.value).padStart(2, '0')}
            </div>
            <div className="text-[8px] sm:text-[10px] text-cyber-cyan/40 font-mono uppercase">{stat.label}</div>
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// Settings Panel
// =============================================================================

function SettingsPanel() {
  const { settings, updateSettings, isLoading } = usePomodoroStore();
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);
  const [localSettings, setLocalSettings] = useState(settings);
  const [saving, setSaving] = useState(false);

  useEffect(() => { setLocalSettings(settings); }, [settings]);

  const handleSave = async () => { 
    setSaving(true);
    try { await updateSettings(localSettings); setIsOpen(false); } 
    finally { setSaving(false); }
  };

  if (!isOpen) {
    return (
      <button onClick={() => setIsOpen(true)} className="flex items-center gap-2 px-3 py-1.5 text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all font-mono text-sm border border-transparent hover:border-cyber-cyan/30">
        <span>⚙️</span><span className="hidden sm:inline">{t('pomodoro.settings', 'Settings')}</span>
      </button>
    );
  }

  return (
    <div className="fixed inset-0 bg-black/90 backdrop-blur-md flex items-center justify-center z-50 p-4 overflow-auto">
      <div className="relative bg-cyber-surface border border-cyber-cyan/40 rounded-xl w-full max-w-md shadow-2xl shadow-cyber-cyan/20">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between">
          <h3 className="text-lg font-cyber font-semibold text-cyber-cyan">⚙️ {t('pomodoro.settingsTitle', 'POMODORO CONFIG')}</h3>
          <button onClick={() => setIsOpen(false)} className="w-8 h-8 flex items-center justify-center text-cyber-cyan/50 hover:text-cyber-cyan rounded">✕</button>
        </div>
        <div className="p-4 space-y-4">
          <div className="space-y-3">
            <div className="text-xs font-mono text-cyber-cyan/40 uppercase tracking-wider">⏱️ DURATION</div>
            {[
              { key: 'workDurationMinutes', label: 'Work', min: 1, max: 120, default: 25, color: '#ff0040' },
              { key: 'shortBreakMinutes', label: 'Short Break', min: 1, max: 30, default: 5, color: '#00ff9f' },
              { key: 'longBreakMinutes', label: 'Long Break', min: 1, max: 60, default: 15, color: '#00d4ff' },
              { key: 'sessionsUntilLongBreak', label: 'Until Long', min: 1, max: 10, default: 4, color: '#a855f7' },
            ].map((field) => (
              <div key={field.key} className="flex items-center justify-between p-2 bg-black/30 rounded-lg border border-cyber-cyan/10">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 rounded-full" style={{ backgroundColor: field.color }} />
                  <label className="text-sm font-mono text-cyber-cyan/80">{field.label}</label>
                </div>
                <input 
                  type="number" min={field.min} max={field.max} 
                  value={(localSettings as any)[field.key]} 
                  onChange={(e) => setLocalSettings({ ...localSettings, [field.key]: parseInt(e.target.value) || field.default })} 
                  className="w-16 px-2 py-1 bg-cyber-bg border border-cyber-cyan/30 rounded text-center font-mono text-sm"
                  style={{ color: field.color }}
                />
              </div>
            ))}
          </div>
          <div className="space-y-3">
            <div className="text-xs font-mono text-cyber-cyan/40 uppercase tracking-wider">🤖 AUTO</div>
            {[
              { key: 'autoStartBreaks', label: 'Auto-start breaks' },
              { key: 'autoStartWork', label: 'Auto-start work' },
            ].map((toggle) => (
              <div key={toggle.key} className="flex items-center justify-between p-2 bg-black/30 rounded-lg border border-cyber-cyan/10">
                <span className="text-sm text-cyber-cyan/80 font-mono">{toggle.label}</span>
                <button 
                  onClick={() => setLocalSettings({ ...localSettings, [toggle.key]: !(localSettings as any)[toggle.key] })} 
                  className={`w-12 h-6 rounded-full transition-all ${(localSettings as any)[toggle.key] ? 'bg-cyber-cyan' : 'bg-cyber-cyan/20'}`}
                >
                  <div className={`w-5 h-5 bg-cyber-bg rounded-full shadow transition-transform ${(localSettings as any)[toggle.key] ? 'translate-x-6' : 'translate-x-0.5'}`} />
                </button>
              </div>
            ))}
          </div>
          <div className="flex gap-3 pt-2">
            <CyberButton onClick={() => setIsOpen(false)} variant="ghost" size="md">{t('common.cancel', 'Cancel')}</CyberButton>
            <CyberButton onClick={handleSave} disabled={isLoading} variant="primary" size="md" loading={saving}>{t('common.save', 'SAVE')}</CyberButton>
          </div>
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Main Pomodoro Page
// =============================================================================

export default function PomodoroPage() {
  const store = usePomodoroStore();
  const { activeSession, remainingSeconds, settings, isLoading } = store;
  const { t } = useTranslation();

  useEffect(() => {
    store.fetchSettings();
    store.fetchActiveSession();
    store.fetchTodayStats();
    if (Notification.permission === 'default') Notification.requestPermission();
  }, []);

  useEffect(() => {
    const handleKeyPress = (e: KeyboardEvent) => {
      if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) return;
      if (e.code === 'Space' && (!activeSession || activeSession.status === 'completed' || activeSession.status === 'cancelled')) {
        e.preventDefault();
        store.startSession('work');
      }
    };
    window.addEventListener('keydown', handleKeyPress);
    return () => window.removeEventListener('keydown', handleKeyPress);
  }, [activeSession]);

  const getTotalSeconds = () => activeSession ? activeSession.durationMinutes * 60 : settings.workDurationMinutes * 60;
  const currentSessionType: SessionType = activeSession?.sessionType ?? 'work';
  const isRunning = activeSession ? isSessionRunning(activeSession.status) : false;

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-auto">
      {/* Background effects */}
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="flex-shrink-0 p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <span className="text-2xl">🍅</span>
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan">
              <TypewriterText text={t('nav.pomodoro', 'POMODORO')} speed={30} />
            </h1>
            {isRunning && (
              <div className="hidden sm:flex items-center gap-2 px-2 py-1 bg-red-500/20 border border-red-500/30 rounded-full">
                <div className="w-2 h-2 bg-red-500 rounded-full animate-pulse" />
                <span className="text-xs font-mono text-red-400">ACTIVE</span>
              </div>
            )}
          </div>
          <SettingsPanel />
        </div>
      </header>

      {/* Main content - Desktop: horizontal layout, Mobile: vertical */}
      <div className="flex-1 flex flex-col lg:flex-row items-center justify-center p-4 gap-6 lg:gap-12 relative">
        {isLoading && !activeSession ? (
          <div className="flex flex-col items-center gap-4">
            <div className="animate-spin rounded-full h-12 w-12 border-2 border-cyber-cyan border-t-transparent" />
            <span className="text-cyber-cyan/50 font-mono text-sm animate-pulse">INITIALIZING...</span>
          </div>
        ) : (
          <>
            {/* Timer */}
            <TimerCircle 
              remainingSeconds={activeSession && (isSessionRunning(activeSession.status) || activeSession.status === 'paused') ? remainingSeconds : getTotalSeconds()} 
              totalSeconds={getTotalSeconds()} 
              sessionType={currentSessionType}
              isRunning={isRunning}
            />
            
            {/* Controls and task */}
            <div className="flex flex-col items-center gap-4 lg:gap-6">
              {activeSession?.taskDescription && (
                <div className="text-center max-w-sm px-4 py-3 bg-cyber-surface/50 rounded-lg border border-cyber-cyan/20">
                  <div className="text-[10px] font-mono text-cyber-cyan/40 uppercase tracking-wider mb-1">CURRENT TASK</div>
                  <p className="text-cyber-cyan/70 font-mono text-sm">"{activeSession.taskDescription}"</p>
                </div>
              )}
              <ControlButtons />
            </div>
          </>
        )}
      </div>

      {/* Stats - always visible at bottom */}
      <div className="flex-shrink-0 p-3 sm:p-4 relative z-10">
        <StatsCard />
      </div>
    </div>
  );
}
