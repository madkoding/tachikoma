import { useState, KeyboardEvent } from 'react';
import { useTranslation } from 'react-i18next';

interface ChatInputProps {
  onSend: (message: string) => void;
  disabled?: boolean;
}

export default function ChatInput({ onSend, disabled }: ChatInputProps) {
  const { t } = useTranslation();
  const [message, setMessage] = useState('');

  const handleSubmit = () => {
    if (message.trim() && !disabled) {
      onSend(message);
      setMessage('');
    }
  };

  const handleKeyDown = (e: KeyboardEvent<HTMLTextAreaElement>) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSubmit();
    }
  };

  return (
    <div className="flex gap-2">
      <textarea
        value={message}
        onChange={(e) => setMessage(e.target.value)}
        onKeyDown={handleKeyDown}
        placeholder={t('chat.placeholder')}
        disabled={disabled}
        rows={1}
        className="flex-1 resize-none rounded-lg border border-gray-300 dark:border-gray-600 
                   bg-white dark:bg-gray-800 px-4 py-3 text-sm
                   focus:outline-none focus:ring-2 focus:ring-neuro-500
                   disabled:opacity-50 disabled:cursor-not-allowed"
        style={{ maxHeight: '200px' }}
      />
      <button
        onClick={handleSubmit}
        disabled={disabled || !message.trim()}
        className="px-4 py-2 bg-neuro-500 hover:bg-neuro-600 text-white rounded-lg
                   disabled:opacity-50 disabled:cursor-not-allowed
                   transition-colors duration-200"
      >
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
                d="M12 19l9 2-9-18-9 18 9-2zm0 0v-8" />
        </svg>
      </button>
    </div>
  );
}
