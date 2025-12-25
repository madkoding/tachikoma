import { memo, useRef, useEffect } from 'react';
import ReactMarkdown, { Components } from 'react-markdown';
import { Prism as SyntaxHighlighter } from 'react-syntax-highlighter';
import { oneDark } from 'react-syntax-highlighter/dist/esm/styles/prism';

interface StreamingTextProps {
  readonly content: string;
  readonly isStreaming: boolean;
}

// Custom cyberpunk theme
const cyberpunkTheme = {
  ...oneDark,
  'pre[class*="language-"]': {
    ...oneDark['pre[class*="language-"]'],
    background: 'rgba(0, 20, 40, 0.8)',
    border: '1px solid rgba(0, 255, 255, 0.2)',
    borderRadius: '5px',
    margin: '0.5rem 0',
  },
  'code[class*="language-"]': {
    ...oneDark['code[class*="language-"]'],
    background: 'transparent',
    fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
    fontSize: '0.875rem',
  },
};

// Static markdown components (no streaming effects)
const staticComponents: Partial<Components> = {
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  code({ className, children, ...props }: any) {
    const match = /language-(\w+)/.exec(className || '');
    const isInline = !match && !className;
    const codeContent = Array.isArray(children) ? children.join('') : String(children || '');
    const codeString = codeContent.replace(/\n$/, '');
    
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
      <div className="code-block-container">
        <SyntaxHighlighter
          style={cyberpunkTheme}
          language={match ? match[1] : 'text'}
          PreTag="div"
          customStyle={{
            margin: '0.75rem 0',
            padding: '1rem',
            borderRadius: '5px',
            background: 'rgba(0, 20, 40, 0.8)',
            border: '1px solid rgba(0, 255, 255, 0.2)',
          }}
          codeTagProps={{
            style: {
              fontFamily: "'JetBrains Mono', 'Fira Code', monospace",
              fontSize: '0.875rem',
            },
          }}
        >
          {codeString}
        </SyntaxHighlighter>
      </div>
    );
  },
  p: ({ children }) => <p className="text-cyber-cyan/80 mb-2 leading-relaxed">{children}</p>,
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
};

function StreamingText({ content, isStreaming }: Readonly<StreamingTextProps>) {
  const containerRef = useRef<HTMLDivElement>(null);
  const prevLengthRef = useRef(0);

  // Apply glow effect to new characters
  useEffect(() => {
    if (!isStreaming || !containerRef.current) return;
    
    const currentLength = content.length;
    const prevLength = prevLengthRef.current;
    
    // Only animate if content has grown
    if (currentLength > prevLength) {
      // Add streaming-active class to trigger CSS animation
      containerRef.current.classList.add('streaming-active');
      
      // Remove class after animation duration
      const timer = setTimeout(() => {
        containerRef.current?.classList.remove('streaming-active');
      }, 300);
      
      prevLengthRef.current = currentLength;
      return () => clearTimeout(timer);
    }
    
    prevLengthRef.current = currentLength;
  }, [content, isStreaming]);

  // Reset prev length when streaming stops
  useEffect(() => {
    if (!isStreaming) {
      prevLengthRef.current = 0;
    }
  }, [isStreaming]);

  return (
    <div 
      ref={containerRef}
      className={isStreaming ? 'streaming-content' : ''}
    >
      <ReactMarkdown components={staticComponents}>
        {content}
      </ReactMarkdown>
    </div>
  );
}

export default memo(StreamingText);