import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useChecklistStore, Checklist } from '../../stores/checklistStore';

interface CreateChecklistModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
}

// Helper to generate UUID
function generateUUID(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c: string) => {
    const r = Math.trunc(Math.random() * 16);
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

export default function CreateChecklistModal({ isOpen, onClose }: CreateChecklistModalProps) {
  const { t } = useTranslation();
  const { addChecklist, setSelectedChecklist } = useChecklistStore();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [priority, setPriority] = useState<1 | 2 | 3 | 4 | 5>(3);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;

    const now = new Date();
    const newChecklist: Checklist = {
      id: generateUUID(),
      title: title.trim(),
      description: description.trim() || undefined,
      items: [],
      priority,
      order: 0, // New checklists go to the top
      isArchived: false,
      createdAt: now,
      updatedAt: now,
    };

    addChecklist(newChecklist);
    setSelectedChecklist(newChecklist.id);
    handleClose();
  };

  const handleClose = () => {
    setTitle('');
    setDescription('');
    setPriority(3);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-6 w-full max-w-md">
        <h2 className="text-xl font-cyber font-bold text-cyber-cyan mb-6">
          {t('checklists.createTitle')}
        </h2>

        <form onSubmit={handleSubmit} className="space-y-4">
          {/* Title */}
          <div>
            <label className="block text-sm font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.titleLabel')} *
            </label>
            <input
              type="text"
              value={title}
              onChange={(e) => setTitle(e.target.value)}
              placeholder={t('checklists.titlePlaceholder')}
              className="w-full bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30"
              autoFocus
            />
          </div>

          {/* Description */}
          <div>
            <label className="block text-sm font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.descriptionLabel')}
            </label>
            <textarea
              value={description}
              onChange={(e) => setDescription(e.target.value)}
              placeholder={t('checklists.descriptionPlaceholder')}
              className="w-full bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30 resize-none"
              rows={3}
            />
          </div>

          {/* Priority */}
          <div>
            <label className="block text-sm font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.priorityLabel')}
            </label>
            <div className="flex gap-2">
              {([1, 2, 3, 4, 5] as const).map((p) => (
                <button
                  key={p}
                  type="button"
                  onClick={() => setPriority(p)}
                  className={`flex-1 py-2 rounded-lg border text-sm font-mono transition-all ${
                    priority === p
                      ? getPriorityActiveClass(p)
                      : 'border-cyber-cyan/20 text-cyber-cyan/50 hover:border-cyber-cyan/40'
                  }`}
                >
                  P{p}
                </button>
              ))}
            </div>
          </div>

          {/* Actions */}
          <div className="flex gap-3 pt-4">
            <button
              type="button"
              onClick={handleClose}
              className="flex-1 px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan border border-cyber-cyan/30 hover:border-cyber-cyan/50 rounded-lg transition-all"
            >
              {t('common.cancel')}
            </button>
            <button
              type="submit"
              disabled={!title.trim()}
              className="flex-1 cyber-button disabled:opacity-50 disabled:cursor-not-allowed"
            >
              {t('checklists.create')}
            </button>
          </div>
        </form>
      </div>
    </div>
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
