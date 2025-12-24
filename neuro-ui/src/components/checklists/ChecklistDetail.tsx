import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { Checklist, useChecklistStore, ChecklistItem } from '../../stores/checklistStore';
import ChecklistItemRow from './ChecklistItemRow';
import clsx from 'clsx';

interface ChecklistDetailProps {
  readonly checklist: Checklist;
  readonly onBack: () => void;
}

// Helper to generate UUID
function generateUUID(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replaceAll(/[xy]/g, (c) => {
    const r = Math.trunc(Math.random() * 16);
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export default function ChecklistDetail({ checklist, onBack }: ChecklistDetailProps) {
  const { t } = useTranslation();
  const { updateChecklist, deleteChecklist, addItem, deleteItem, toggleItem } = useChecklistStore();
  const [newItemContent, setNewItemContent] = useState('');
  const [isEditing, setIsEditing] = useState(false);
  const [editTitle, setEditTitle] = useState(checklist.title);
  const [editDescription, setEditDescription] = useState(checklist.description || '');
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  const completedCount = checklist.items.filter((item) => item.isCompleted).length;
  const totalCount = checklist.items.length;
  const progress = totalCount > 0 ? (completedCount / totalCount) * 100 : 0;

  const handleAddItem = (e: React.FormEvent) => {
    e.preventDefault();
    if (!newItemContent.trim()) return;

    const newItem: ChecklistItem = {
      id: generateUUID(),
      content: newItemContent.trim(),
      isCompleted: false,
      order: checklist.items.length,
      createdAt: new Date(),
    };

    addItem(checklist.id, newItem);
    setNewItemContent('');
  };

  const handleSaveEdit = () => {
    updateChecklist(checklist.id, {
      title: editTitle,
      description: editDescription || undefined,
    });
    setIsEditing(false);
  };

  const handleDelete = () => {
    deleteChecklist(checklist.id);
    onBack();
  };

  const handleArchiveToggle = () => {
    updateChecklist(checklist.id, { isArchived: !checklist.isArchived });
  };

  return (
    <div className="flex flex-col h-full bg-cyber-bg">
      {/* Header */}
      <div className="p-4 border-b border-cyber-cyan/20 bg-cyber-surface">
        <div className="flex items-center gap-4 mb-4">
          <button
            onClick={onBack}
            className="lg:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all"
          >
            <BackIcon />
          </button>

          {isEditing ? (
            <input
              type="text"
              value={editTitle}
              onChange={(e) => setEditTitle(e.target.value)}
              className="flex-1 bg-transparent text-xl font-cyber font-bold text-cyber-cyan border-b border-cyber-cyan/50 focus:outline-none focus:border-cyber-cyan"
            />
          ) : (
            <h2 className="flex-1 text-xl font-cyber font-bold text-cyber-cyan truncate">
              {checklist.title}
            </h2>
          )}

          <div className="flex items-center gap-2">
            {isEditing ? (
              <>
                <button
                  onClick={handleSaveEdit}
                  className="p-2 text-green-400 hover:bg-green-400/10 rounded-lg transition-all"
                >
                  <CheckIcon />
                </button>
                <button
                  onClick={() => {
                    setIsEditing(false);
                    setEditTitle(checklist.title);
                    setEditDescription(checklist.description || '');
                  }}
                  className="p-2 text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                >
                  <CloseIcon />
                </button>
              </>
            ) : (
              <>
                <button
                  onClick={() => setIsEditing(true)}
                  className="p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all"
                  title={t('common.edit')}
                >
                  <EditIcon />
                </button>
                <button
                  onClick={handleArchiveToggle}
                  className={clsx(
                    'p-2 rounded-lg transition-all',
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
                  className="p-2 text-red-400/70 hover:text-red-400 hover:bg-red-400/10 rounded-lg transition-all"
                  title={t('common.delete')}
                >
                  <TrashIcon />
                </button>
              </>
            )}
          </div>
        </div>

        {isEditing ? (
          <textarea
            value={editDescription}
            onChange={(e) => setEditDescription(e.target.value)}
            placeholder={t('checklists.descriptionPlaceholder')}
            className="w-full bg-cyber-bg/50 text-sm text-cyber-cyan/70 p-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 resize-none"
            rows={2}
          />
        ) : (
          checklist.description && (
            <p className="text-sm text-cyber-cyan/50">{checklist.description}</p>
          )
        )}

        {/* Progress */}
        <div className="mt-4">
          <div className="flex items-center justify-between text-xs text-cyber-cyan/50 mb-1">
            <span>{t('checklists.progress')}</span>
            <span>
              {completedCount}/{totalCount} ({Math.round(progress)}%)
            </span>
          </div>
          <div className="h-2 bg-cyber-cyan/10 rounded-full overflow-hidden">
            <div
              className="h-full bg-gradient-to-r from-cyber-cyan to-cyber-cyan/70 transition-all duration-300"
              style={{ width: `${progress}%` }}
            />
          </div>
        </div>
      </div>

      {/* Items List */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="space-y-2">
          {checklist.items
            .sort((a, b) => a.order - b.order)
            .map((item) => (
              <ChecklistItemRow
                key={item.id}
                item={item}
                onToggle={() => toggleItem(checklist.id, item.id)}
                onDelete={() => deleteItem(checklist.id, item.id)}
              />
            ))}
        </div>

        {checklist.items.length === 0 && (
          <div className="text-center py-12 text-cyber-cyan/40">
            <p className="font-mono text-sm">{t('checklists.noItems')}</p>
          </div>
        )}
      </div>

      {/* Add Item Form */}
      <div className="p-4 border-t border-cyber-cyan/20 bg-cyber-surface">
        <form onSubmit={handleAddItem} className="flex gap-2">
          <input
            type="text"
            value={newItemContent}
            onChange={(e) => setNewItemContent(e.target.value)}
            placeholder={t('checklists.addItemPlaceholder')}
            className="flex-1 bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30"
          />
          <button
            type="submit"
            disabled={!newItemContent.trim()}
            className="cyber-button disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <PlusIcon />
          </button>
        </form>
      </div>

      {/* Delete Confirmation Modal */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50">
          <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-6 max-w-sm mx-4">
            <h3 className="text-lg font-cyber font-bold text-cyber-cyan mb-2">
              {t('checklists.deleteConfirm.title')}
            </h3>
            <p className="text-cyber-cyan/70 text-sm mb-6">
              {t('checklists.deleteConfirm.message')}
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
