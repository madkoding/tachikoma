import { Message } from '../stores/chatStore';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { memo } from 'react';
import StreamingText from './StreamingText';
import ChecklistDetector from './ChecklistDetector';
import LedDate from './common/LedDate';

interface ChatMessageProps {
  readonly message: Message;
  readonly isStreaming?: boolean;
}

function ChatMessage({ message, isStreaming = false }: Readonly<ChatMessageProps>) {
  const { t } = useTranslation();
  const isUser = message.role === 'user';

  return (
    <div
      className={clsx(
        'flex gap-2 sm:gap-3 p-2 sm:p-4 rounded border transition-all',
        isUser 
          ? 'bg-cyber-magenta/5 border-cyber-magenta/20' 
          : 'bg-cyber-cyan/5 border-cyber-cyan/20'
      )}
    >
      {/* Avatar */}
      <div
        className={clsx(
          'w-7 h-7 sm:w-8 sm:h-8 rounded flex items-center justify-center text-xs sm:text-sm font-bold shrink-0 font-cyber',
          isUser 
            ? 'bg-cyber-magenta/20 text-cyber-magenta border border-cyber-magenta/50' 
            : 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50'
        )}
      >
        {isUser ? 'U' : 'T'}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0 overflow-hidden">
        <div className="flex items-center gap-2 mb-1">
          <span className={clsx(
            'font-medium text-xs sm:text-sm font-mono',
            isUser ? 'text-cyber-magenta' : 'neon-cyan'
          )}>
            {isUser ? t('message.user') : t('message.assistant')}
          </span>
          <LedDate date={message.createdAt} format="time" className="text-[10px] sm:text-xs" />
        </div>
        
        <div className={clsx(
          'prose prose-sm max-w-none prose-invert text-xs sm:text-sm md:text-base overflow-x-auto',
          isStreaming && 'message-streaming'
        )}>
          <StreamingText content={message.content} isStreaming={isStreaming && !isUser} />
        </div>

        {/* Checklist detector - only for assistant messages */}
        {!isUser && (
          <ChecklistDetector content={message.content} isStreaming={isStreaming} />
        )}

        {/* Token info for assistant messages */}
        {!isUser && (message.model || message.tokensPrompt !== undefined || message.processingTimeMs !== undefined) && (
          <div className="mt-2 text-[10px] sm:text-xs text-cyber-cyan/50 flex flex-wrap gap-2 sm:gap-3 font-mono">
            {message.model && <span>{t('message.model')}: <span className="text-cyber-green">{message.model}</span></span>}
            {(message.tokensPrompt !== undefined || message.tokensCompletion !== undefined) && (
              <span className="hidden sm:inline">{t('message.tokens')}: <span className="text-cyber-yellow">{message.tokensPrompt ?? 0} + {message.tokensCompletion ?? 0}</span></span>
            )}
            {message.processingTimeMs !== undefined && message.processingTimeMs > 0 && (
              <>
                <span>{t('message.time')}: <span className="text-cyber-magenta">{(message.processingTimeMs / 1000).toFixed(1)}s</span></span>
                <span className="hidden sm:inline">{t('message.speed')}: <span className="text-cyber-cyan">{((message.tokensCompletion || 0) / (message.processingTimeMs / 1000)).toFixed(1)} tok/s</span></span>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}

export default memo(ChatMessage);