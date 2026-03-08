import { useEffect, useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import {
  useKanbanStore,
  KanbanBoard,
  KanbanColumn,
  KanbanCard,
  COLUMN_COLORS,
  CARD_COLORS,
} from '../stores/kanbanStore';
import TypewriterText from '../components/common/TypewriterText';

// =============================================================================
// Card Component (Cyberpunk)
// =============================================================================

function CardItem({
  card,
  columnId,
  onEdit,
  onDelete,
}: {
  card: KanbanCard;
  columnId: string;
  onEdit: (card: KanbanCard) => void;
  onDelete: (cardId: string) => void;
}) {
  const { setDraggedCard } = useKanbanStore();

  const handleDragStart = (e: React.DragEvent) => {
    setDraggedCard(card, columnId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragEnd = () => {
    setDraggedCard(null, null);
  };

  return (
    <div
      draggable
      onDragStart={handleDragStart}
      onDragEnd={handleDragEnd}
      className="group bg-cyber-bg rounded-lg p-3 border-l-4 cursor-grab active:cursor-grabbing border border-cyber-cyan/20 hover:border-cyber-cyan/40 transition-all"
      style={{ borderLeftColor: card.color || '#00d4ff' }}
    >
      <div className="flex justify-between items-start gap-2">
        <h4 className="font-mono text-sm text-cyber-cyan flex-1">{card.title}</h4>
        <div className="opacity-0 group-hover:opacity-100 flex gap-1 transition-opacity">
          <button onClick={() => onEdit(card)} className="p-1 text-cyber-cyan/40 hover:text-cyber-cyan transition-colors">✏️</button>
          <button onClick={() => onDelete(card.id)} className="p-1 text-cyber-cyan/40 hover:text-red-400 transition-colors">🗑️</button>
        </div>
      </div>
      {card.description && <p className="text-xs text-cyber-cyan/50 mt-1 line-clamp-2 font-mono">{card.description}</p>}
      {card.labels.length > 0 && (
        <div className="flex flex-wrap gap-1 mt-2">
          {card.labels.map((label, i) => (
            <span key={i} className="px-2 py-0.5 bg-cyber-cyan/20 text-cyber-cyan text-xs rounded-full font-mono">{label}</span>
          ))}
        </div>
      )}
      {card.dueDate && <div className="flex items-center gap-1 mt-2 text-xs text-cyber-cyan/40 font-mono">📅 {card.dueDate.toLocaleDateString()}</div>}
    </div>
  );
}

// =============================================================================
// Column Component (Cyberpunk)
// =============================================================================

function ColumnComponent({
  column,
  onAddCard,
  onEditCard,
  onDeleteCard,
  onEditColumn,
  onDeleteColumn,
}: {
  column: KanbanColumn;
  onAddCard: (columnId: string) => void;
  onEditCard: (card: KanbanCard) => void;
  onDeleteCard: (columnId: string, cardId: string) => void;
  onEditColumn: (column: KanbanColumn) => void;
  onDeleteColumn: (columnId: string) => void;
}) {
  const { t } = useTranslation();
  const { draggedCard, draggedFromColumn, moveCard } = useKanbanStore();
  const [isDragOver, setIsDragOver] = useState(false);
  const dropZoneRef = useRef<HTMLDivElement>(null);

  const isOverLimit = column.wipLimit && column.cards.length >= column.wipLimit;

  const handleDragOver = (e: React.DragEvent) => {
    e.preventDefault();
    if (draggedCard && !isOverLimit) setIsDragOver(true);
  };

  const handleDragLeave = () => setIsDragOver(false);

  const handleDrop = (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragOver(false);
    if (draggedCard && draggedFromColumn && !isOverLimit) {
      const dropZone = dropZoneRef.current;
      if (dropZone) {
        const cards = dropZone.querySelectorAll('[draggable="true"]');
        let targetOrder = column.cards.length;
        const mouseY = e.clientY;
        for (let i = 0; i < cards.length; i++) {
          const cardRect = cards[i].getBoundingClientRect();
          const cardMiddle = cardRect.top + cardRect.height / 2;
          if (mouseY < cardMiddle) { targetOrder = i; break; }
        }
        moveCard(draggedFromColumn, draggedCard.id, column.id, targetOrder);
      }
    }
  };

  return (
    <div
      className={`flex-shrink-0 w-64 sm:w-72 bg-cyber-surface rounded-xl p-3 flex flex-col max-h-full border border-cyber-cyan/20 ${isDragOver ? 'ring-2 ring-cyber-cyan bg-cyber-cyan/5' : ''}`}
      onDragOver={handleDragOver}
      onDragLeave={handleDragLeave}
      onDrop={handleDrop}
    >
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <div className="w-3 h-3 rounded-full" style={{ backgroundColor: column.color || '#00d4ff', boxShadow: `0 0 8px ${column.color || '#00d4ff'}50` }} />
          <h3 className="font-cyber font-semibold text-cyber-cyan text-sm">{column.name}</h3>
          <span className="text-xs text-cyber-cyan/50 font-mono">{column.cards.length}{column.wipLimit && `/${column.wipLimit}`}</span>
        </div>
        <div className="flex gap-1">
          <button onClick={() => onEditColumn(column)} className="p-1 text-cyber-cyan/40 hover:text-cyber-cyan transition-colors text-sm">⚙️</button>
          <button onClick={() => onDeleteColumn(column.id)} className="p-1 text-cyber-cyan/40 hover:text-red-400 transition-colors text-sm">🗑️</button>
        </div>
      </div>
      {isOverLimit && (
        <div className="mb-2 px-2 py-1 bg-amber-500/20 text-amber-400 text-xs rounded-lg flex items-center gap-1 font-mono">⚠️ {t('kanban.wipLimitReached', 'WIP limit reached')}</div>
      )}
      <div ref={dropZoneRef} className="flex-1 overflow-y-auto space-y-2 min-h-[100px]">
        {column.cards.map((card) => (
          <CardItem key={card.id} card={card} columnId={column.id} onEdit={onEditCard} onDelete={(cardId) => onDeleteCard(column.id, cardId)} />
        ))}
      </div>
      <button onClick={() => onAddCard(column.id)} disabled={Boolean(isOverLimit)} className="mt-3 w-full py-2 text-sm font-mono text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed border border-dashed border-cyber-cyan/30 hover:border-cyber-cyan/50">
        + {t('kanban.addCard', 'Add Card')}
      </button>
    </div>
  );
}

// =============================================================================
// Card Modal (Cyberpunk)
// =============================================================================

function CardModal({ isOpen, card, columnId, onClose, onSave }: {
  isOpen: boolean;
  card: KanbanCard | null;
  columnId: string;
  onClose: () => void;
  onSave: (columnId: string, cardId: string | null, data: { title: string; description?: string; color?: string; labels?: string[] }) => void;
}) {
  const { t } = useTranslation();
  const [title, setTitle] = useState('');
  const [description, setDescription] = useState('');
  const [color, setColor] = useState<string | undefined>(undefined);
  const [labelsText, setLabelsText] = useState('');

  useEffect(() => {
    if (card) { setTitle(card.title); setDescription(card.description || ''); setColor(card.color); setLabelsText(card.labels.join(', ')); }
    else { setTitle(''); setDescription(''); setColor(undefined); setLabelsText(''); }
  }, [card]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    const labels = labelsText.split(',').map((l) => l.trim()).filter((l) => l.length > 0);
    onSave(columnId, card?.id || null, { title, description, color, labels });
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-md shadow-2xl shadow-cyber-cyan/10">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between">
          <h3 className="text-lg font-cyber font-semibold text-cyber-cyan">{card ? t('kanban.editCard', 'Edit Card') : t('kanban.newCard', 'New Card')}</h3>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>
        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('kanban.cardTitle', 'Title')}</label>
            <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} placeholder={t('kanban.cardTitlePlaceholder', 'Enter card title...')} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" autoFocus />
          </div>
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('kanban.cardDescription', 'Description')}</label>
            <textarea value={description} onChange={(e) => setDescription(e.target.value)} placeholder={t('kanban.cardDescriptionPlaceholder', 'Add a description...')} rows={3} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm resize-none focus:border-cyber-cyan focus:outline-none" />
          </div>
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">{t('kanban.cardColor', 'Color')}</label>
            <div className="flex gap-2 flex-wrap">
              {CARD_COLORS.map((c) => (
                <button key={c.name} type="button" onClick={() => setColor(c.value)} className={`w-8 h-8 rounded-full border-2 transition-all ${color === c.value ? 'border-cyber-cyan scale-110' : 'border-transparent hover:scale-105'}`} style={{ backgroundColor: c.value || '#1a1a2e', boxShadow: c.value ? `0 0 8px ${c.value}50` : undefined }} />
              ))}
            </div>
          </div>
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('kanban.cardLabels', 'Labels (comma-separated)')}</label>
            <input type="text" value={labelsText} onChange={(e) => setLabelsText(e.target.value)} placeholder="bug, feature, urgent" className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
          </div>
          <div className="flex gap-2 pt-2">
            <button type="button" onClick={onClose} className="flex-1 px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">{t('common.cancel', 'Cancel')}</button>
            <button type="submit" disabled={!title.trim()} className="flex-1 cyber-button disabled:opacity-50">{t('common.save', 'Save')}</button>
          </div>
        </form>
      </div>
    </div>
  );
}

// =============================================================================
// Column Modal (Cyberpunk)
// =============================================================================

function ColumnModal({ isOpen, column, onClose, onSave }: {
  isOpen: boolean;
  column: KanbanColumn | null;
  onClose: () => void;
  onSave: (columnId: string | null, data: { name: string; color?: string; wipLimit?: number }) => void;
}) {
  const { t } = useTranslation();
  const [name, setName] = useState('');
  const [color, setColor] = useState<string>('#00d4ff');
  const [wipLimit, setWipLimit] = useState<number | ''>('');

  useEffect(() => {
    if (column) { setName(column.name); setColor(column.color || '#00d4ff'); setWipLimit(column.wipLimit || ''); }
    else { setName(''); setColor('#00d4ff'); setWipLimit(''); }
  }, [column]);

  if (!isOpen) return null;

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;
    onSave(column?.id || null, { name, color, wipLimit: wipLimit ? Number(wipLimit) : undefined });
    onClose();
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-md shadow-2xl shadow-cyber-cyan/10">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between">
          <h3 className="text-lg font-cyber font-semibold text-cyber-cyan">{column ? t('kanban.editColumn', 'Edit Column') : t('kanban.newColumn', 'New Column')}</h3>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>
        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('kanban.columnName', 'Name')}</label>
            <input type="text" value={name} onChange={(e) => setName(e.target.value)} placeholder={t('kanban.columnNamePlaceholder', 'Enter column name...')} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" autoFocus />
          </div>
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">{t('kanban.columnColor', 'Color')}</label>
            <div className="flex gap-2 flex-wrap">
              {COLUMN_COLORS.map((c) => (
                <button key={c.name} type="button" onClick={() => setColor(c.value)} className={`w-8 h-8 rounded-full border-2 transition-all ${color === c.value ? 'border-cyber-cyan scale-110' : 'border-transparent hover:scale-105'}`} style={{ backgroundColor: c.value, boxShadow: `0 0 8px ${c.value}50` }} />
              ))}
            </div>
          </div>
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('kanban.wipLimit', 'WIP Limit (optional)')}</label>
            <input type="number" min={1} max={99} value={wipLimit} onChange={(e) => setWipLimit(e.target.value ? parseInt(e.target.value) : '')} placeholder="5" className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
            <p className="text-xs text-cyber-cyan/40 mt-1 font-mono">{t('kanban.wipLimitHelp', 'Maximum number of cards allowed in this column')}</p>
          </div>
          <div className="flex gap-2 pt-2">
            <button type="button" onClick={onClose} className="flex-1 px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">{t('common.cancel', 'Cancel')}</button>
            <button type="submit" disabled={!name.trim()} className="flex-1 cyber-button disabled:opacity-50">{t('common.save', 'Save')}</button>
          </div>
        </form>
      </div>
    </div>
  );
}

// =============================================================================
// Board Selector (Cyberpunk)
// =============================================================================

function BoardSelector({ boards, currentBoard, onSelect, onCreate }: {
  boards: { id: string; name: string; color?: string }[];
  currentBoard: KanbanBoard | null;
  onSelect: (boardId: string) => void;
  onCreate: () => void;
}) {
  const { t } = useTranslation();
  const [isOpen, setIsOpen] = useState(false);

  return (
    <div className="relative">
      <button onClick={() => setIsOpen(!isOpen)} className="flex items-center gap-2 px-3 py-2 bg-cyber-surface border border-cyber-cyan/30 rounded-lg hover:border-cyber-cyan/50 transition-colors">
        {currentBoard ? (
          <>
            <div className="w-3 h-3 rounded-full" style={{ backgroundColor: currentBoard.color || '#00d4ff', boxShadow: `0 0 6px ${currentBoard.color || '#00d4ff'}` }} />
            <span className="font-mono text-sm text-cyber-cyan">{currentBoard.name}</span>
          </>
        ) : (
          <span className="text-cyber-cyan/50 font-mono text-sm">{t('kanban.selectBoard', 'Select Board')}</span>
        )}
        <span className="ml-2 text-cyber-cyan/50">▼</span>
      </button>
      {isOpen && (
        <div className="absolute top-full left-0 mt-2 w-64 bg-cyber-surface border border-cyber-cyan/30 rounded-lg shadow-xl z-50">
          <div className="p-2 max-h-64 overflow-y-auto">
            {boards.map((board) => (
              <button key={board.id} onClick={() => { onSelect(board.id); setIsOpen(false); }} className={`w-full flex items-center gap-2 px-3 py-2 rounded-lg text-left transition-colors ${currentBoard?.id === board.id ? 'bg-cyber-cyan/20 text-cyber-cyan' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
                <div className="w-3 h-3 rounded-full" style={{ backgroundColor: board.color || '#00d4ff' }} />
                <span className="font-mono text-sm">{board.name}</span>
              </button>
            ))}
          </div>
          <div className="border-t border-cyber-cyan/20 p-2">
            <button onClick={() => { onCreate(); setIsOpen(false); }} className="w-full flex items-center gap-2 px-3 py-2 text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">+ {t('kanban.newBoard', 'New Board')}</button>
          </div>
        </div>
      )}
    </div>
  );
}

// =============================================================================
// Main Kanban Page (Cyberpunk)
// =============================================================================

export default function KanbanPage() {
  const { t } = useTranslation();
  const { boards, currentBoard, isLoading, fetchBoards, fetchBoard, createBoard, deleteBoard, createColumn, updateColumn, deleteColumn, createCard, updateCard, deleteCard } = useKanbanStore();

  const [isCardModalOpen, setIsCardModalOpen] = useState(false);
  const [isColumnModalOpen, setIsColumnModalOpen] = useState(false);
  const [editingCard, setEditingCard] = useState<KanbanCard | null>(null);
  const [editingColumn, setEditingColumn] = useState<KanbanColumn | null>(null);
  const [activeColumnId, setActiveColumnId] = useState<string>('');
  const [showSidebar, setShowSidebar] = useState(false);

  useEffect(() => { fetchBoards(); }, [fetchBoards]);

  const handleSelectBoard = (boardId: string) => { fetchBoard(boardId); setShowSidebar(false); };
  const handleCreateBoard = async () => { const name = prompt(t('kanban.enterBoardName', 'Enter board name:')); if (name) await createBoard(name); };
  const handleDeleteBoard = async () => { if (currentBoard && confirm(t('kanban.confirmDeleteBoard', 'Delete this board?'))) await deleteBoard(currentBoard.id); };
  const handleAddCard = (columnId: string) => { setActiveColumnId(columnId); setEditingCard(null); setIsCardModalOpen(true); };
  const handleEditCard = (card: KanbanCard) => { setActiveColumnId(card.columnId); setEditingCard(card); setIsCardModalOpen(true); };
  const handleSaveCard = async (columnId: string, cardId: string | null, data: { title: string; description?: string; color?: string; labels?: string[] }) => { if (cardId) await updateCard(columnId, cardId, data); else await createCard(columnId, data.title, data.description, data.color, data.labels); };
  const handleDeleteCard = async (columnId: string, cardId: string) => { if (confirm(t('kanban.confirmDeleteCard', 'Delete this card?'))) await deleteCard(columnId, cardId); };
  const handleAddColumn = () => { setEditingColumn(null); setIsColumnModalOpen(true); };
  const handleEditColumn = (column: KanbanColumn) => { setEditingColumn(column); setIsColumnModalOpen(true); };
  const handleSaveColumn = async (columnId: string | null, data: { name: string; color?: string; wipLimit?: number }) => { if (columnId) await updateColumn(columnId, data); else await createColumn(data.name, data.color, data.wipLimit); };
  const handleDeleteColumn = async (columnId: string) => { if (confirm(t('kanban.confirmDeleteColumn', 'Delete this column and all its cards?'))) await deleteColumn(columnId); };

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2 flex-wrap">
          <div className="flex items-center gap-2 sm:gap-4">
            <button onClick={() => setShowSidebar(!showSidebar)} className="sm:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan"><TypewriterText text={t('nav.kanban', 'Kanban')} speed={30} /></h1>
            <div className="hidden sm:block"><BoardSelector boards={boards} currentBoard={currentBoard} onSelect={handleSelectBoard} onCreate={handleCreateBoard} /></div>
          </div>
          {currentBoard && (
            <div className="flex gap-2">
              <button onClick={handleAddColumn} className="cyber-button text-sm">+ <span className="hidden sm:inline">{t('kanban.addColumn', 'Add Column')}</span></button>
              <button onClick={handleDeleteBoard} className="p-2 text-red-400 hover:bg-red-500/10 rounded-lg transition-colors">🗑️</button>
            </div>
          )}
        </div>
        <div className="sm:hidden mt-3"><BoardSelector boards={boards} currentBoard={currentBoard} onSelect={handleSelectBoard} onCreate={handleCreateBoard} /></div>
      </header>

      <div className="flex-1 overflow-x-auto p-3 sm:p-4 relative">
        {isLoading && !currentBoard ? (
          <div className="flex items-center justify-center h-full"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" /></div>
        ) : !currentBoard ? (
          <div className="flex flex-col items-center justify-center h-full text-center">
            <span className="text-5xl mb-4">🗂️</span>
            <p className="text-cyber-cyan/50 font-mono text-sm mb-4">{t('kanban.noBoard', 'No board selected')}</p>
            <button onClick={handleCreateBoard} className="cyber-button">+ {t('kanban.createFirstBoard', 'Create your first board')}</button>
          </div>
        ) : (
          <div className="flex gap-3 sm:gap-4 h-full pb-4">
            {currentBoard.columns.sort((a, b) => a.order - b.order).map((column) => (
              <ColumnComponent key={column.id} column={column} onAddCard={handleAddCard} onEditCard={handleEditCard} onDeleteCard={handleDeleteCard} onEditColumn={handleEditColumn} onDeleteColumn={handleDeleteColumn} />
            ))}
            <button onClick={handleAddColumn} className="flex-shrink-0 w-64 sm:w-72 h-32 border-2 border-dashed border-cyber-cyan/30 rounded-xl flex items-center justify-center text-cyber-cyan/50 hover:text-cyber-cyan hover:border-cyber-cyan/50 transition-colors font-mono">+ {t('kanban.addColumn', 'Add Column')}</button>
          </div>
        )}
      </div>

      <CardModal isOpen={isCardModalOpen} card={editingCard} columnId={activeColumnId} onClose={() => setIsCardModalOpen(false)} onSave={handleSaveCard} />
      <ColumnModal isOpen={isColumnModalOpen} column={editingColumn} onClose={() => setIsColumnModalOpen(false)} onSave={handleSaveColumn} />
    </div>
  );
}
