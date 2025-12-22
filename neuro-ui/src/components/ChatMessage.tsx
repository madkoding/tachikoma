import { Message } from '../stores/chatStore';
import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import ReactMarkdown from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface ChatMessageProps {
  readonly message: Message;
}

// Custom cyberpunk theme based on oneDark
const cyberpunkTheme = {
  ...oneDark,
  'pre[class*="language-"]': {
    ...oneDark['pre[class*="language-"]'],
    background: 'rgba(0, 20, 40, 0.8)',
    border: '1px solid rgba(0, 255, 255, 0.2)',
    borderRadius: '0.5rem',
    margin: '0.5rem 0',
  },
  'code[class*="language-"]': {
    ...oneDark['code[class*="language-"]'],
    background: 'transparent',
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    fontSize: '0.875rem',
  },
};

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
        
        <div className="prose prose-sm max-w-none prose-invert">
          <ReactMarkdown
            components={{
              // eslint-disable-next-line @typescript-eslint/no-unused-vars
              code({ node, className, children, ...props }) {
                const match = /language-(\w+)/.exec(className || '');
                const isInline = !match && !className;
                const codeString = String(children).replace(/\n$/, '');
                
                if (isInline) {
                  return (
                    <code 
                      className="bg-cyber-cyan/10 text-cyber-cyan px-1.5 py-0.5 rounded text-sm font-mono border border-cyber-cyan/20" 
                      {...props}
                    >
                      {children}
                    </code>
                  );
                }
                
                return (
                  <SyntaxHighlighter
                    style={cyberpunkTheme}
                    language={match ? match[1] : 'text'}
                    PreTag="div"
                    customStyle={{
                      margin: '0.75rem 0',
                      padding: '1rem',
                      borderRadius: '0.5rem',
                      background: 'rgba(0, 20, 40, 0.8)',
                      border: '1px solid rgba(0, 255, 255, 0.2)',
                    }}
                    codeTagProps={{
                      style: {
                        fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
                        fontSize: '0.875rem',
                      }
                    }}
                  >
                    {codeString}
                  </SyntaxHighlighter>
                );
              },
              p: ({ children }) => <p className="text-cyber-cyan/80 mb-2">{children}</p>,
              ul: ({ children }) => <ul className="text-cyber-cyan/80 list-disc list-inside mb-2">{children}</ul>,
              ol: ({ children }) => <ol className="text-cyber-cyan/80 list-decimal list-inside mb-2">{children}</ol>,
              li: ({ children }) => <li className="text-cyber-cyan/80">{children}</li>,
              h1: ({ children }) => <h1 className="text-cyber-cyan font-bold text-xl mb-2">{children}</h1>,
              h2: ({ children }) => <h2 className="text-cyber-cyan font-bold text-lg mb-2">{children}</h2>,
              h3: ({ children }) => <h3 className="text-cyber-cyan font-bold text-md mb-2">{children}</h3>,
              strong: ({ children }) => <strong className="text-cyber-yellow font-bold">{children}</strong>,
              em: ({ children }) => <em className="text-cyber-magenta italic">{children}</em>,
              a: ({ href, children }) => (
                <a href={href} className="text-cyber-cyan underline hover:text-cyber-magenta" target="_blank" rel="noopener noreferrer">
                  {children}
                </a>
              ),
              blockquote: ({ children }) => (
                <blockquote className="border-l-2 border-cyber-cyan/50 pl-4 italic text-cyber-cyan/60 my-2">
                  {children}
                </blockquote>
              ),
            }}
          >
            {message.content}
          </ReactMarkdown>
        </div>

        {/* Token info for assistant messages */}
        {!isUser && (message.model || message.tokensPrompt !== undefined || message.processingTimeMs !== undefined) && (
          <div className="mt-2 text-xs text-cyber-cyan/50 flex flex-wrap gap-3 font-mono">
            {message.model && <span>{t('message.model')}: <span className="text-cyber-green">{message.model}</span></span>}
            {(message.tokensPrompt !== undefined || message.tokensCompletion !== undefined) && (
              <span>{t('message.tokens')}: <span className="text-cyber-yellow">{message.tokensPrompt ?? 0} + {message.tokensCompletion ?? 0}</span></span>
            )}
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
