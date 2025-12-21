import { Message } from '../stores/chatStore';
import clsx from 'clsx';
import ReactMarkdown from 'react-markdown';

interface ChatMessageProps {
  message: Message;
}

export default function ChatMessage({ message }: ChatMessageProps) {
  const isUser = message.role === 'user';

  return (
    <div
      className={clsx(
        'flex gap-3 p-4 rounded-lg',
        isUser ? 'bg-neuro-50 dark:bg-neuro-900/20' : 'bg-gray-50 dark:bg-gray-800'
      )}
    >
      {/* Avatar */}
      <div
        className={clsx(
          'w-8 h-8 rounded-full flex items-center justify-center text-white text-sm font-medium shrink-0',
          isUser ? 'bg-neuro-500' : 'bg-gray-600 dark:bg-gray-500'
        )}
      >
        {isUser ? 'U' : 'N'}
      </div>

      {/* Content */}
      <div className="flex-1 min-w-0">
        <div className="flex items-center gap-2 mb-1">
          <span className="font-medium text-sm">
            {isUser ? 'You' : 'NEURO-OS'}
          </span>
          <span className="text-xs text-gray-500">
            {new Date(message.createdAt).toLocaleTimeString()}
          </span>
        </div>
        
        <div className="prose prose-sm dark:prose-invert max-w-none">
          <ReactMarkdown>{message.content}</ReactMarkdown>
        </div>

        {/* Token info for assistant messages */}
        {!isUser && message.tokensPrompt !== undefined && (
          <div className="mt-2 text-xs text-gray-500 flex gap-3">
            {message.model && <span>Model: {message.model}</span>}
            <span>Tokens: {message.tokensPrompt} + {message.tokensCompletion}</span>
          </div>
        )}
      </div>
    </div>
  );
}
