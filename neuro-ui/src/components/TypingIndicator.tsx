import { useTranslation } from 'react-i18next';

export default function TypingIndicator() {
  const { t } = useTranslation();

  return (
    <div className="flex gap-3 p-4 rounded border border-cyber-cyan/20 bg-cyber-cyan/5">
      <div className="w-8 h-8 rounded flex items-center justify-center bg-cyber-cyan/20 text-cyber-cyan text-sm font-bold shrink-0 border border-cyber-cyan/50 font-cyber">
        T
      </div>
      <div className="flex items-center">
        <div className="typing-indicator flex gap-1">
          <span></span>
          <span></span>
          <span></span>
        </div>
        <span className="ml-3 text-sm text-cyber-cyan/60 font-mono">{t('chat.thinking')}</span>
      </div>
    </div>
  );
}
