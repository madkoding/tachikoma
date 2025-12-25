import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useNotesStore, Note, NoteFolder } from '../stores/notesStore';
import TypewriterText from '../components/common/TypewriterText';

// =============================================================================
// Note Editor Modal
// =============================================================================

function NoteModal({
  note,
  folders,
  onClose,
  onSave,
  onDelete,
}: {
  note: Note | null;
  folders: NoteFolder[];
  onClose: () => void;
  onSave: (data: { title: string; content?: string; color?: string; tags?: string[]; folder_id?: string }) => void;
  onDelete?: () => void;
}) {
  const { t } = useTranslation();
  const [title, setTitle] = useState(note?.title || '');
  const [content, setContent] = useState(note?.content || '');
  const [color, setColor] = useState(note?.color || '#00d4ff');
  const [tagsInput, setTagsInput] = useState(note?.tags?.join(', ') || '');
  const [folderId, setFolderId] = useState(note?.folderId || '');

  const COLORS = ['#00d4ff', '#00ff9f', '#ff00ff', '#ff6b00', '#ffff00', '#ef4444', '#a855f7', '#ec4899', '#64748b', '#1e293b'];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    const tags = tagsInput.split(',').map(t => t.trim()).filter(Boolean);
    onSave({ title: title.trim(), content: content.trim() || undefined, color, tags: tags.length > 0 ? tags : undefined, folder_id: folderId || undefined });
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-2xl shadow-2xl shadow-cyber-cyan/10 max-h-[90vh] overflow-auto">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between sticky top-0 bg-cyber-surface">
          <h2 className="text-lg font-cyber font-semibold text-cyber-cyan">
            {note ? t('notes.editNote', 'Edit Note') : t('notes.newNote', 'New Note')}
          </h2>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('notes.noteTitle', 'Title')} *</label>
            <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" placeholder={t('notes.noteTitlePlaceholder', 'Enter note title...')} autoFocus />
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('notes.noteContent', 'Content')}</label>
            <textarea value={content} onChange={(e) => setContent(e.target.value)} rows={8} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm resize-none focus:border-cyber-cyan focus:outline-none" placeholder={t('notes.noteContentPlaceholder', 'Write your note...')} />
          </div>

          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">{t('notes.color', 'Color')}</label>
              <div className="flex flex-wrap gap-2">
                {COLORS.map((c) => (
                  <button key={c} type="button" onClick={() => setColor(c)} className={`w-6 h-6 rounded-lg transition-all ${color === c ? 'ring-2 ring-white ring-offset-2 ring-offset-cyber-surface scale-110' : 'hover:scale-110'}`} style={{ backgroundColor: c }} />
                ))}
              </div>
            </div>
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('notes.folders', 'Folder')}</label>
              <select value={folderId} onChange={(e) => setFolderId(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm focus:border-cyber-cyan focus:outline-none">
                <option value="">{t('notes.allNotes', 'All Notes')}</option>
                {folders.map((f) => <option key={f.id} value={f.id}>{f.name}</option>)}
              </select>
            </div>
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('notes.tags', 'Tags')}</label>
            <input type="text" value={tagsInput} onChange={(e) => setTagsInput(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" placeholder={t('notes.tagsPlaceholder', 'work, ideas, personal...')} />
          </div>

          <div className="flex flex-col sm:flex-row justify-between gap-2 pt-2">
            {note && onDelete && <button type="button" onClick={onDelete} className="px-4 py-2 text-red-400 hover:bg-red-500/10 rounded-lg transition-colors font-mono text-sm">🗑️ {t('common.delete', 'Delete')}</button>}
            <div className="flex gap-2 ml-auto">
              <button type="button" onClick={onClose} className="px-4 py-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-colors font-mono text-sm">{t('common.cancel', 'Cancel')}</button>
              <button type="submit" disabled={!title.trim()} className="cyber-button disabled:opacity-50">{t('common.save', 'Save')}</button>
            </div>
          </div>
        </form>
      </div>
    </div>
  );
}

// =============================================================================
// Note Card Component
// =============================================================================

function NoteCard({ note, onClick, onTogglePin }: { note: Note; onClick: () => void; onTogglePin: () => void }) {
  return (
    <button onClick={onClick} className="w-full text-left p-3 sm:p-4 rounded-xl border transition-all hover:scale-[1.02] group relative" style={{ backgroundColor: `${note.color}10`, borderColor: `${note.color}30` }}>
      {note.isPinned && <span className="absolute top-2 right-2 text-amber-400">📌</span>}
      <h3 className="font-cyber font-medium text-sm sm:text-base mb-2 pr-6 truncate" style={{ color: note.color }}>{note.title}</h3>
      {note.content && <p className="text-cyber-cyan/50 text-xs sm:text-sm font-mono line-clamp-3 mb-3">{note.content}</p>}
      {note.tags && note.tags.length > 0 && (
        <div className="flex flex-wrap gap-1 mb-2">
          {note.tags.slice(0, 3).map((tag, i) => <span key={i} className="px-2 py-0.5 text-xs font-mono rounded-full" style={{ backgroundColor: `${note.color}20`, color: note.color }}>#{tag}</span>)}
          {note.tags.length > 3 && <span className="text-xs font-mono text-cyber-cyan/40">+{note.tags.length - 3}</span>}
        </div>
      )}
      <div className="flex items-center justify-between text-xs font-mono text-cyber-cyan/40">
        <span>{note.updatedAt.toLocaleDateString()}</span>
        <button onClick={(e) => { e.stopPropagation(); onTogglePin(); }} className="p-1 opacity-0 group-hover:opacity-100 transition-opacity hover:text-amber-400">{note.isPinned ? '📌' : '📍'}</button>
      </div>
    </button>
  );
}

// =============================================================================
// Folder Sidebar
// =============================================================================

function FolderSidebar({ folders, selectedFolderId, showPinned, showArchived, onSelectFolder, onShowPinned, onShowArchived, onCreateFolder, onDeleteFolder }: {
  folders: NoteFolder[];
  selectedFolderId: string | null;
  showPinned: boolean;
  showArchived: boolean;
  onSelectFolder: (id: string | null) => void;
  onShowPinned: (show: boolean) => void;
  onShowArchived: (show: boolean) => void;
  onCreateFolder: () => void;
  onDeleteFolder: (id: string) => void;
}) {
  const { t } = useTranslation();

  return (
    <div className="w-full md:w-56 lg:w-64 bg-cyber-surface border-r border-cyber-cyan/20 p-3 flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-cyber font-semibold text-cyber-cyan text-sm">{t('notes.folders', 'Folders')}</h3>
        <button onClick={onCreateFolder} className="p-1.5 text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">+</button>
      </div>

      <div className="space-y-1 flex-1 overflow-auto">
        <button onClick={() => { onSelectFolder(null); onShowPinned(false); onShowArchived(false); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${!selectedFolderId && !showPinned && !showArchived ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          📝 {t('notes.allNotes', 'All Notes')}
        </button>
        <button onClick={() => { onSelectFolder(null); onShowPinned(true); onShowArchived(false); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${showPinned ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          📌 {t('notes.pinned', 'Pinned')}
        </button>
        <button onClick={() => { onSelectFolder(null); onShowPinned(false); onShowArchived(true); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${showArchived ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          📦 {t('notes.archived', 'Archived')}
        </button>

        {folders.length > 0 && <div className="border-t border-cyber-cyan/20 my-2" />}
        {folders.map((folder) => (
          <div key={folder.id} className="group flex items-center">
            <button onClick={() => { onSelectFolder(folder.id); onShowPinned(false); onShowArchived(false); }} className={`flex-1 px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${selectedFolderId === folder.id ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
              📁 {folder.name}
            </button>
            <button onClick={() => onDeleteFolder(folder.id)} className="opacity-0 group-hover:opacity-100 p-1.5 text-cyber-cyan/40 hover:text-red-400 transition-all">🗑️</button>
          </div>
        ))}
      </div>
    </div>
  );
}

// =============================================================================
// Main Notes Page
// =============================================================================

export default function NotesPage() {
  const { t } = useTranslation();
  const { notes, folders, selectedNote, selectedFolderId, isLoading, error, searchQuery, loadNotes, loadFolders, selectNote, selectFolder, setSearchQuery, createNote, updateNote, deleteNote, togglePin, toggleArchive, createFolder, deleteFolder, clearError } = useNotesStore();

  const [showNoteModal, setShowNoteModal] = useState(false);
  const [editingNote, setEditingNote] = useState<Note | null>(null);
  const [showSidebar, setShowSidebar] = useState(false);
  const [showPinned, setShowPinned] = useState(false);
  const [showArchived, setShowArchived] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);

  useEffect(() => { loadNotes(); loadFolders(); }, [loadNotes, loadFolders]);

  const handleNewNote = () => { setEditingNote(null); setShowNoteModal(true); };
  const handleEditNote = (note: Note) => { setEditingNote(note); setShowNoteModal(true); };

  const handleSaveNote = async (data: Parameters<typeof createNote>[0]) => {
    try {
      if (editingNote) await updateNote(editingNote.id, data);
      else await createNote(data);
      setShowNoteModal(false); setEditingNote(null);
    } catch (e) { console.error('Failed to save note:', e); }
  };

  const handleDeleteNote = async () => {
    if (editingNote && window.confirm(t('notes.confirmDelete', 'Delete this note?'))) {
      try { await deleteNote(editingNote.id); setShowNoteModal(false); setEditingNote(null); } catch (e) { console.error('Failed to delete note:', e); }
    }
  };

  const handleCreateFolder = async () => {
    if (showNewFolderInput && newFolderName.trim()) {
      try { await createFolder({ name: newFolderName.trim() }); setNewFolderName(''); setShowNewFolderInput(false); } catch (e) { console.error('Failed to create folder:', e); }
    } else { setShowNewFolderInput(true); }
  };

  const handleDeleteFolder = async (id: string) => {
    if (window.confirm(t('notes.confirmDeleteFolder', 'Delete this folder?'))) {
      try { await deleteFolder(id); } catch (e) { console.error('Failed to delete folder:', e); }
    }
  };

  // Filter notes
  let filteredNotes = notes;
  if (showPinned) filteredNotes = notes.filter(n => n.isPinned);
  else if (showArchived) filteredNotes = notes.filter(n => n.isArchived);
  else if (selectedFolderId) filteredNotes = notes.filter(n => n.folderId === selectedFolderId);
  else filteredNotes = notes.filter(n => !n.isArchived);

  if (searchQuery) {
    const q = searchQuery.toLowerCase();
    filteredNotes = filteredNotes.filter(n => n.title.toLowerCase().includes(q) || n.content?.toLowerCase().includes(q) || n.tags?.some(tag => tag.toLowerCase().includes(q)));
  }

  // Sort: pinned first, then by date
  filteredNotes = [...filteredNotes].sort((a, b) => {
    if (a.isPinned && !b.isPinned) return -1;
    if (!a.isPinned && b.isPinned) return 1;
    return b.updatedAt.getTime() - a.updatedAt.getTime();
  });

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 sm:gap-4">
            <button onClick={() => setShowSidebar(!showSidebar)} className="md:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan"><TypewriterText text={t('nav.notes', 'Notes')} speed={30} /></h1>
          </div>
          <div className="flex items-center gap-2">
            <div className="relative hidden sm:block">
              <input type="text" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder={t('notes.searchPlaceholder', 'Search notes...')} className="w-40 lg:w-60 px-3 py-1.5 pl-8 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
              <svg className="w-4 h-4 absolute left-2.5 top-1/2 -translate-y-1/2 text-cyber-cyan/40" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
            </div>
            <button onClick={handleNewNote} className="cyber-button text-sm">+ <span className="hidden sm:inline">{t('notes.newNote', 'New Note')}</span></button>
          </div>
        </div>
        {/* Mobile search */}
        <div className="mt-3 sm:hidden">
          <input type="text" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder={t('notes.searchPlaceholder', 'Search notes...')} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
        </div>
      </header>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-red-500/20 border border-red-500/50 text-red-400 rounded-lg flex justify-between items-center font-mono text-sm">
          <span>{error}</span><button onClick={clearError} className="text-red-400 hover:text-red-300">✕</button>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden relative">
        {isLoading && <div className="absolute inset-0 bg-cyber-bg/80 flex items-center justify-center z-20"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" /></div>}

        {/* Sidebar */}
        <div className={`${showSidebar ? 'flex' : 'hidden md:flex'} absolute md:relative inset-0 md:inset-auto z-10`}>
          <button onClick={() => setShowSidebar(false)} className="md:hidden absolute inset-0 bg-black/50" />
          <div className="relative">
            <button onClick={() => setShowSidebar(false)} className="md:hidden absolute top-2 right-2 z-10 p-2 text-cyber-cyan/70 hover:text-cyber-cyan">✕</button>
            <FolderSidebar folders={folders} selectedFolderId={selectedFolderId} showPinned={showPinned} showArchived={showArchived} onSelectFolder={(id) => { selectFolder(id); setShowSidebar(false); }} onShowPinned={(show) => { setShowPinned(show); setShowSidebar(false); }} onShowArchived={(show) => { setShowArchived(show); setShowSidebar(false); }} onCreateFolder={handleCreateFolder} onDeleteFolder={handleDeleteFolder} />
          </div>
        </div>

        {/* New folder input popup */}
        {showNewFolderInput && (
          <div className="absolute left-4 md:left-60 top-4 bg-cyber-surface border border-cyber-cyan/30 rounded-lg shadow-lg p-3 z-30">
            <input type="text" value={newFolderName} onChange={(e) => setNewFolderName(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()} placeholder={t('notes.folderName', 'Folder name')} className="px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm" autoFocus />
            <div className="flex gap-2 mt-2">
              <button onClick={handleCreateFolder} className="px-3 py-1 bg-cyber-cyan text-cyber-bg rounded-lg text-sm font-mono">{t('common.create', 'Create')}</button>
              <button onClick={() => { setShowNewFolderInput(false); setNewFolderName(''); }} className="px-3 py-1 text-cyber-cyan/70 hover:text-cyber-cyan text-sm font-mono">{t('common.cancel', 'Cancel')}</button>
            </div>
          </div>
        )}

        {/* Notes grid */}
        <div className={`flex-1 overflow-auto p-3 sm:p-4 ${showSidebar ? 'hidden md:block' : 'block'}`}>
          {filteredNotes.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <span className="text-5xl mb-4">📝</span>
              <p className="text-cyber-cyan/50 font-mono text-sm mb-4">{t('notes.noNotes', 'No notes yet')}</p>
              <button onClick={handleNewNote} className="cyber-button">+ {t('notes.newNote', 'New Note')}</button>
            </div>
          ) : (
            <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3 sm:gap-4">
              {filteredNotes.map((note) => <NoteCard key={note.id} note={note} onClick={() => handleEditNote(note)} onTogglePin={() => togglePin(note.id)} />)}
            </div>
          )}
        </div>
      </div>

      {showNoteModal && <NoteModal note={editingNote} folders={folders} onClose={() => { setShowNoteModal(false); setEditingNote(null); }} onSave={handleSaveNote} onDelete={editingNote ? handleDeleteNote : undefined} />}
    </div>
  );
}
