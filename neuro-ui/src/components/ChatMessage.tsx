import { Message } from '../stores/chatStore';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import ReactMarkdown from 'react-markdown';

interface ChatMessageProps {
  readonly message: Message;
}

export default function ChatMessage({ message }: Readonly<ChatMessageProps>) {
  const { t } = useTranslation();
  const isUser = message.role === 'user';

  return (
    <div
      className={clsx(
        'flex gap-3 p-4 rounded border transition-all',
        isUser 
          ? 'bg-cyber-magenta/5 border-cyber-magenta/20' 
          : 'bg-cyber-cyan/5 border-cyber-cyan/20'
      )}
    >
      {/* Avatar */}
      <div
        className={clsx(
          'w-8 h-8 rounded flex items-center justify-center text-sm font-bold shrink-0 font-cyber',
          isUser 
            ? 'bg-cyber-magenta/20 text-cyber-magenta border border-cyber-magenta/50' 
            : 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50'
        )}
      >
        {isUser ? 'U' : 'T'}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className={clsx(
            'font-medium text-sm font-mono',
            isUser ? 'text-cyber-magenta' : 'neon-cyan'
          )}>
            {isUser ? t('message.user') : t('message.assistant')}
          </span>
          <span className="text-xs text-cyber-cyan/40 font-mono">
            {new Date(message.createdAt).toLocaleTimeString()}
          </span>
        </div>
        
        <div className="prose prose-sm max-w-none">
          <ReactMarkdown>{message.content}</ReactMarkdown>
        </div>

        {/* Token info for assistant messages */}
        {!isUser && message.tokensPrompt !== undefined && (
          <div className="mt-2 text-xs text-cyber-cyan/50 flex flex-wrap gap-3 font-mono">
            {message.model && <span>{t('message.model')}: <span className="text-cyber-green">{message.model}</span></span>}
            <span>{t('message.tokens')}: <span className="text-cyber-yellow">{message.tokensPrompt} + {message.tokensCompletion}</span></span>
            {message.processingTimeMs !== undefined && message.processingTimeMs > 0 && (
              <>
                <span>{t('message.time')}: <span className="text-cyber-magenta">{(message.processingTimeMs / 1000).toFixed(1)}s</span></span>
                <span>{t('message.speed')}: <span className="text-cyber-cyan">{((message.tokensCompletion || 0) / (message.processingTimeMs / 1000)).toFixed(1)} tok/s</span></span>
              </>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
