import { useEffect, useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useDocsStore, Document, DocFolder, DocType } from '../stores/docsStore';
import TypewriterText from '../components/common/TypewriterText';

type DocumentType = DocType;

// =============================================================================
// Document Editor Modal
// =============================================================================

function DocModal({
  doc,
  folders,
  onClose,
  onSave,
  onDelete,
}: {
  doc: Document | null;
  folders: DocFolder[];
  onClose: () => void;
  onSave: (data: { title: string; content?: string; doc_type?: DocumentType; folder_id?: string }) => void;
  onDelete?: () => void;
}) {
  const { t } = useTranslation();
  const [title, setTitle] = useState(doc?.title || '');
  const [content, setContent] = useState(doc?.content || '');
  const [docType, setDocType] = useState<DocumentType>(doc?.docType || 'text');
  const [folderId, setFolderId] = useState(doc?.folderId || '');

  const DOC_TYPES: { value: DocumentType; label: string; icon: string }[] = [
    { value: 'text', label: t('docs.type.text', 'Text'), icon: '📄' },
    { value: 'markdown', label: t('docs.type.markdown', 'Markdown'), icon: '📝' },
    { value: 'code', label: t('docs.type.code', 'Code'), icon: '💻' },
    { value: 'spreadsheet', label: t('docs.type.spreadsheet', 'Spreadsheet'), icon: '📊' },
    { value: 'pdf', label: t('docs.type.pdf', 'PDF'), icon: '📑' },
  ];

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    if (!title.trim()) return;
    onSave({ title: title.trim(), content: content.trim() || undefined, doc_type: docType, folder_id: folderId || undefined });
  };

  return (
    <div className="fixed inset-0 bg-black/80 backdrop-blur-sm flex items-center justify-center z-50 p-4">
      <div className="bg-cyber-surface border border-cyber-cyan/30 rounded-xl w-full max-w-3xl shadow-2xl shadow-cyber-cyan/10 max-h-[90vh] overflow-auto">
        <div className="p-4 border-b border-cyber-cyan/20 flex items-center justify-between sticky top-0 bg-cyber-surface">
          <h2 className="text-lg font-cyber font-semibold text-cyber-cyan">
            {doc ? t('docs.editDoc', 'Edit Document') : t('docs.newDoc', 'New Document')}
          </h2>
          <button onClick={onClose} className="text-cyber-cyan/50 hover:text-cyber-cyan transition-colors">✕</button>
        </div>

        <form onSubmit={handleSubmit} className="p-4 space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('docs.docTitle', 'Title')} *</label>
              <input type="text" value={title} onChange={(e) => setTitle(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" placeholder={t('docs.docTitlePlaceholder', 'Enter document title...')} autoFocus />
            </div>
            <div>
              <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('notes.folders', 'Folder')}</label>
              <select value={folderId} onChange={(e) => setFolderId(e.target.value)} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm focus:border-cyber-cyan focus:outline-none">
                <option value="">{t('docs.allDocs', 'All Documents')}</option>
                {folders.map((f) => <option key={f.id} value={f.id}>{f.name}</option>)}
              </select>
            </div>
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-2">Type</label>
            <div className="flex flex-wrap gap-2">
              {DOC_TYPES.map((type) => (
                <button key={type.value} type="button" onClick={() => setDocType(type.value)} className={`px-3 py-1.5 rounded-lg text-xs font-mono transition-all flex items-center gap-1.5 ${docType === type.value ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'bg-cyber-bg text-cyber-cyan/50 border border-cyber-cyan/20 hover:border-cyber-cyan/40'}`}>
                  <span>{type.icon}</span><span className="hidden sm:inline">{type.label}</span>
                </button>
              ))}
            </div>
          </div>

          <div>
            <label className="block text-xs font-mono text-cyber-cyan/70 uppercase tracking-wider mb-1">{t('docs.docContent', 'Content')}</label>
            <textarea value={content} onChange={(e) => setContent(e.target.value)} rows={12} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm resize-none focus:border-cyber-cyan focus:outline-none" placeholder="Write your document content..." />
          </div>

          <div className="flex flex-col sm:flex-row justify-between gap-2 pt-2">
            {doc && onDelete && <button type="button" onClick={onDelete} className="px-4 py-2 text-red-400 hover:bg-red-500/10 rounded-lg transition-colors font-mono text-sm">🗑️ {t('common.delete', 'Delete')}</button>}
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
// Document Item Component
// =============================================================================

function DocItem({ doc, onClick, onToggleStar }: { doc: Document; onClick: () => void; onToggleStar: () => void }) {
  const typeIcons: Record<DocumentType, string> = { text: '📄', markdown: '📝', code: '💻', spreadsheet: '📊', pdf: '📑' };
  const typeColors: Record<DocumentType, string> = { text: '#00d4ff', markdown: '#00ff9f', code: '#ff6b00', spreadsheet: '#a855f7', pdf: '#ef4444' };

  return (
    <button onClick={onClick} className="w-full text-left p-3 sm:p-4 rounded-xl border border-cyber-cyan/20 bg-cyber-surface/50 transition-all hover:border-cyber-cyan/40 hover:bg-cyber-cyan/5 group">
      <div className="flex items-start gap-3">
        <span className="text-2xl sm:text-3xl" style={{ filter: `drop-shadow(0 0 8px ${typeColors[doc.docType]}50)` }}>{typeIcons[doc.docType]}</span>
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3 className="font-cyber font-medium text-cyber-cyan text-sm sm:text-base truncate">{doc.title}</h3>
            {doc.isStarred && <span className="text-amber-400">⭐</span>}
          </div>
          <p className="text-xs font-mono text-cyber-cyan/40 mt-1">{doc.updatedAt.toLocaleDateString()} • {(doc.sizeBytes / 1024).toFixed(1)} KB</p>
        </div>
        <button onClick={(e) => { e.stopPropagation(); onToggleStar(); }} className="p-2 opacity-0 group-hover:opacity-100 transition-opacity text-cyber-cyan/50 hover:text-amber-400">{doc.isStarred ? '⭐' : '☆'}</button>
      </div>
    </button>
  );
}

// =============================================================================
// Folder Sidebar
// =============================================================================

function DocFolderSidebar({ folders, selectedFolderId, showStarred, onSelectFolder, onShowStarred, onCreateFolder, onDeleteFolder, storageUsed, storageTotal }: {
  folders: DocFolder[];
  selectedFolderId: string | null;
  showStarred: boolean;
  onSelectFolder: (id: string | null) => void;
  onShowStarred: (show: boolean) => void;
  onCreateFolder: () => void;
  onDeleteFolder: (id: string) => void;
  storageUsed: number;
  storageTotal: number;
}) {
  const { t } = useTranslation();
  const usagePercent = (storageUsed / storageTotal) * 100;

  return (
    <div className="w-full md:w-56 lg:w-64 bg-cyber-surface border-r border-cyber-cyan/20 p-3 flex flex-col">
      <div className="flex items-center justify-between mb-3">
        <h3 className="font-cyber font-semibold text-cyber-cyan text-sm">{t('docs.folders', 'Folders')}</h3>
        <button onClick={onCreateFolder} className="p-1.5 text-cyber-cyan/50 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg transition-all">+</button>
      </div>

      <div className="space-y-1 flex-1 overflow-auto">
        <button onClick={() => { onSelectFolder(null); onShowStarred(false); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${!selectedFolderId && !showStarred ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          📁 {t('docs.allDocs', 'All Documents')}
        </button>
        <button onClick={() => { onSelectFolder(null); onShowStarred(true); }} className={`w-full px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${showStarred ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
          ⭐ {t('docs.starred', 'Starred')}
        </button>

        {folders.length > 0 && <div className="border-t border-cyber-cyan/20 my-2" />}
        {folders.map((folder) => (
          <div key={folder.id} className="group flex items-center">
            <button onClick={() => { onSelectFolder(folder.id); onShowStarred(false); }} className={`flex-1 px-3 py-2 rounded-lg text-left text-sm font-mono transition-colors flex items-center gap-2 ${selectedFolderId === folder.id ? 'bg-cyber-cyan/20 text-cyber-cyan border border-cyber-cyan/50' : 'text-cyber-cyan/70 hover:bg-cyber-cyan/10'}`}>
              📂 {folder.name}
            </button>
            <button onClick={() => onDeleteFolder(folder.id)} className="opacity-0 group-hover:opacity-100 p-1.5 text-cyber-cyan/40 hover:text-red-400 transition-all">��️</button>
          </div>
        ))}
      </div>

      {/* Storage indicator */}
      <div className="mt-4 pt-4 border-t border-cyber-cyan/20">
        <div className="flex items-center justify-between text-xs font-mono text-cyber-cyan/50 mb-2">
          <span>{t('docs.storage', 'Storage')}</span>
          <span>{(storageUsed / 1024 / 1024).toFixed(1)} / {(storageTotal / 1024 / 1024).toFixed(0)} MB</span>
        </div>
        <div className="h-1.5 bg-cyber-cyan/20 rounded-full overflow-hidden">
          <div className="h-full bg-cyber-cyan transition-all" style={{ width: `${Math.min(usagePercent, 100)}%` }} />
        </div>
      </div>
    </div>
  );
}

// =============================================================================
// Main Docs Page
// =============================================================================

export default function DocsPage() {
  const { t } = useTranslation();
  const { documents, folders, selectedFolderId, isLoading, error, searchQuery, storageStats, loadDocs, loadFolders, selectFolder, setSearchQuery, createDoc, updateDoc, deleteDoc, toggleStarDoc, createFolder, deleteFolder, clearError } = useDocsStore();

  // Computed storage values
  const storageUsed = storageStats?.totalSizeBytes ?? 0;
  const storageTotal = 100 * 1024 * 1024; // 100 MB default limit

  const [showDocModal, setShowDocModal] = useState(false);
  const [editingDoc, setEditingDoc] = useState<Document | null>(null);
  const [showSidebar, setShowSidebar] = useState(false);
  const [showStarred, setShowStarred] = useState(false);
  const [newFolderName, setNewFolderName] = useState('');
  const [showNewFolderInput, setShowNewFolderInput] = useState(false);

  useEffect(() => { loadDocs(); loadFolders(); }, [loadDocs, loadFolders]);

  const handleNewDoc = () => { setEditingDoc(null); setShowDocModal(true); };
  const handleEditDoc = (doc: Document) => { setEditingDoc(doc); setShowDocModal(true); };

  const handleSaveDoc = async (data: Parameters<typeof createDoc>[0]) => {
    try {
      if (editingDoc) await updateDoc(editingDoc.id, data);
      else await createDoc(data);
      setShowDocModal(false); setEditingDoc(null);
    } catch (e) { console.error('Failed to save document:', e); }
  };

  const handleDeleteDoc = async () => {
    if (editingDoc && window.confirm(t('docs.confirmDelete', 'Delete this document?'))) {
      try { await deleteDoc(editingDoc.id); setShowDocModal(false); setEditingDoc(null); } catch (e) { console.error('Failed to delete document:', e); }
    }
  };

  const handleCreateFolder = async () => {
    if (showNewFolderInput && newFolderName.trim()) {
      try { await createFolder({ name: newFolderName.trim() }); setNewFolderName(''); setShowNewFolderInput(false); } catch (e) { console.error('Failed to create folder:', e); }
    } else { setShowNewFolderInput(true); }
  };

  const handleDeleteFolder = async (id: string) => {
    if (window.confirm(t('docs.confirmDeleteFolder', 'Delete this folder?'))) {
      try { await deleteFolder(id); } catch (e) { console.error('Failed to delete folder:', e); }
    }
  };

  // Filter documents
  let filteredDocs = documents;
  if (showStarred) filteredDocs = documents.filter(d => d.isStarred);
  else if (selectedFolderId) filteredDocs = documents.filter(d => d.folderId === selectedFolderId);

  if (searchQuery) {
    const q = searchQuery.toLowerCase();
    filteredDocs = filteredDocs.filter(d => d.title.toLowerCase().includes(q) || d.content?.toLowerCase().includes(q));
  }

  // Sort by date
  filteredDocs = [...filteredDocs].sort((a, b) => b.updatedAt.getTime() - a.updatedAt.getTime());

  return (
    <div className="h-full flex flex-col bg-cyber-bg overflow-hidden">
      <div className="fixed inset-0 pointer-events-none opacity-5" style={{ backgroundImage: 'linear-gradient(to right, cyan 1px, transparent 1px), linear-gradient(to bottom, cyan 1px, transparent 1px)', backgroundSize: '40px 40px' }} />

      <header className="p-3 sm:p-4 border-b border-cyber-cyan/20 bg-cyber-surface/80 backdrop-blur relative z-10">
        <div className="flex items-center justify-between gap-2">
          <div className="flex items-center gap-2 sm:gap-4">
            <button onClick={() => setShowSidebar(!showSidebar)} className="md:hidden p-2 text-cyber-cyan/70 hover:text-cyber-cyan hover:bg-cyber-cyan/10 rounded-lg">
              <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M4 6h16M4 12h16M4 18h16" /></svg>
            </button>
            <h1 className="text-lg sm:text-xl font-cyber font-bold text-cyber-cyan"><TypewriterText text={t('nav.docs', 'Documents')} speed={30} /></h1>
          </div>
          <div className="flex items-center gap-2">
            <div className="relative hidden sm:block">
              <input type="text" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder={t('docs.searchPlaceholder', 'Search documents...')} className="w-40 lg:w-60 px-3 py-1.5 pl-8 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
              <svg className="w-4 h-4 absolute left-2.5 top-1/2 -translate-y-1/2 text-cyber-cyan/40" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" /></svg>
            </div>
            <button onClick={handleNewDoc} className="cyber-button text-sm">+ <span className="hidden sm:inline">{t('docs.newDoc', 'New Document')}</span></button>
          </div>
        </div>
        <div className="mt-3 sm:hidden">
          <input type="text" value={searchQuery} onChange={(e) => setSearchQuery(e.target.value)} placeholder={t('docs.searchPlaceholder', 'Search documents...')} className="w-full px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan placeholder-cyber-cyan/30 font-mono text-sm focus:border-cyber-cyan focus:outline-none" />
        </div>
      </header>

      {error && (
        <div className="mx-4 mt-4 p-3 bg-red-500/20 border border-red-500/50 text-red-400 rounded-lg flex justify-between items-center font-mono text-sm">
          <span>{error}</span><button onClick={clearError} className="text-red-400 hover:text-red-300">✕</button>
        </div>
      )}

      <div className="flex-1 flex overflow-hidden relative">
        {isLoading && <div className="absolute inset-0 bg-cyber-bg/80 flex items-center justify-center z-20"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-cyber-cyan" /></div>}

        <div className={`${showSidebar ? 'flex' : 'hidden md:flex'} absolute md:relative inset-0 md:inset-auto z-10`}>
          <button onClick={() => setShowSidebar(false)} className="md:hidden absolute inset-0 bg-black/50" />
          <div className="relative">
            <button onClick={() => setShowSidebar(false)} className="md:hidden absolute top-2 right-2 z-10 p-2 text-cyber-cyan/70 hover:text-cyber-cyan">✕</button>
            <DocFolderSidebar folders={folders} selectedFolderId={selectedFolderId} showStarred={showStarred} onSelectFolder={(id) => { selectFolder(id); setShowSidebar(false); }} onShowStarred={(show) => { setShowStarred(show); setShowSidebar(false); }} onCreateFolder={handleCreateFolder} onDeleteFolder={handleDeleteFolder} storageUsed={storageUsed} storageTotal={storageTotal} />
          </div>
        </div>

        {showNewFolderInput && (
          <div className="absolute left-4 md:left-60 top-4 bg-cyber-surface border border-cyber-cyan/30 rounded-lg shadow-lg p-3 z-30">
            <input type="text" value={newFolderName} onChange={(e) => setNewFolderName(e.target.value)} onKeyDown={(e) => e.key === 'Enter' && handleCreateFolder()} placeholder={t('docs.folderName', 'Folder name')} className="px-3 py-2 bg-cyber-bg border border-cyber-cyan/30 rounded-lg text-cyber-cyan font-mono text-sm" autoFocus />
            <div className="flex gap-2 mt-2">
              <button onClick={handleCreateFolder} className="px-3 py-1 bg-cyber-cyan text-cyber-bg rounded-lg text-sm font-mono">{t('common.create', 'Create')}</button>
              <button onClick={() => { setShowNewFolderInput(false); setNewFolderName(''); }} className="px-3 py-1 text-cyber-cyan/70 hover:text-cyber-cyan text-sm font-mono">{t('common.cancel', 'Cancel')}</button>
            </div>
          </div>
        )}

        <div className={`flex-1 overflow-auto p-3 sm:p-4 ${showSidebar ? 'hidden md:block' : 'block'}`}>
          {filteredDocs.length === 0 ? (
            <div className="flex flex-col items-center justify-center h-full text-center">
              <span className="text-5xl mb-4">📄</span>
              <p className="text-cyber-cyan/50 font-mono text-sm mb-4">{t('docs.noDocs', 'No documents yet')}</p>
              <button onClick={handleNewDoc} className="cyber-button">+ {t('docs.newDoc', 'New Document')}</button>
            </div>
          ) : (
            <div className="grid grid-cols-1 lg:grid-cols-2 gap-3 sm:gap-4">
              {filteredDocs.map((doc) => <DocItem key={doc.id} doc={doc} onClick={() => handleEditDoc(doc)} onToggleStar={() => toggleStarDoc(doc.id)} />)}
            </div>
          )}
        </div>
      </div>

      {showDocModal && <DocModal doc={editingDoc} folders={folders} onClose={() => { setShowDocModal(false); setEditingDoc(null); }} onSave={handleSaveDoc} onDelete={editingDoc ? handleDeleteDoc : undefined} />}
    </div>
  );
}
