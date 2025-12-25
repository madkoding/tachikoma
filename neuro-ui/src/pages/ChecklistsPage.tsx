import { useState, useCallback, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { useChecklistStore, Checklist } from '../stores/checklistStore';
import ChecklistCard from '../components/checklists/ChecklistCard';
import ChecklistDetail from '../components/checklists/ChecklistDetail';
import CreateChecklistModal from '../components/checklists/CreateChecklistModal';
import ImportMarkdownModal from '../components/checklists/ImportMarkdownModal';
import TypewriterText from '../components/common/TypewriterText';

export default function ChecklistsPage() {
  const { t } = useTranslation();
  const { checklists, selectedChecklistId, setSelectedChecklist, reorderChecklists, fetchChecklists, fetchChecklist } = useChecklistStore();

  // Load checklists from API on mount
  useEffect(() => {
    fetchChecklists();
  }, [fetchChecklists]);

  // Load checklist details when selected
  useEffect(() => {
    if (selectedChecklistId) {
      const checklist = checklists.find(c => c.id === selectedChecklistId);
      // Fetch full details if we don't have items loaded yet
      if (checklist && checklist.items.length === 0 && checklist.totalItems > 0) {
        fetchChecklist(selectedChecklistId);
      }
    }
  }, [selectedChecklistId, checklists, fetchChecklist]);
  const [isCreateModalOpen, setIsCreateModalOpen] = useState(false);
  const [isImportModalOpen, setIsImportModalOpen] = useState(false);
  const [draggedId, setDraggedId] = useState<string | null>(null);
  const [dragOverId, setDragOverId] = useState<string | null>(null);

  const selectedChecklist = checklists.find((c) => c.id === selectedChecklistId);
  
  // Sort by order field
  const sortedChecklists = [...checklists].sort((a, b) => (a.order ?? 0) - (b.order ?? 0));
  const activeChecklists = sortedChecklists.filter((c) => !c.isArchived);
  const archivedChecklists = sortedChecklists.filter((c) => c.isArchived);

  const handleSelectChecklist = (checklist: Checklist) => {
    setSelectedChecklist(checklist.id);
  };

  const handleBack = () => {
    setSelectedChecklist(null);
  };

  // Drag and drop handlers
  const handleDragStart = useCallback((e: React.DragEvent, checklistId: string) => {
    setDraggedId(checklistId);
    e.dataTransfer.effectAllowed = 'move';
    e.dataTransfer.setData('text/plain', checklistId);
    // Add visual feedback
    if (e.currentTarget instanceof HTMLElement) {
      e.currentTarget.style.opacity = '0.5';
    }
  }, []);

  const handleDragEnd = useCallback((e: React.DragEvent) => {
    setDraggedId(null);
    setDragOverId(null);
    if (e.currentTarget instanceof HTMLElement) {
      e.currentTarget.style.opacity = '1';
    }
  }, []);

  const handleDragOver = useCallback((e: React.DragEvent, checklistId: string) => {
    e.preventDefault();
    e.dataTransfer.dropEffect = 'move';
    if (draggedId && draggedId !== checklistId) {
      setDragOverId(checklistId);
    }
  }, [draggedId]);

  const handleDragLeave = useCallback(() => {
    setDragOverId(null);
  }, []);

  const handleDrop = useCallback((e: React.DragEvent, targetId: string) => {
    e.preventDefault();
    
    if (!draggedId || draggedId === targetId) {
      setDraggedId(null);
      setDragOverId(null);
      return;
    }

    // Reorder only active checklists
    const draggedIndex = activeChecklists.findIndex((c) => c.id === draggedId);
    const targetIndex = activeChecklists.findIndex((c) => c.id === targetId);

    if (draggedIndex === -1 || targetIndex === -1) {
      setDraggedId(null);
      setDragOverId(null);
      return;
    }

    // Create new order
    const reordered = [...activeChecklists];
    const [removed] = reordered.splice(draggedIndex, 1);
    reordered.splice(targetIndex, 0, removed);

    // Update order values
    const updated = reordered.map((c, index) => ({ ...c, order: index }));
    reorderChecklists(updated);

    setDraggedId(null);
    setDragOverId(null);
  }, [draggedId, activeChecklists, reorderChecklists]);

  return (
    <div className="flex h-full bg-cyber-bg overflow-hidden">
      {/* Checklist List Panel */}
      <div
        className={`${
          selectedChecklist ? 'hidden md:flex' : 'flex'
        } flex-col w-full md:w-72 lg:w-80 xl:w-96 border-r border-cyber-cyan/20 bg-cyber-surface overflow-hidden`}
      >
        {/* Header */}
        <div className="p-3 sm:p-4 border-b border-cyber-cyan/20">
          <div className="flex items-center justify-between mb-2 sm:mb-4">
            <h1 className="text-base sm:text-lg lg:text-xl font-cyber font-bold text-cyber-cyan">
              <TypewriterText text={t('checklists.title')} speed={20} />
            </h1>
            <div className="flex gap-1 sm:gap-2">
              <button
                onClick={() => setIsImportModalOpen(true)}
                className="p-1.5 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all border border-transparent hover:border-cyber-cyan/30"
                title={t('checklists.import')}
              >
                <ImportIcon />
              </button>
              <button
                onClick={() => setIsCreateModalOpen(true)}
                className="p-1.5 sm:p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all border border-transparent hover:border-cyber-cyan/30"
                title={t('checklists.create')}
              >
                <PlusIcon />
              </button>
            </div>
          </div>
        </div>

        {/* Checklists */}
        <div className="flex-1 overflow-y-auto p-2 sm:p-4 space-y-2 sm:space-y-3">
          {activeChecklists.length === 0 && archivedChecklists.length === 0 ? (
            <EmptyState
              onCreateClick={() => setIsCreateModalOpen(true)}
              onImportClick={() => setIsImportModalOpen(true)}
            />
          ) : (
            <>
              {activeChecklists.map((checklist) => (
                <div
                  key={checklist.id}
                  draggable
                  onDragStart={(e) => handleDragStart(e, checklist.id)}
                  onDragEnd={handleDragEnd}
                  onDragOver={(e) => handleDragOver(e, checklist.id)}
                  onDragLeave={handleDragLeave}
                  onDrop={(e) => handleDrop(e, checklist.id)}
                  className={`transition-all duration-200 ${
                    dragOverId === checklist.id ? 'transform translate-y-1 opacity-70' : ''
                  } ${draggedId === checklist.id ? 'cursor-grabbing' : 'cursor-grab'}`}
                >
                  <ChecklistCard
                    checklist={checklist}
                    isSelected={selectedChecklistId === checklist.id}
                    onClick={() => handleSelectChecklist(checklist)}
                    isDragging={draggedId === checklist.id}
                    isDragOver={dragOverId === checklist.id}
                  />
                </div>
              ))}

              {archivedChecklists.length > 0 && (
                <div className="mt-6">
                  <h3 className="text-xs font-mono text-cyber-cyan/50 uppercase tracking-wider mb-3">
                    {t('checklists.archived')}
                  </h3>
                  {archivedChecklists.map((checklist) => (
                    <ChecklistCard
                      key={checklist.id}
                      checklist={checklist}
                      isSelected={selectedChecklistId === checklist.id}
                      onClick={() => handleSelectChecklist(checklist)}
                    />
                  ))}
                </div>
              )}
            </>
          )}
        </div>
      </div>

      {/* Detail Panel */}
      <div
        className={`${
          selectedChecklist ? 'flex' : 'hidden md:flex'
        } flex-1 flex-col overflow-hidden`}
      >
        {selectedChecklist ? (
          <ChecklistDetail checklist={selectedChecklist} onBack={handleBack} />
        ) : (
          <EmptyDetailState />
        )}
      </div>

      {/* Modals */}
      <CreateChecklistModal
        isOpen={isCreateModalOpen}
        onClose={() => setIsCreateModalOpen(false)}
      />
      <ImportMarkdownModal
        isOpen={isImportModalOpen}
        onClose={() => setIsImportModalOpen(false)}
      />
    </div>
  );
}

// Empty state component
function EmptyState({
  onCreateClick,
  onImportClick,
}: {
  onCreateClick: () => void;
  onImportClick: () => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8">
      <div className="w-16 h-16 mb-4 rounded-2xl bg-cyber-cyan/10 border border-cyber-cyan/30 flex items-center justify-center">
        <ChecklistIcon />
      </div>
      <h3 className="text-lg font-cyber font-bold text-cyber-cyan mb-2">
        <TypewriterText text={t('checklists.empty.title')} speed={20} />
      </h3>
      <p className="text-cyber-cyan/50 text-sm mb-6 max-w-xs">
        <TypewriterText text={t('checklists.empty.description')} delay={300} speed={12} />
      </p>
      <div className="flex gap-3">
        <button
          onClick={onCreateClick}
          className="cyber-button flex items-center gap-2"
        >
          <PlusIcon />
          {t('checklists.create')}
        </button>
        <button
          onClick={onImportClick}
          className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan border border-cyber-cyan/30 hover:border-cyber-cyan/50 rounded-lg transition-all flex items-center gap-2"
        >
          <ImportIcon />
          {t('checklists.import')}
        </button>
      </div>
    </div>
  );
}

// Empty detail state
function EmptyDetailState() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col items-center justify-center h-full text-center p-8 bg-cyber-bg">
      <div className="w-20 h-20 mb-6 rounded-2xl bg-cyber-cyan/5 border border-cyber-cyan/20 flex items-center justify-center">
        <svg
          className="w-10 h-10 text-cyber-cyan/30"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1.5}
            d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"
          />
        </svg>
      </div>
      <p className="text-cyber-cyan/40 font-mono text-sm">
        <TypewriterText text={t('checklists.selectToView')} speed={15} />
      </p>
    </div>
  );
}

// Icons
function PlusIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 4v16m8-8H4" />
    </svg>
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

function ChecklistIcon() {
  return (
    <svg className="w-8 h-8 text-cyber-cyan/50" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1.5}
        d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2m-6 9l2 2 4-4"
      />
    </svg>
  );
}
