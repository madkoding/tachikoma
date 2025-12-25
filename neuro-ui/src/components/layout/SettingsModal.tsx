import { useTranslation } from 'react-i18next';
import clsx from 'clsx';

interface SettingsModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
}

export default function SettingsModal({ isOpen, onClose }: SettingsModalProps) {
  const { t, i18n } = useTranslation();

  if (!isOpen) return null;

  const handleLanguageChange = (lang: string) => {
    i18n.changeLanguage(lang);
  };

  return (
    <div 
      className="fixed inset-0 bg-black/70 flex items-center justify-center z-[100] p-4"
      onClick={onClose}
    >
      <div 
        className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-4 sm:p-6 max-w-sm w-full shadow-[0_0_30px_rgba(0,245,255,0.2)]"
        onClick={(e) => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan flex items-center gap-2">
            <SettingsIcon />
            {t('settings.title')}
          </h2>
          <button
            onClick={onClose}
            className="p-1.5 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all"
          >
            <CloseIcon />
          </button>
        </div>

        {/* Language Setting */}
        <div className="space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/50 uppercase tracking-wider mb-2">
              {t('settings.language')}
            </label>
            <div className="flex gap-2">
              <button
                onClick={() => handleLanguageChange('es')}
                className={clsx(
                  'flex-1 py-2 px-3 rounded-lg border text-sm font-mono transition-all',
                  i18n.language === 'es'
                    ? 'bg-cyber-cyan/20 border-cyber-cyan text-cyber-cyan'
                    : 'border-cyber-cyan/20 text-cyber-cyan/50 hover:border-cyber-cyan/40 hover:text-cyber-cyan'
                )}
              >
                🇪🇸 Español
              </button>
              <button
                onClick={() => handleLanguageChange('en')}
                className={clsx(
                  'flex-1 py-2 px-3 rounded-lg border text-sm font-mono transition-all',
                  i18n.language === 'en'
                    ? 'bg-cyber-cyan/20 border-cyber-cyan text-cyber-cyan'
                    : 'border-cyber-cyan/20 text-cyber-cyan/50 hover:border-cyber-cyan/40 hover:text-cyber-cyan'
                )}
              >
                🇺🇸 English
              </button>
            </div>
          </div>

          {/* Version info */}
          <div className="pt-4 border-t border-cyber-cyan/20">
            <p className="text-xs text-cyber-cyan/30 font-mono text-center">
              NEURO-OS v1.0.0
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}

function SettingsIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
    </svg>
  );
}
