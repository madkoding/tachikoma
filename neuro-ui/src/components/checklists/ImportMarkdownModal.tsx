import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useChecklistStore } from '../../stores/checklistStore';

interface ImportMarkdownModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
}

export default function ImportMarkdownModal({ isOpen, onClose }: ImportMarkdownModalProps) {
  const { t } = useTranslation();
  const { importFromMarkdown, setSelectedChecklist } = useChecklistStore();
  const [markdown, setMarkdown] = useState('');
  const [customTitle, setCustomTitle] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleImport = () => {
    if (!markdown.trim()) {
      setError(t('checklists.import.errorEmpty'));
      return;
    }

    // Check if there are any checkbox items
    const hasCheckboxes = /^[-*]\s*\[[ xX]\]/m.test(markdown);
    if (!hasCheckboxes) {
      setError(t('checklists.import.errorNoCheckboxes'));
      return;
    }

    try {
      const newChecklist = importFromMarkdown(markdown, customTitle || undefined);
      setSelectedChecklist(newChecklist.id);
      handleClose();
    } catch {
      setError(t('checklists.import.errorParsing'));
    }
  };

  const handleClose = () => {
    setMarkdown('');
    setCustomTitle('');
    setError(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-6 w-full max-w-lg">
        <h2 className="text-xl font-cyber font-bold text-cyber-cyan mb-2">
          {t('checklists.import.title')}
        </h2>
        <p className="text-sm text-cyber-cyan/50 mb-6">
          {t('checklists.import.description')}
        </p>

        <div className="space-y-4">
          {/* Custom Title (optional) */}
          <div>
            <label className="block text-sm font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.import.customTitle')}
            </label>
            <input
              type="text"
              value={customTitle}
              onChange={(e) => setCustomTitle(e.target.value)}
              placeholder={t('checklists.import.customTitlePlaceholder')}
              className="w-full bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30"
            />
          </div>

          {/* Markdown Input */}
          <div>
            <label className="block text-sm font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.import.markdownLabel')}
            </label>
            <textarea
              value={markdown}
              onChange={(e) => {
                setMarkdown(e.target.value);
                setError(null);
              }}
              placeholder={`# My Checklist\n- [ ] First item\n- [ ] Second item\n- [x] Completed item`}
              className="w-full bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30 font-mono text-sm resize-none"
              rows={10}
            />
          </div>

          {/* Error message */}
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          )}

          {/* Format Help */}
          <div className="p-3 bg-cyber-cyan/5 border border-cyber-cyan/20 rounded-lg">
            <p className="text-xs font-mono text-cyber-cyan/70 mb-2">
              {t('checklists.import.formatHelp')}:
            </p>
            <pre className="text-xs text-cyber-cyan/50 font-mono">
              {`- [ ] Unchecked item\n- [x] Checked item\n* [ ] Also works with asterisks`}
            </pre>
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
              onClick={handleImport}
              disabled={!markdown.trim()}
              className="flex-1 cyber-button disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              <ImportIcon />
              {t('checklists.import.button')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}

function ImportIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={2}
        d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"
      />
    </svg>
  );
}
