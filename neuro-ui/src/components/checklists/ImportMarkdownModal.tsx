import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useChecklistStore } from '../../stores/checklistStore';
import TypewriterText from '../common/TypewriterText';

interface ImportMarkdownModalProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
}

type ImportMode = 'single' | 'multiple';

export default function ImportMarkdownModal({ isOpen, onClose }: ImportMarkdownModalProps) {
  const { t } = useTranslation();
  const { importFromMarkdown, importMultipleFromMarkdown, setSelectedChecklist } = useChecklistStore();
  const [markdown, setMarkdown] = useState('');
  const [customTitle, setCustomTitle] = useState('');
  const [importMode, setImportMode] = useState<ImportMode>('multiple');
  const [error, setError] = useState<string | null>(null);
  const [importResult, setImportResult] = useState<{ count: number } | null>(null);

  // Detect if markdown has multiple sections
  const hasSections = /^##\s+.+$/m.test(markdown);
  const sectionCount = (markdown.match(/^##\s+.+$/gm) || []).length;

  const handleImport = async () => {
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
      if (importMode === 'multiple' && hasSections) {
        const newChecklists = await importMultipleFromMarkdown(markdown);
        if (newChecklists.length > 0) {
          setSelectedChecklist(newChecklists[0].id);
          setImportResult({ count: newChecklists.length });
          // Close after a short delay to show the result
          setTimeout(handleClose, 1500);
        } else {
          setError(t('checklists.import.errorNoSections'));
        }
      } else {
        const newChecklist = await importFromMarkdown(markdown, customTitle || undefined);
        setSelectedChecklist(newChecklist.id);
        handleClose();
      }
    } catch {
      setError(t('checklists.import.errorParsing'));
    }
  };

  const handleClose = () => {
    setMarkdown('');
    setCustomTitle('');
    setError(null);
    setImportResult(null);
    onClose();
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-black/70 flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl p-6 w-full max-w-2xl max-h-[90vh] overflow-y-auto">
        <h2 className="text-xl font-cyber font-bold text-cyber-cyan mb-2">
          <TypewriterText text={t('checklists.import.title')} speed={20} />
        </h2>
        <p className="text-sm text-cyber-cyan/50 mb-6">
          <TypewriterText text={t('checklists.import.description')} delay={300} speed={10} />
        </p>

        {/* Success Message */}
        {importResult && (
          <div className="mb-4 p-4 bg-green-500/10 border border-green-500/30 rounded-lg">
            <div className="flex items-center gap-2 text-green-400">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
              </svg>
              <span className="font-mono">
                {t('checklists.import.successMultiple', { count: importResult.count })}
              </span>
            </div>
          </div>
        )}

        <div className="space-y-4">
          {/* Import Mode Toggle */}
          {hasSections && (
            <div className="p-3 bg-cyber-cyan/5 border border-cyber-cyan/20 rounded-lg">
              <p className="text-sm text-cyber-cyan/70 mb-3">
                {t('checklists.import.sectionsDetected', { count: sectionCount })}
              </p>
              <div className="flex gap-2">
                <button
                  type="button"
                  onClick={() => setImportMode('multiple')}
                  className={`flex-1 px-3 py-2 text-sm rounded-lg border transition-all ${
                    importMode === 'multiple'
                      ? 'bg-cyber-cyan/20 border-cyber-cyan text-cyber-cyan'
                      : 'border-cyber-cyan/30 text-cyber-cyan/50 hover:border-cyber-cyan/50'
                  }`}
                >
                  <div className="flex items-center justify-center gap-2">
                    <MultipleIcon />
                    <span>{t('checklists.import.modeMultiple')}</span>
                  </div>
                </button>
                <button
                  type="button"
                  onClick={() => setImportMode('single')}
                  className={`flex-1 px-3 py-2 text-sm rounded-lg border transition-all ${
                    importMode === 'single'
                      ? 'bg-cyber-cyan/20 border-cyber-cyan text-cyber-cyan'
                      : 'border-cyber-cyan/30 text-cyber-cyan/50 hover:border-cyber-cyan/50'
                  }`}
                >
                  <div className="flex items-center justify-center gap-2">
                    <SingleIcon />
                    <span>{t('checklists.import.modeSingle')}</span>
                  </div>
                </button>
              </div>
            </div>
          )}

          {/* Custom Title (only for single mode) */}
          {importMode === 'single' && (
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
          )}

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
                setImportResult(null);
              }}
              placeholder={`# My Project Roadmap

## 🟢 Phase 1: Setup
> Initial setup and configuration

- [ ] **Install dependencies:** npm install
- [ ] Configure environment variables
- [x] Create project structure

## 🔵 Phase 2: Development
- [ ] Implement core features
- [ ] Write unit tests`}
              className="w-full bg-cyber-bg/50 text-cyber-cyan px-4 py-2 rounded-lg border border-cyber-cyan/20 focus:outline-none focus:border-cyber-cyan/50 placeholder:text-cyber-cyan/30 font-mono text-sm resize-none"
              rows={12}
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
            <pre className="text-xs text-cyber-cyan/50 font-mono whitespace-pre-wrap">
              {`# Main Title
## 🟢 Section (becomes a checklist)
> Optional description (blockquote)
- [ ] **Task title:** Description
- [x] Completed task
* [ ] Also works with asterisks`}
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
              disabled={!markdown.trim() || importResult !== null}
              className="flex-1 cyber-button disabled:opacity-50 disabled:cursor-not-allowed flex items-center justify-center gap-2"
            >
              <ImportIcon />
              {importMode === 'multiple' && hasSections
                ? t('checklists.import.buttonMultiple', { count: sectionCount })
                : t('checklists.import.button')}
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

function MultipleIcon() {
  return (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 11H5m14 0a2 2 0 012 2v6a2 2 0 01-2 2H5a2 2 0 01-2-2v-6a2 2 0 012-2m14 0V9a2 2 0 00-2-2M5 11V9a2 2 0 012-2m0 0V5a2 2 0 012-2h6a2 2 0 012 2v2M7 7h10" />
    </svg>
  );
}

function SingleIcon() {
  return (
    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
    </svg>
  );
}
