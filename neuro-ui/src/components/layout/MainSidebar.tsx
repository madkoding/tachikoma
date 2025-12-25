import { useState } from 'react';
import { NavLink, useLocation } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { useMusicStore } from '../../stores/musicStore';
import { useNotificationStore } from '../../stores/notificationStore';
import SettingsModal from './SettingsModal';

// Navigation items configuration
const navItems = [
  { path: '/', icon: 'chat', labelKey: 'nav.chat' },
  { path: '/checklists', icon: 'checklist', labelKey: 'nav.checklists' },
  { path: '/kanban', icon: 'kanban', labelKey: 'nav.kanban' },
  { path: '/notes', icon: 'notes', labelKey: 'nav.notes' },
  { path: '/docs', icon: 'docs', labelKey: 'nav.docs' },
  { path: '/calendar', icon: 'calendar', labelKey: 'nav.calendar' },
  { path: '/pomodoro', icon: 'pomodoro', labelKey: 'nav.pomodoro' },
  { path: '/music', icon: 'music', labelKey: 'nav.music' },
  { path: '/images', icon: 'images', labelKey: 'nav.images' },
];

export default function MainSidebar() {
  const { t } = useTranslation();
  const location = useLocation();
  const currentSong = useMusicStore(state => state.player.currentSong);
  const activeNotifications = useNotificationStore(state => state.activeNotifications);
  const [isSettingsOpen, setIsSettingsOpen] = useState(false);
  
  // Show extra padding when mini player is visible (not on music page and has song)
  const showMiniPlayerPadding = currentSong && location.pathname !== '/music';

  return (
    <>
      <aside className={clsx(
        "relative z-50 h-full w-12 sm:w-16 bg-cyber-surface border-r border-cyber-cyan/20 shrink-0 flex flex-col",
        showMiniPlayerPadding && "pb-16 sm:pb-0"
      )}>
        {/* Glow effect */}
        <div className="absolute inset-0 bg-gradient-to-b from-cyber-cyan/5 to-transparent pointer-events-none"></div>

        {/* Logo - clickable to open settings */}
        <div className="p-1 sm:p-2 relative">
          <button
            onClick={() => setIsSettingsOpen(true)}
            className="flex items-center justify-center w-full"
            title={t('settings.title')}
          >
            <div className="w-8 h-8 sm:w-10 sm:h-10 shrink-0 rounded-lg bg-cyber-cyan/20 border border-cyber-cyan flex items-center justify-center shadow-[0_0_15px_rgba(0,245,255,0.3)] hover:bg-cyber-cyan/30 hover:shadow-[0_0_20px_rgba(0,245,255,0.5)] transition-all cursor-pointer">
              <TachikomaIcon />
            </div>
          </button>
        </div>

        {/* Navigation Items */}
        <nav className="flex-1 px-0.5 sm:px-1 py-1 sm:py-2 space-y-0.5 sm:space-y-1 relative overflow-y-auto scrollbar-thin scrollbar-thumb-cyber-cyan/20 scrollbar-track-transparent">
          {navItems.map((item) => (
            <NavItem
              key={item.path}
              to={item.path}
              icon={<NavIcon type={item.icon} />}
              label={t(item.labelKey)}
              hasNotification={!!activeNotifications[item.path]}
            />
          ))}
        </nav>
      </aside>

      {/* Settings Modal */}
      <SettingsModal isOpen={isSettingsOpen} onClose={() => setIsSettingsOpen(false)} />
    </>
  );
}

interface NavItemProps {
  readonly to: string;
  readonly icon: React.ReactNode;
  readonly label: string;
  readonly hasNotification?: boolean;
}

function NavItem({ to, icon, label, hasNotification }: Readonly<NavItemProps>) {
  return (
    <NavLink
      to={to}
      title={label}
      end={to === '/'}
      className={({ isActive }) =>
        clsx(
          'relative flex items-center justify-center px-1.5 sm:px-2 py-2 sm:py-2.5 rounded-xl transition-all font-mono text-sm tracking-wide',
          isActive
            ? 'bg-cyber-cyan/10 text-cyber-cyan border border-cyber-cyan/50 shadow-[0_0_15px_rgba(0,245,255,0.2)]'
            : 'text-cyber-cyan/60 hover:text-cyber-cyan hover:bg-cyber-cyan/5 border border-transparent hover:border-cyber-cyan/20',
          hasNotification && !isActive && 'notification-pulse'
        )
      }
    >
      {icon}
      {/* Notification indicator dot */}
      {hasNotification && (
        <span className="absolute top-0.5 right-0.5 sm:top-1 sm:right-1 w-2 h-2 bg-cyber-magenta rounded-full animate-ping-slow shadow-[0_0_8px_rgba(255,0,128,0.8)]" />
      )}
    </NavLink>
  );
}

// Icon Components
interface NavIconProps {
  readonly type: string;
}

function NavIcon({ type }: NavIconProps) {
  const icons: Record<string, React.ReactNode> = {
    chat: <ChatIcon />,
    checklist: <ChecklistIcon />,
    kanban: <KanbanIcon />,
    notes: <NotesIcon />,
    docs: <DocsIcon />,
    calendar: <CalendarIcon />,
    pomodoro: <PomodoroIcon />,
    music: <MusicIcon />,
    images: <ImagesIcon />,
  };
  return <>{icons[type] || <ChatIcon />}</>;
}

function TachikomaIcon() {
  return (
    <svg className="w-6 h-6 text-cyber-cyan" fill="currentColor" stroke="currentColor" viewBox="0 0 100 100">
      <g fill="currentColor">
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(0 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(45 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(90 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(135 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(180 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(225 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(270 50 50)" />
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(315 50 50)" />
      </g>
      <circle cx="50" cy="50" r="38" fill="none" stroke="currentColor" strokeWidth="6" />
      <circle cx="50" cy="50" r="28" fill="none" stroke="currentColor" strokeWidth="4" />
      <circle cx="50" cy="50" r="18" fill="none" stroke="currentColor" strokeWidth="3" />
      <circle cx="50" cy="50" r="6" fill="currentColor" />
    </svg>
  );
}

function ChatIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 12h.01M12 12h.01M16 12h.01M21 12c0 4.418-4.03 8-9 8a9.863 9.863 0 01-4.255-.949L3 20l1.395-3.72C3.512 15.042 3 13.574 3 12c0-4.418 4.03-8 9-8s9 3.582 9 8z" />
    </svg>
  );
}

function ChecklistIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
    </svg>
  );
}

function KanbanIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 17V7m0 10a2 2 0 01-2 2H5a2 2 0 01-2-2V7a2 2 0 012-2h2a2 2 0 012 2m0 10a2 2 0 002 2h2a2 2 0 002-2M9 7a2 2 0 012-2h2a2 2 0 012 2m0 10V7m0 10a2 2 0 002 2h2a2 2 0 002-2V7a2 2 0 00-2-2h-2a2 2 0 00-2 2" />
    </svg>
  );
}

function NotesIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z" />
    </svg>
  );
}

function DocsIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
    </svg>
  );
}

function CalendarIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
    </svg>
  );
}

function PomodoroIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
    </svg>
  );
}

function MusicIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 19V6l12-3v13M9 19c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zm12-3c0 1.105-1.343 2-3 2s-3-.895-3-2 1.343-2 3-2 3 .895 3 2zM9 10l12-3" />
    </svg>
  );
}

function ImagesIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 16l4.586-4.586a2 2 0 012.828 0L16 16m-2-2l1.586-1.586a2 2 0 012.828 0L20 14m-6-6h.01M6 20h12a2 2 0 002-2V6a2 2 0 00-2-2H6a2 2 0 00-2 2v12a2 2 0 002 2z" />
    </svg>
  );
}
