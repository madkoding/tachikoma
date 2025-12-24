import { useState, useRef, useEffect } from 'react';
import { ChecklistItem } from '../../stores/checklistStore';
import ReactMarkdown, { Components } from 'react-markdown';
import TypewriterText from '../common/TypewriterText';
import clsx from 'clsx';

// Markdown components for checklist items
const markdownComponents: Partial<Components> = {
  p: ({ children }) => <span className="text-cyber-cyan">{children}</span>,
  strong: ({ children }) => <strong className="text-cyber-yellow font-bold">{children}</strong>,
  em: ({ children }) => <em className="text-cyber-magenta italic">{children}</em>,
  code: ({ children }) => <code className="bg-cyber-cyan/10 text-cyber-cyan px-1 rounded text-xs font-mono">{children}</code>,
  a: ({ href, children }) => <a href={href} className="text-cyber-cyan underline" target="_blank" rel="noopener noreferrer">{children}</a>,
};

interface ChecklistItemRowProps {
  readonly item: ChecklistItem;
  readonly onToggle: () => void;
  readonly onDelete: () => void;
  readonly onUpdate: (content: string) => void;
  readonly isDragging?: boolean;
  readonly isDragOver?: boolean;
  readonly onDragStart?: (e: React.DragEvent) => void;
  readonly onDragEnd?: (e: React.DragEvent) => void;
  readonly onDragOver?: (e: React.DragEvent) => void;
  readonly onDragLeave?: () => void;
  readonly onDrop?: (e: React.DragEvent) => void;
}

export default function ChecklistItemRow({ 
  item, 
  onToggle, 
  onDelete,
  onUpdate,
  isDragging = false,
  isDragOver = false,
  onDragStart,
  onDragEnd,
  onDragOver,
  onDragLeave,
  onDrop,
}: ChecklistItemRowProps) {
  const [showDelete, setShowDelete] = useState(false);
  const [isEditing, setIsEditing] = useState(false);
  const [editContent, setEditContent] = useState(item.content);
  const [typewriterComplete, setTypewriterComplete] = useState(false);
  const inputRef = useRef<HTMLTextAreaElement>(null);

  useEffect(() => {
    if (isEditing && inputRef.current) {
      inputRef.current.focus();
      inputRef.current.select();
    }
  }, [isEditing]);

  // Reset typewriter when content changes
  useEffect(() => {
    setTypewriterComplete(false);
  }, [item.content]);

  const handleSave = () => {
    const trimmed = editContent.trim();
    if (trimmed && trimmed !== item.content) {
      onUpdate(trimmed);
    } else {
      setEditContent(item.content);
    }
    setIsEditing(false);
  };

  const handleKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSave();
    } else if (e.key === 'Escape') {
      setEditContent(item.content);
      setIsEditing(false);
    }
  };

  const handleContentClick = () => {
    setIsEditing(true);
  };

  const handleContentKeyDown = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      setIsEditing(true);
    }
  };

  return (
    <li
      draggable={!isEditing}
      onDragStart={onDragStart}
      onDragEnd={onDragEnd}
      onDragOver={onDragOver}
      onDragLeave={onDragLeave}
      onDrop={onDrop}
      className={clsx(
        'group flex items-start gap-3 p-3 rounded-lg border transition-all list-none',
        item.isCompleted
          ? 'bg-cyber-cyan/5 border-cyber-cyan/10'
          : 'bg-cyber-surface border-cyber-cyan/20 hover:border-cyber-cyan/30',
        isDragging && 'opacity-50 scale-95',
        isDragOver && 'border-cyber-cyan border-dashed bg-cyber-cyan/10 transform translate-y-0.5'
      )}
      onMouseEnter={() => setShowDelete(true)}
      onMouseLeave={() => setShowDelete(false)}
    >
      {/* Drag handle */}
      <div className="flex flex-col gap-0.5 opacity-30 hover:opacity-60 transition-opacity cursor-grab shrink-0 mt-0.5">
        <div className="flex gap-0.5">
          <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
          <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
        </div>
        <div className="flex gap-0.5">
          <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
          <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
        </div>
      </div>

      {/* Checkbox */}
      <button
        onClick={onToggle}
        className={clsx(
          'w-5 h-5 rounded border-2 flex items-center justify-center transition-all shrink-0 mt-0.5',
          item.isCompleted
            ? 'bg-cyber-cyan border-cyber-cyan'
            : 'border-cyber-cyan/40 hover:border-cyber-cyan'
        )}
      >
        {item.isCompleted && (
          <svg className="w-3 h-3 text-cyber-bg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
          </svg>
        )}
      </button>

      {/* Content */}
      {isEditing ? (
        <textarea
          ref={inputRef}
          value={editContent}
          onChange={(e) => setEditContent(e.target.value)}
          onBlur={handleSave}
          onKeyDown={handleKeyDown}
          className="flex-1 bg-cyber-bg/50 text-cyber-cyan text-sm px-2 py-1 rounded border border-cyber-cyan/30 focus:outline-none focus:border-cyber-cyan resize-none min-h-[32px]"
          rows={editContent.split('\n').length || 1}
        />
      ) : (
        <button
          type="button"
          onClick={handleContentClick}
          onKeyDown={handleContentKeyDown}
          className={clsx(
            'flex-1 text-sm transition-all cursor-text min-w-0 text-left bg-transparent border-none p-0',
            item.isCompleted ? 'opacity-40 line-through' : '',
            'prose prose-sm prose-invert max-w-none',
            '[&_p]:m-0 [&_p]:leading-relaxed [&_p]:text-cyber-cyan',
            '[&_code]:bg-cyber-cyan/10 [&_code]:px-1 [&_code]:rounded [&_code]:text-cyber-cyan',
            '[&_strong]:text-cyber-cyan [&_strong]:font-semibold',
            '[&_em]:text-cyber-cyan/80',
            '[&_a]:text-cyber-cyan [&_a]:underline',
          )}
        >
          {typewriterComplete ? (
            <ReactMarkdown components={markdownComponents}>
              {item.content}
            </ReactMarkdown>
          ) : (
            <TypewriterText 
              text={item.content} 
              speed={10} 
              onComplete={() => setTypewriterComplete(true)}
            />
          )}
        </button>
      )}

      {/* Delete button */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          onDelete();
        }}
        className={clsx(
          'p-1 text-red-400/50 hover:text-red-400 rounded transition-all shrink-0',
          showDelete ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
        )}
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </li>
  );
}
