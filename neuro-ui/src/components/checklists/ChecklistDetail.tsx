import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useLocation } from 'react-router-dom';
import { Checklist, useChecklistStore } from '../../stores/checklistStore';
import { useMusicStore } from '../../stores/musicStore';
import ChecklistItemRow from './ChecklistItemRow';
import TypewriterText from '../common/TypewriterText';
import clsx from 'clsx';

interface ChecklistDetailProps {
  readonly checklist: Checklist;
  readonly onBack: () => void;
}

type Priority = 1 | 2 | 3 | 4 | 5;

export default function ChecklistDetail({ checklist, onBack }: ChecklistDetailProps) {
  const { t } = useTranslation();
  const location = useLocation();
  const { updateChecklist, deleteChecklist, addItem, deleteItem, toggleItem, updateItem, reorderItems } = useChecklistStore();
  const currentSong = useMusicStore(state => state.player.currentSong);
  const showMiniPlayerPadding = currentSong && location.pathname !== '/music';
  const [newItemContent, setNewItemContent] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(checklist.title);
  const [editDescription, setEditDescription] = useState(checklist.description || '');
  const [editPriority, setEditPriority] = useState<Priority>(checklist.priority as Priority);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);
  const [draggedItemId, setDraggedItemId] = useState<string | null>(null);
  const [dragOverItemId, setDragOverItemId] = useState<string | null>(null);

  const completedCount = checklist.items.filter((item) => item.isCompleted).length;
  const totalCount = checklist.items.length;
  const progress = totalCount > 0 ? (completedCount / totalCount) * 100 : 0;

  const handleAddItem = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newItemContent.trim()) return;

    try {
      await addItem(checklist.id, newItemContent.trim());
      setNewItemContent('');
    } catch (error) {
      console.error('Failed to add item:', error);
    }
  };

  const handleSaveEdit = async () => {
    try {
      await updateChecklist(checklist.id, {
        title: editTitle,
        description: editDescription || undefined,
        priority: editPriority,
      });
      setIsEditing(false);
    } catch (error) {
      console.error('Failed to save checklist:', error);
    }
  };

  const handleDelete = async () => {
    try {
      await deleteChecklist(checklist.id);
      onBack();
    } catch (error) {
      console.error('Failed to delete checklist:', error);
    }
  };

  const handleArchiveToggle = async () => {
    try {
      await updateChecklist(checklist.id, { isArchived: !checklist.isArchived });
    } catch (error) {
      console.error('Failed to archive checklist:', error);
    }
  };

  // Item drag and drop handlers
  const handleItemDragStart = (itemId: string) => (e: React.DragEvent) => {
    setDraggedItemId(itemId);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', itemId);
  };

  const handleItemDragEnd = () => {
    setDraggedItemId(null);
    setDragOverItemId(null);
  };

  const handleItemDragOver = (itemId: string) => (e: React.DragEvent) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    if (itemId !== draggedItemId) {
      setDragOverItemId(itemId);
    }
  };

  const handleItemDragLeave = () => {
    setDragOverItemId(null);
  };

  const handleItemDrop = (targetItemId: string) => (e: React.DragEvent) => {
    e.preventDefault();
    if (!draggedItemId || draggedItemId === targetItemId) {
      setDraggedItemId(null);
      setDragOverItemId(null);
      return;
    }

    const sortedItems = [...checklist.items].sort((a, b) => a.order - b.order);
    const draggedIndex = sortedItems.findIndex((item) => item.id === draggedItemId);
    const targetIndex = sortedItems.findIndex((item) => item.id === targetItemId);

    if (draggedIndex === -1 || targetIndex === -1) return;

    // Remove dragged item and insert at target position
    const [draggedItem] = sortedItems.splice(draggedIndex, 1);
    sortedItems.splice(targetIndex, 0, draggedItem);

    // Update order values
    const reorderedItems = sortedItems.map((item, index) => ({
      ...item,
      order: index,
    }));

    reorderItems(checklist.id, reorderedItems);
    setDraggedItemId(null);
    setDragOverItemId(null);
  };

  const handleItemUpdate = (itemId: string) => async (content: string) => {
    try {
      await updateItem(checklist.id, itemId, { content });
    } catch (error) {
      console.error('Failed to update item:', error);
    }
  };

  return (
    <div className="flex flex-col h-full bg-cyber-bg overflow-hidden">
      {/* Header */}
      <div className="p-2 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface flex-shrink-0">
        <div className="flex items-center gap-2 sm:gap-4 mb-2 sm:mb-4">
          <button
            onClick={onBack}
            className="md:hidden p-1 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all flex-shrink-0"
          >
            <BackIcon />
          </button>

          {isEditing ? (
            <input
              type="text"
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              className="flex-1 bg-transparent text-base sm:text-xl font-cyber font-bold text-cyber-cyan border-b border-cyber-cyan/50 focus:outline-none focus:border-cyber-cyan min-w-0"
            />
          ) : (
            <h2 className="flex-1 text-base sm:text-xl font-cyber font-bold text-cyber-cyan truncate">
              <TypewriterText text={checklist.title} speed={18} />
            </h2>
          )}

          <div className="flex items-center gap-1 sm:gap-2 flex-shrink-0">
            {isEditing ? (
              <>
                <button
                  onClick={handleSaveEdit}
                  className="p-1.5 sm:p-2 text-green-400 hover:bg-green-400/10 rounded-lg transition-all"
                >
                  <CheckIcon />
                </button>
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setEditTitle(checklist.title);
                    setEditDescription(checklist.description || '');
                    setEditPriority(checklist.priority as Priority);
                  }}
                  className="p-1.5 sm:p-2 text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                >
                  <CloseIcon />
                </button>
              </>
            ) : (
              <>
                <button
                  onClick={() => setIsEditing(true)}
                  className="p-1.5 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all"
                  title={t('common.edit')}
                >
                  <EditIcon />
                </button>
                <button
                  onClick={handleArchiveToggle}
                  className={clsx(
                    'p-1.5 sm:p-2 rounded-lg transition-all hidden sm:block',
                    checklist.isArchived
                      ? 'text-yellow-400 hover:bg-yellow-400/10'
                      : 'text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10'
                  )}
                  title={checklist.isArchived ? t('checklists.unarchive') : t('checklists.archive')}
                >
                  <ArchiveIcon />
                </button>
                <button
                  onClick={() => setShowDeleteConfirm(true)}
                  className="p-1.5 sm:p-2 text-red-400/70 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                  title={t('common.delete')}
                >
                  <TrashIcon />
                </button>
              </>
            )}
          </div>
        </div>

        {isEditing ? (
          <>
            <textarea
              value={editDescription}
              onChange={(e) => setEditDescription(e.target.value)}
              placeholder={t('checklists.descriptionPlaceholder')}
              className="w-full bg-cyber-bg/50 text-sm text-cyber-cyan/70 p-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 resize-none"
              rows={2}
            />
            {/* Priority selector */}
            <div className="mt-3">
              <label className="block text-xs font-mono text-cyber-cyan/50 mb-2">
                {t('checklists.priorityLabel')}
              </label>
              <div className="flex gap-1 sm:gap-2">
                {([1, 2, 3, 4, 5] as const).map((p) => (
                  <button
                    key={p}
                    type="button"
                    onClick={() => setEditPriority(p)}
                    className={clsx(
                      'flex-1 py-1 sm:py-1.5 rounded-lg border text-[10px] sm:text-xs font-mono transition-all',
                      editPriority === p
                        ? getPriorityActiveClass(p)
                        : 'border-cyber-cyan/20 text-cyber-cyan/50 hover:border-cyber-cyan/40'
                    )}
                  >
                    {getPriorityLabel(p)}
                  </button>
                ))}
              </div>
            </div>
          </>
        ) : (
          <div className="flex items-center gap-3">
            {checklist.description && (
              <p className="text-xs sm:text-sm text-cyber-cyan/50 flex-1 line-clamp-2">
                <TypewriterText text={checklist.description} speed={10} delay={200} />
              </p>
            )}
            <span
              className={clsx(
                'text-[10px] sm:text-xs px-1.5 sm:px-2 py-0.5 rounded-full border shrink-0',
                getPriorityBadgeClass(checklist.priority)
              )}
            >
              {getPriorityLabel(checklist.priority)}
            </span>
          </div>
        )}

        {/* Progress */}
        <div className="mt-3 sm:mt-4">
          <div className="flex items-center justify-between text-[10px] sm:text-xs text-cyber-cyan/50 mb-1">
            <span><TypewriterText text={t('checklists.progress')} speed={15} /></span>
            <span>
              {completedCount}/{totalCount} ({Math.round(progress)}%)
            </span>
          </div>
          <div className="h-1.5 sm:h-2 bg-cyber-cyan/10 rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-cyber-cyan to-cyber-cyan/70 transition-all duration-300"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      </div>

      {/* Items List */}
      <div className="flex-1 overflow-y-auto p-2 sm:p-4">
        <ul className="space-y-1.5 sm:space-y-2">
          {checklist.items
            .sort((a, b) => a.order - b.order)
            .map((item) => (
              <ChecklistItemRow
                key={item.id}
                item={item}
                onToggle={async () => {
                  try {
                    await toggleItem(checklist.id, item.id);
                  } catch (error) {
                    console.error('Failed to toggle item:', error);
                  }
                }}
                onDelete={async () => {
                  try {
                    await deleteItem(checklist.id, item.id);
                  } catch (error) {
                    console.error('Failed to delete item:', error);
                  }
                }}
                onUpdate={handleItemUpdate(item.id)}
                isDragging={draggedItemId === item.id}
                isDragOver={dragOverItemId === item.id}
                onDragStart={handleItemDragStart(item.id)}
                onDragEnd={handleItemDragEnd}
                onDragOver={handleItemDragOver(item.id)}
                onDragLeave={handleItemDragLeave}
                onDrop={handleItemDrop(item.id)}
              />
            ))}
        </ul>

        {checklist.items.length === 0 && (
          <div className="text-center py-8 sm:py-12 text-cyber-cyan/40">
            <p className="font-mono text-xs sm:text-sm">
              <TypewriterText text={t('checklists.noItems')} speed={12} />
            </p>
          </div>
        )}
      </div>

      {/* Add Item Form */}
      <div className={clsx(
        "p-2 sm:p-4 border-t border-cyber-cyan/20 bg-cyber-surface flex-shrink-0",
        showMiniPlayerPadding && "mb-20 sm:mb-0"
      )}>
        <form onSubmit={handleAddItem} className="flex gap-2">
          <input
            type="text"
            value={newItemContent}
            onChange={(e) => setNewItemContent(e.target.value)}
            placeholder={t('checklists.addItemPlaceholder')}
            className="flex-1 bg-cyber-bg/50 text-cyber-cyan px-2 sm:px-4 py-1.5 sm:py-2 text-xs sm:text-sm rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30 min-w-0"
          />
          <button
            type="submit"
            disabled={!newItemContent.trim()}
            className="cyber-button disabled:opacity-50 disabled:cursor-not-allowed flex-shrink-0 px-2 sm:px-4 py-1.5 sm:py-2 text-xs sm:text-sm"
          >
            <PlusIcon />
          </button>
        </form>
      </div>

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
          <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-4 sm:p-6 max-w-sm w-full">
            <h3 className="text-base sm:text-lg font-cyber font-bold text-cyber-cyan mb-2">
              <TypewriterText text={t('checklists.deleteConfirm.title')} speed={20} />
            </h3>
            <p className="text-cyber-cyan/70 text-sm mb-6">
              <TypewriterText text={t('checklists.deleteConfirm.message')} delay={200} speed={10} />
            </p>
            <div className="flex gap-3 justify-end">
              <button
                onClick={() => setShowDeleteConfirm(false)}
                className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan border border-cyber-cyan/30 hover:border-cyber-cyan/50 rounded-lg transition-all"
              >
                {t('common.cancel')}
              </button>
              <button
                onClick={handleDelete}
                className="px-4 py-2 bg-red-500/20 text-red-400 border border-red-500/30 hover:border-red-500/50 rounded-lg transition-all"
              >
                {t('common.delete')}
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}

// Icons
function BackIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
    </svg>
  );
}

function EditIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M11 5H6a2 2 0 00-2 2v11a2 2 0 002 2h11a2 2 0 002-2v-5m-1.414-9.414a2 2 0 112.828 2.828L11.828 15H9v-2.828l8.586-8.586z"
      />
    </svg>
  );
}

function ArchiveIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M5 8h14M5 8a2 2 0 110-4h14a2 2 0 110 4M5 8v10a2 2 0 002 2h10a2 2 0 002-2V8m-9 4h4"
      />
    </svg>
  );
}

function TrashIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16"
      />
    </svg>
  );
}

function CheckIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
    </svg>
  );
}

function CloseIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
    </svg>
  );
}

function PlusIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
    </svg>
  );
}

function getPriorityActiveClass(priority: number): string {
  switch (priority) {
    case 5:
      return 'bg-red-500/20 text-red-400 border-red-500/50';
    case 4:
      return 'bg-orange-500/20 text-orange-400 border-orange-500/50';
    case 3:
      return 'bg-yellow-500/20 text-yellow-400 border-yellow-500/50';
    case 2:
      return 'bg-green-500/20 text-green-400 border-green-500/50';
    default:
      return 'bg-cyber-cyan/20 text-cyber-cyan border-cyber-cyan/50';
  }
}

function getPriorityBadgeClass(priority: number): string {
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
}

function getPriorityLabel(priority: number): string {
  switch (priority) {
    case 5:
      return 'Urgente';
    case 4:
      return 'Alta';
    case 3:
      return 'Media';
    case 2:
      return 'Baja';
    default:
      return 'Muy baja';
  }
}
