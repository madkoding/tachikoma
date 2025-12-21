import { useTranslation } from 'react-i18next';

export default function TypingIndicator() {
  const { t } = useTranslation();

  return (
    <div className="flex gap-3 p-4 rounded-lg bg-gray-50 dark:bg-gray-800">
      <div className="w-8 h-8 rounded-full flex items-center justify-center bg-gray-600 dark:bg-gray-500 text-white text-sm font-medium shrink-0">
        N
      </div>
      <div className="flex items-center">
        <div className="typing-indicator flex gap-1">
          <span className="w-2 h-2 bg-gray-400 rounded-full"></span>
          <span className="w-2 h-2 bg-gray-400 rounded-full"></span>
          <span className="w-2 h-2 bg-gray-400 rounded-full"></span>
        </div>
        <span className="ml-2 text-sm text-gray-500">{t('chat.thinking')}</span>
      </div>
    </div>
  );
}
