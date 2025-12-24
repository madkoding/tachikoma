import { Checklist } from '../../stores/checklistStore';
import clsx from 'clsx';

interface ChecklistCardProps {
  readonly checklist: Checklist;
  readonly isSelected: boolean;
  readonly onClick: () => void;
  readonly isDragging?: boolean;
  readonly isDragOver?: boolean;
}

export default function ChecklistCard({ 
  checklist, 
  isSelected, 
  onClick,
  isDragging = false,
  isDragOver = false,
}: ChecklistCardProps) {
  const completedCount = checklist.items.filter((item) => item.isCompleted).length;
  const totalCount = checklist.items.length;
  const progress = totalCount > 0 ? (completedCount / totalCount) * 100 : 0;

  const getPriorityColor = (priority: number) => {
    switch (priority) {
      case 5:
        return 'bg-red-500/20 text-red-400 border-red-500/30';
      case 4:
        return 'bg-orange-500/20 text-orange-400 border-orange-500/30';
      case 3:
        return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/30';
      case 2:
        return 'bg-green-500/20 text-green-400 border-green-500/30';
      default:
        return 'bg-cyber-cyan/20 text-cyber-cyan border-cyber-cyan/30';
    }
  };

  return (
    <button
      onClick={onClick}
      className={clsx(
        'w-full text-left p-4 rounded-xl border transition-all',
        isSelected
          ? 'bg-cyber-cyan/10 border-cyber-cyan/50 shadow-[0_0_15px_rgba(0,245,255,0.15)]'
          : 'bg-cyber-bg/50 border-cyber-cyan/20 hover:border-cyber-cyan/40 hover:bg-cyber-cyan/5',
        checklist.isArchived && 'opacity-60',
        isDragging && 'opacity-50 scale-95',
        isDragOver && 'border-cyber-cyan border-dashed bg-cyber-cyan/10'
      )}
    >
      {/* Drag handle indicator */}
      <div className="flex items-start justify-between mb-2">
        <div className="flex items-center gap-2">
          <div className="flex flex-col gap-0.5 opacity-30 hover:opacity-60 transition-opacity cursor-grab">
            <div className="flex gap-0.5">
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
            </div>
            <div className="flex gap-0.5">
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
            </div>
            <div className="flex gap-0.5">
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
              <div className="w-1 h-1 rounded-full bg-cyber-cyan" />
            </div>
          </div>
          <h3 className="font-semibold text-cyber-cyan truncate pr-2">{checklist.title}</h3>
        </div>
        <span
          className={clsx(
            'text-xs px-2 py-0.5 rounded-full border shrink-0',
            getPriorityColor(checklist.priority)
          )}
        >
          P{checklist.priority}
        </span>
      </div>

      {checklist.description && (
        <p className="text-sm text-cyber-cyan/50 line-clamp-2 mb-3">{checklist.description}</p>
      )}

      {/* Progress bar */}
      <div className="mb-2">
        <div className="h-1.5 bg-cyber-cyan/10 rounded-full overflow-hidden">
          <div
            className="h-full bg-cyber-cyan transition-all duration-300"
            style={{ width: `${progress}%` }}
          />
        </div>
      </div>

      {/* Stats */}
      <div className="flex items-center justify-between text-xs text-cyber-cyan/50">
        <span>
          {completedCount}/{totalCount} items
        </span>
        {checklist.dueDate && (
          <span className="flex items-center gap-1">
            <CalendarIcon />
            {new Date(checklist.dueDate).toLocaleDateString()}
          </span>
        )}
      </div>
    </button>
  );
}

function CalendarIcon() {
  return (
    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z"
      />
    </svg>
  );
}
