import { memo, useState, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { useChecklistStore, Checklist, ChecklistItem } from '../stores/checklistStore';
import { useNavigate } from 'react-router-dom';
import clsx from 'clsx';

interface ChecklistDetectorProps {
  readonly content: string;
  readonly isStreaming: boolean;
}

interface DetectedChecklistItem {
  id: string;
  content: string;
  isCompleted: boolean;
}

interface DetectedChecklist {
  id: string;
  title: string;
  description?: string;
  items: DetectedChecklistItem[];
}

// Helper to generate UUID
function generateUUID(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  // eslint-disable-next-line prefer-string-replace-all
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = Math.trunc(Math.random() * 16);
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

// Clean markdown from text
function cleanMarkdown(text: string): string {
  return text
    .replace(/\*\*(.+?)\*\*/g, '$1')
    .replace(/\*(.+?)\*/g, '$1')
    .trim();
}

// Detect checklist patterns in the content
function detectChecklists(content: string): DetectedChecklist[] {
  const checklists: DetectedChecklist[] = [];
  const lines = content.split('\n');
  
  let currentChecklist: DetectedChecklist | null = null;
  let collectingItems = false;
  
  for (const line of lines) {
    const trimmedLine = line.trim();
    
    // Detect headers (potential checklist titles)
    const headerRegex = /^(#{1,3})\s+(.+)$/;
    const headerMatch = headerRegex.exec(trimmedLine);
    if (headerMatch) {
      if (currentChecklist && currentChecklist.items.length > 0) {
        checklists.push(currentChecklist);
      }
      currentChecklist = {
        id: generateUUID(),
        title: cleanMarkdown(headerMatch[2]),
        items: [],
      };
      collectingItems = false;
      continue;
    }
    
    // Detect bold titles
    const boldTitleRegex = /^\*\*(.+?)\*\*:?\s*$/;
    const boldTitleMatch = boldTitleRegex.exec(trimmedLine);
    if (boldTitleMatch && !trimmedLine.includes('- [')) {
      if (currentChecklist && currentChecklist.items.length > 0) {
        checklists.push(currentChecklist);
      }
      currentChecklist = {
        id: generateUUID(),
        title: boldTitleMatch[1].trim(),
        items: [],
      };
      collectingItems = false;
      continue;
    }
    
    // Detect checkbox items
    const checkboxRegex = /^[-*]\s*\[([ xX])\]\s+(.+)$/;
    const checkboxMatch = checkboxRegex.exec(trimmedLine);
    if (checkboxMatch) {
      currentChecklist ??= {
        id: generateUUID(),
        title: 'Checklist',
        items: [],
      };
      
      currentChecklist.items.push({
        id: generateUUID(),
        content: cleanMarkdown(checkboxMatch[2]),
        isCompleted: checkboxMatch[1].toLowerCase() === 'x',
      });
      collectingItems = true;
      continue;
    }
    
    // Empty line
    if (!trimmedLine && collectingItems) {
      collectingItems = false;
    }
  }
  
  if (currentChecklist && currentChecklist.items.length > 0) {
    checklists.push(currentChecklist);
  }
  
  return checklists.filter(cl => cl.items.length >= 2);
}

function ChecklistDetector({ content, isStreaming }: ChecklistDetectorProps) {
  const { t } = useTranslation();
  const navigate = useNavigate();
  const { addChecklist, setSelectedChecklist, checklists: existingChecklists } = useChecklistStore();
  const [savedChecklists, setSavedChecklists] = useState<Set<string>>(new Set());
  const [expandedChecklists, setExpandedChecklists] = useState<Set<string>>(new Set());
  const [editableChecklists, setEditableChecklists] = useState<Map<string, DetectedChecklist>>(new Map());
  const [editingItemId, setEditingItemId] = useState<string | null>(null);
  const [editingTitleId, setEditingTitleId] = useState<string | null>(null);
  
  // Detect checklists in content and initialize editable state
  const detectedChecklists = useMemo(() => {
    if (isStreaming) return [];
    return detectChecklists(content);
  }, [content, isStreaming]);
  
  // Get the editable version of a checklist (or the original if not edited)
  const getChecklist = useCallback((original: DetectedChecklist): DetectedChecklist => {
    return editableChecklists.get(original.id) ?? original;
  }, [editableChecklists]);
  
  // Update a checklist in the editable state
  const updateChecklist = useCallback((checklistId: string, updates: Partial<DetectedChecklist>) => {
    setEditableChecklists(prev => {
      const current = prev.get(checklistId) ?? detectedChecklists.find(c => c.id === checklistId);
      if (!current) return prev;
      const next = new Map(prev);
      next.set(checklistId, { ...current, ...updates });
      return next;
    });
  }, [detectedChecklists]);
  
  // Toggle item completion
  const toggleItem = useCallback((checklistId: string, itemId: string) => {
    const checklist = getChecklist(detectedChecklists.find(c => c.id === checklistId)!);
    const updatedItems = checklist.items.map(item =>
      item.id === itemId ? { ...item, isCompleted: !item.isCompleted } : item
    );
    updateChecklist(checklistId, { items: updatedItems });
  }, [detectedChecklists, getChecklist, updateChecklist]);
  
  // Update item content
  const updateItemContent = useCallback((checklistId: string, itemId: string, newContent: string) => {
    const checklist = getChecklist(detectedChecklists.find(c => c.id === checklistId)!);
    const updatedItems = checklist.items.map(item =>
      item.id === itemId ? { ...item, content: newContent } : item
    );
    updateChecklist(checklistId, { items: updatedItems });
    setEditingItemId(null);
  }, [detectedChecklists, getChecklist, updateChecklist]);
  
  // Delete item
  const deleteItem = useCallback((checklistId: string, itemId: string) => {
    const checklist = getChecklist(detectedChecklists.find(c => c.id === checklistId)!);
    const updatedItems = checklist.items.filter(item => item.id !== itemId);
    updateChecklist(checklistId, { items: updatedItems });
  }, [detectedChecklists, getChecklist, updateChecklist]);
  
  // Update checklist title
  const updateTitle = useCallback((checklistId: string, newTitle: string) => {
    updateChecklist(checklistId, { title: newTitle });
    setEditingTitleId(null);
  }, [updateChecklist]);
  
  // Delete entire checklist (just hide it from view)
  const deleteChecklist = useCallback((checklistId: string) => {
    updateChecklist(checklistId, { items: [] }); // Empty items will filter it out
  }, [updateChecklist]);
  
  // Toggle expanded state
  const toggleExpanded = useCallback((checklistId: string) => {
    setExpandedChecklists(prev => {
      const next = new Set(prev);
      if (next.has(checklistId)) {
        next.delete(checklistId);
      } else {
        next.add(checklistId);
      }
      return next;
    });
  }, []);
  
  // Filter out deleted checklists
  const visibleChecklists = useMemo(() => {
    return detectedChecklists.filter(cl => {
      const editable = editableChecklists.get(cl.id);
      return !editable || editable.items.length > 0;
    });
  }, [detectedChecklists, editableChecklists]);
  
  if (visibleChecklists.length === 0) {
    return null;
  }
  
  const handleSaveChecklist = (originalChecklist: DetectedChecklist) => {
    const checklist = getChecklist(originalChecklist);
    const now = new Date();
    const items: ChecklistItem[] = checklist.items.map((item, idx) => ({
      id: generateUUID(),
      content: item.content,
      isCompleted: item.isCompleted,
      order: idx,
      createdAt: now,
    }));
    
    const newChecklist: Checklist = {
      id: generateUUID(),
      title: checklist.title,
      description: checklist.description,
      items,
      priority: 3,
      order: existingChecklists.length,
      isArchived: false,
      createdAt: now,
      updatedAt: now,
    };
    
    addChecklist(newChecklist);
    setSelectedChecklist(newChecklist.id);
    setSavedChecklists(prev => new Set(prev).add(originalChecklist.id));
  };
  
  const handleGoToChecklists = () => {
    navigate('/checklists');
  };
  
  return (
    <div className="mt-4 space-y-3">
      {visibleChecklists.map((originalChecklist) => {
        const checklist = getChecklist(originalChecklist);
        const isSaved = savedChecklists.has(originalChecklist.id);
        const isExpanded = expandedChecklists.has(originalChecklist.id);
        const completedCount = checklist.items.filter(i => i.isCompleted).length;
        
        return (
          <div
            key={originalChecklist.id}
            className="border border-cyber-cyan/30 rounded-lg bg-cyber-cyan/5 overflow-hidden"
          >
            {/* Header */}
            <div className="p-3 flex items-start justify-between gap-3">
              <div className="flex-1 min-w-0">
                <div className="flex items-center gap-2 mb-1">
                  <svg className="w-4 h-4 text-cyber-cyan shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4" />
                  </svg>
                  {editingTitleId === originalChecklist.id ? (
                    <input
                      type="text"
                      defaultValue={checklist.title}
                      autoFocus
                      className="flex-1 bg-cyber-bg/50 text-cyber-cyan text-sm px-2 py-0.5 rounded border border-cyber-cyan/30 focus:outline-none focus:border-cyber-cyan"
                      onBlur={(e) => updateTitle(originalChecklist.id, e.target.value)}
                      onKeyDown={(e) => {
                        if (e.key === 'Enter') {
                          updateTitle(originalChecklist.id, e.currentTarget.value);
                        } else if (e.key === 'Escape') {
                          setEditingTitleId(null);
                        }
                      }}
                    />
                  ) : (
                    <button
                      onClick={() => !isSaved && setEditingTitleId(originalChecklist.id)}
                      className={clsx(
                        'font-semibold text-cyber-cyan truncate text-left',
                        !isSaved && 'hover:text-cyber-cyan/80 cursor-text'
                      )}
                      disabled={isSaved}
                    >
                      {checklist.title}
                    </button>
                  )}
                </div>
                <p className="text-xs text-cyber-cyan/50">
                  {completedCount}/{checklist.items.length} {t('checklists.items', { count: checklist.items.length })}
                </p>
              </div>
              
              <div className="flex items-center gap-1 shrink-0">
                {!isSaved && (
                  <button
                    onClick={() => deleteChecklist(originalChecklist.id)}
                    className="p-1.5 text-red-400/50 hover:text-red-400 hover:bg-red-400/10 rounded transition-all"
                    title={t('common.delete')}
                  >
                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 7l-.867 12.142A2 2 0 0116.138 21H7.862a2 2 0 01-1.995-1.858L5 7m5 4v6m4-6v6m1-10V4a1 1 0 00-1-1h-4a1 1 0 00-1 1v3M4 7h16" />
                    </svg>
                  </button>
                )}
                <button
                  onClick={() => toggleExpanded(originalChecklist.id)}
                  className="p-1.5 text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded transition-all"
                >
                  <svg className={clsx('w-4 h-4 transition-transform', isExpanded && 'rotate-180')} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 9l-7 7-7-7" />
                  </svg>
                </button>
                {isSaved ? (
                  <button
                    onClick={handleGoToChecklists}
                    className="px-3 py-1.5 text-xs font-mono bg-green-500/20 text-green-400 border border-green-500/30 rounded-lg hover:bg-green-500/30 transition-all flex items-center gap-1"
                  >
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" />
                    </svg>
                    {t('checklists.saved')}
                  </button>
                ) : (
                  <button
                    onClick={() => handleSaveChecklist(originalChecklist)}
                    className="px-3 py-1.5 text-xs font-mono bg-cyber-cyan/10 text-cyber-cyan border border-cyber-cyan/30 rounded-lg hover:bg-cyber-cyan/20 hover:border-cyber-cyan/50 transition-all flex items-center gap-1"
                  >
                    <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
                    </svg>
                    {t('checklists.saveChecklist')}
                  </button>
                )}
              </div>
            </div>
            
            {/* Items list */}
            <div className={clsx(
              'border-t border-cyber-cyan/20 overflow-hidden transition-all',
              isExpanded ? 'max-h-96 opacity-100' : 'max-h-0 opacity-0'
            )}>
              <div className="p-2 space-y-1 max-h-80 overflow-y-auto">
                {checklist.items.map((item) => (
                  <div
                    key={item.id}
                    className="group flex items-center gap-2 p-1.5 rounded hover:bg-cyber-cyan/5 transition-all"
                  >
                    {/* Checkbox */}
                    <button
                      onClick={() => !isSaved && toggleItem(originalChecklist.id, item.id)}
                      disabled={isSaved}
                      className={clsx(
                        'w-4 h-4 rounded border flex items-center justify-center shrink-0 transition-all',
                        item.isCompleted
                          ? 'bg-cyber-cyan border-cyber-cyan'
                          : 'border-cyber-cyan/40 hover:border-cyber-cyan',
                        isSaved && 'cursor-default'
                      )}
                    >
                      {item.isCompleted && (
                        <svg className="w-2.5 h-2.5 text-cyber-bg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                        </svg>
                      )}
                    </button>
                    
                    {/* Content */}
                    {editingItemId === item.id ? (
                      <input
                        type="text"
                        defaultValue={item.content}
                        autoFocus
                        className="flex-1 bg-cyber-bg/50 text-cyber-cyan text-sm px-2 py-0.5 rounded border border-cyber-cyan/30 focus:outline-none focus:border-cyber-cyan"
                        onBlur={(e) => updateItemContent(originalChecklist.id, item.id, e.target.value)}
                        onKeyDown={(e) => {
                          if (e.key === 'Enter') {
                            updateItemContent(originalChecklist.id, item.id, e.currentTarget.value);
                          } else if (e.key === 'Escape') {
                            setEditingItemId(null);
                          }
                        }}
                      />
                    ) : (
                      <button
                        onClick={() => !isSaved && setEditingItemId(item.id)}
                        disabled={isSaved}
                        className={clsx(
                          'flex-1 text-sm text-left truncate',
                          item.isCompleted ? 'text-cyber-cyan/40 line-through' : 'text-cyber-cyan/80',
                          !isSaved && 'hover:text-cyber-cyan cursor-text'
                        )}
                      >
                        {item.content}
                      </button>
                    )}
                    
                    {/* Delete button */}
                    {!isSaved && (
                      <button
                        onClick={() => deleteItem(originalChecklist.id, item.id)}
                        className="p-1 text-red-400/0 group-hover:text-red-400/50 hover:!text-red-400 rounded transition-all"
                      >
                        <svg className="w-3 h-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
                        </svg>
                      </button>
                    )}
                  </div>
                ))}
              </div>
            </div>
            
            {/* Collapsed preview */}
            {!isExpanded && (
              <div className="px-3 pb-3 space-y-1">
                {checklist.items.slice(0, 3).map((item) => (
                  <div key={item.id} className="flex items-center gap-2 text-xs text-cyber-cyan/70">
                    <span className={clsx(
                      'w-3 h-3 rounded border flex items-center justify-center shrink-0',
                      item.isCompleted 
                        ? 'bg-cyber-cyan border-cyber-cyan' 
                        : 'border-cyber-cyan/40'
                    )}>
                      {item.isCompleted && (
                        <svg className="w-2 h-2 text-cyber-bg" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                          <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={3} d="M5 13l4 4L19 7" />
                        </svg>
                      )}
                    </span>
                    <span className={clsx('truncate', item.isCompleted && 'line-through opacity-50')}>
                      {item.content}
                    </span>
                  </div>
                ))}
                {checklist.items.length > 3 && (
                  <button
                    onClick={() => toggleExpanded(originalChecklist.id)}
                    className="text-xs text-cyber-cyan/40 pl-5 hover:text-cyber-cyan transition-all"
                  >
                    +{checklist.items.length - 3} {t('checklists.moreItems')}
                  </button>
                )}
              </div>
            )}
          </div>
        );
      })}
    </div>
  );
}

export default memo(ChecklistDetector);
