import { useState } from 'react';
import { ChecklistItem } from '../../stores/checklistStore';
import clsx from 'clsx';

interface ChecklistItemRowProps {
  readonly item: ChecklistItem;
  readonly onToggle: () => void;
  readonly onDelete: () => void;
}

export default function ChecklistItemRow({ item, onToggle, onDelete }: ChecklistItemRowProps) {
  const [showDelete, setShowDelete] = useState(false);

  return (
    <div
      className={clsx(
        'group flex items-center gap-3 p-3 rounded-lg border transition-all',
        item.isCompleted
          ? 'bg-cyber-cyan/5 border-cyber-cyan/10'
          : 'bg-cyber-surface border-cyber-cyan/20 hover:border-cyber-cyan/30'
      )}
      onMouseEnter={() => setShowDelete(true)}
      onMouseLeave={() => setShowDelete(false)}
    >
      {/* Checkbox */}
      <button
        onClick={onToggle}
        className={clsx(
          'w-5 h-5 rounded border-2 flex items-center justify-center transition-all shrink-0',
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
      <span
        className={clsx(
          'flex-1 text-sm transition-all',
          item.isCompleted ? 'text-cyber-cyan/40 line-through' : 'text-cyber-cyan'
        )}
      >
        {item.content}
      </span>

      {/* Delete button */}
      <button
        onClick={onDelete}
        className={clsx(
          'p-1 text-red-400/50 hover:text-red-400 rounded transition-all',
          showDelete ? 'opacity-100' : 'opacity-0 group-hover:opacity-100'
        )}
      >
        <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
        </svg>
      </button>
    </div>
  );
}
