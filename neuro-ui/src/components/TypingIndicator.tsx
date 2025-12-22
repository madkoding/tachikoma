import { useTranslation } from 'react-i18next';

export default function TypingIndicator() {
  const { t } = useTranslation();

  return (
    <div className="flex items-center gap-2 py-2 px-3">
      <div className="flex items-center gap-1 text-cyber-cyan/70 font-mono text-sm">
        <span className="text-cyber-cyan">{'>'}</span>
        <span>{t('chat.thinking')}</span>
        <span className="animate-pulse text-cyber-cyan">_</span>
      </div>
      <div className="flex gap-0.5">
        <span className="w-1.5 h-1.5 bg-cyber-cyan rounded-full animate-bounce" style={{ animationDelay: '0ms' }}></span>
        <span className="w-1.5 h-1.5 bg-cyber-cyan rounded-full animate-bounce" style={{ animationDelay: '150ms' }}></span>
        <span className="w-1.5 h-1.5 bg-cyber-cyan rounded-full animate-bounce" style={{ animationDelay: '300ms' }}></span>
      </div>
    </div>
  );
}
