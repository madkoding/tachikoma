import { Outlet, NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { useState } from 'react';
import clsx from 'clsx';
import { getMode, changeMode, UIMode } from '../mode';

export default function Layout() {
  const { t, i18n } = useTranslation();
  const [mode, setMode] = useState<UIMode>(getMode());

  const toggleLanguage = () => {
    i18n.changeLanguage(i18n.language === 'en' ? 'es' : 'en');
  };

  const toggleMode = () => {
    const next: UIMode = mode === 'simple' ? 'advanced' : 'simple';
    changeMode(next);
    setMode(next);
  };

  return (
    <div className="h-screen bg-cyber-bg flex overflow-hidden">
      {/* Sidebar - always compact with icons only */}
      <aside className="relative z-50 h-full w-16 bg-cyber-surface border-r border-cyber-cyan/20 shrink-0">
        {/* Glow effect */}
        <div className="absolute inset-0 bg-gradient-to-b from-cyber-cyan/5 to-transparent pointer-events-none"></div>
        
        <div className="p-2 relative">
          <div className="flex items-center justify-center">
            <div className="w-10 h-10 shrink-0 rounded-lg bg-cyber-cyan/20 border border-cyber-cyan flex items-center justify-center shadow-[0_0_15px_rgba(0,245,255,0.3)]">
              <TachikomaIcon />
            </div>
          </div>
        </div>

        <nav className="px-1 space-y-1 relative">
          <NavItem to="/" icon={<DashboardIcon />} label={t('nav.dashboard')} />
          <NavItem to="/graph" icon={<GraphIcon />} label={t('nav.graph')} />
        </nav>

        <div className="absolute bottom-4 left-1 right-1 space-y-1">
          {/* Mode toggle */}
          <button
            onClick={toggleMode}
            className="w-full flex items-center justify-center px-2 py-2 text-sm text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all border border-transparent hover:border-cyber-cyan/30 font-mono"
            title={mode === 'simple' ? t('mode.advancedHint') : t('mode.simpleHint')}
          >
            {mode === 'simple' ? <AdvancedIcon /> : <SimpleIcon />}
          </button>
          <button
            onClick={toggleLanguage}
            className="w-full flex items-center justify-center px-2 py-2 text-sm text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all border border-transparent hover:border-cyber-cyan/30 font-mono"
            title={i18n.language === 'en' ? 'Cambiar a Español' : 'Switch to English'}
          >
            <LanguageIcon />
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 flex flex-col h-full overflow-hidden">
        <div className="flex-1 overflow-hidden">
          <Outlet />
        </div>
      </main>
    </div>
  );
}

interface NavItemProps {
  readonly to: string;
  readonly icon: React.ReactNode;
  readonly label: string;
}

function NavItem({ to, icon, label }: Readonly<NavItemProps>) {
  return (
    <NavLink
      to={to}
      title={label}
      className={({ isActive }) =>
        clsx(
          'flex items-center justify-center px-2 py-3 rounded-xl transition-all font-mono text-sm tracking-wide',
          isActive
            ? 'bg-cyber-cyan/10 text-cyber-cyan border border-cyber-cyan/50 shadow-[0_0_15px_rgba(0,245,255,0.2)]'
            : 'text-cyber-cyan/60 hover:text-cyber-cyan hover:bg-cyber-cyan/5 border border-transparent hover:border-cyber-cyan/20'
        )
      }
    >
      {icon}
    </NavLink>
  );
}

function TachikomaIcon() {
  return (
    <svg className="w-6 h-6 text-cyber-cyan" fill="currentColor" stroke="currentColor" viewBox="0 0 100 100">
      <g fill="currentColor">
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(0 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(45 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(90 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(135 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(180 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(225 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(270 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(315 50 50)"/>
      </g>
      <circle cx="50" cy="50" r="38" fill="none" stroke="currentColor" strokeWidth="6"/>
      <circle cx="50" cy="50" r="28" fill="none" stroke="currentColor" strokeWidth="4"/>
      <circle cx="50" cy="50" r="18" fill="none" stroke="currentColor" strokeWidth="3"/>
      <circle cx="50" cy="50" r="6" fill="currentColor"/>
    </svg>
  );
}

function DashboardIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M4 5a1 1 0 011-1h14a1 1 0 011 1v2a1 1 0 01-1 1H5a1 1 0 01-1-1V5zM4 13a1 1 0 011-1h6a1 1 0 011 1v6a1 1 0 01-1 1H5a1 1 0 01-1-1v-6zM16 13a1 1 0 011-1h2a1 1 0 011 1v6a1 1 0 01-1 1h-2a1 1 0 01-1-1v-6z" />
    </svg>
  );
}

function GraphIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
    </svg>
  );
}

function LanguageIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M3 5h12M9 3v2m1.048 9.5A18.022 18.022 0 016.412 9m6.088 9h7M11 21l5-10 5 10M12.751 5C11.783 10.77 8.07 15.61 3 18.129" />
    </svg>
  );
}

// Simple mode icon (grid) — shows when currently in simple, click to go advanced
function AdvancedIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  );
}

// Advanced mode icon (sliders) — shows when in advanced, click to go simple
function SimpleIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" />
    </svg>
  );
}
