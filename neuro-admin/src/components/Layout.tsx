import { Outlet, NavLink } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';

export default function Layout() {
  const { t, i18n } = useTranslation();

  const toggleLanguage = () => {
    i18n.changeLanguage(i18n.language === 'en' ? 'es' : 'en');
  };

  return (
    <div className="min-h-screen bg-cyber-bg flex">
      {/* Sidebar */}
      <aside className="w-64 bg-cyber-surface border-r border-cyber-cyan/20 relative">
        {/* Glow effect */}
        <div className="absolute inset-0 bg-gradient-to-b from-cyber-cyan/5 to-transparent pointer-events-none"></div>
        
        <div className="p-6 relative">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 rounded-lg bg-cyber-cyan/20 border border-cyber-cyan flex items-center justify-center shadow-[0_0_15px_rgba(0,245,255,0.3)]">
              <TachikomaIcon />
            </div>
            <div>
              <h1 className="text-lg font-bold neon-cyan font-cyber tracking-wider">TACHIKOMA</h1>
              <p className="text-xs text-cyber-cyan/50 font-mono">ADMIN // v1.0</p>
            </div>
          </div>
        </div>

        <nav className="px-4 space-y-1 relative">
          <NavItem to="/" icon={<DashboardIcon />} label={t('nav.dashboard')} />
          <NavItem to="/graph" icon={<GraphIcon />} label={t('nav.graph')} />
          <NavItem to="/memories" icon={<MemoryIcon />} label={t('nav.memories')} />
        </nav>

        <div className="absolute bottom-4 left-4 right-4">
          <button
            onClick={toggleLanguage}
            className="w-full flex items-center gap-2 px-4 py-2 text-sm text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all border border-transparent hover:border-cyber-cyan/30 font-mono"
          >
            <LanguageIcon />
            {i18n.language === 'en' ? 'ESPAÑOL' : 'ENGLISH'}
          </button>
        </div>
      </aside>

      {/* Main Content */}
      <main className="flex-1 p-8 overflow-auto bg-cyber-bg">
        <Outlet />
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
      className={({ isActive }) =>
        clsx(
          'flex items-center gap-3 px-4 py-3.5 rounded-xl transition-all font-mono text-sm tracking-wide',
          isActive
            ? 'bg-cyber-cyan/10 text-cyber-cyan border border-cyber-cyan/50 shadow-[0_0_15px_rgba(0,245,255,0.2)]'
            : 'text-cyber-cyan/60 hover:text-cyber-cyan hover:bg-cyber-cyan/5 border border-transparent hover:border-cyber-cyan/20'
        )
      }
    >
      {icon}
      <span className="uppercase">{label}</span>
    </NavLink>
  );
}

function TachikomaIcon() {
  return (
    <svg className="w-6 h-6 text-cyber-cyan" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z" />
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

function MemoryIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
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
