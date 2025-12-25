import { create } from 'zustand';
import {
  docsApi,
  DocumentDto,
  DocFolderDto,
  StorageStatsDto,
  DocType,
  CreateDocRequest,
  UpdateDocRequest,
  CreateDocFolderRequest,
  UpdateDocFolderRequest,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export type { DocType };

export interface DocFolder {
  id: string;
  name: string;
  color?: string;
  parentId?: string;
  docCount: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface Document {
  id: string;
  title: string;
  content: string;
  folderId?: string;
  docType: DocType;
  mimeType?: string;
  sizeBytes: number;
  isStarred: boolean;
  isShared: boolean;
  sharedWith: string[];
  createdAt: Date;
  updatedAt: Date;
}

export interface StorageStats {
  totalDocuments: number;
  totalSizeBytes: number;
  byType: Record<DocType, number>;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function folderDtoToModel(dto: DocFolderDto): DocFolder {
  return {
    id: dto.id,
    name: dto.name,
    color: dto.color,
    parentId: dto.parent_id,
    docCount: dto.doc_count,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function docDtoToModel(dto: DocumentDto): Document {
  return {
    id: dto.id,
    title: dto.title,
    content: dto.content,
    folderId: dto.folder_id,
    docType: dto.doc_type,
    mimeType: dto.mime_type,
    sizeBytes: dto.size_bytes,
    isStarred: dto.is_starred,
    isShared: dto.is_shared,
    sharedWith: dto.shared_with,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function statsDtoToModel(dto: StorageStatsDto): StorageStats {
  return {
    totalDocuments: dto.total_documents,
    totalSizeBytes: dto.total_size_bytes,
    byType: dto.by_type,
  };
}

// =============================================================================
// Store Interface
// =============================================================================

interface DocsState {
  // Data
  documents: Document[];
  folders: DocFolder[];
  selectedDoc: Document | null;
  selectedFolderId: string | null;
  storageStats: StorageStats | null;
  
  // UI State
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
  filterStarred: boolean;
  
  // Actions
  loadDocs: (folderId?: string) => Promise<void>;
  loadFolders: () => Promise<void>;
  loadStats: () => Promise<void>;
  searchDocs: (query: string) => Promise<void>;
  selectDoc: (doc: Document | null) => void;
  selectFolder: (folderId: string | null) => void;
  setSearchQuery: (query: string) => void;
  setFilterStarred: (starred: boolean) => void;
  
  createDoc: (data: CreateDocRequest) => Promise<Document>;
  updateDoc: (id: string, data: UpdateDocRequest) => Promise<Document>;
  deleteDoc: (id: string) => Promise<void>;
  toggleStarDoc: (id: string) => Promise<Document>;
  
  createFolder: (data: CreateDocFolderRequest) => Promise<DocFolder>;
  updateFolder: (id: string, data: UpdateDocFolderRequest) => Promise<DocFolder>;
  deleteFolder: (id: string) => Promise<void>;
  
  clearError: () => void;
}

// =============================================================================
// Store Implementation
// =============================================================================

export const useDocsStore = create<DocsState>((set, get) => ({
  // Initial state
  documents: [],
  folders: [],
  selectedDoc: null,
  selectedFolderId: null,
  storageStats: null,
  isLoading: false,
  error: null,
  searchQuery: '',
  filterStarred: false,

  // Load documents with optional folder filter
  loadDocs: async (folderId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const dtos = await docsApi.listDocs(folderId);
      let docs = dtos.map(docDtoToModel);
      
      // Apply starred filter if enabled
      if (get().filterStarred) {
        docs = docs.filter((d) => d.isStarred);
      }
      
      set({ documents: docs, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load documents';
      set({ error: message, isLoading: false });
    }
  },

  // Load all folders
  loadFolders: async () => {
    try {
      const dtos = await docsApi.listFolders();
      const folders = dtos.map(folderDtoToModel);
      set({ folders });
    } catch (error) {
      console.error('Failed to load folders:', error);
    }
  },

  // Load storage stats
  loadStats: async () => {
    try {
      const dto = await docsApi.getStats();
      const stats = statsDtoToModel(dto);
      set({ storageStats: stats });
    } catch (error) {
      console.error('Failed to load storage stats:', error);
    }
  },

  // Search documents
  searchDocs: async (query: string) => {
    if (!query.trim()) {
      return get().loadDocs(get().selectedFolderId || undefined);
    }
    
    set({ isLoading: true, error: null, searchQuery: query });
    try {
      const dtos = await docsApi.searchDocs(query);
      const docs = dtos.map(docDtoToModel);
      set({ documents: docs, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to search documents';
      set({ error: message, isLoading: false });
    }
  },

  selectDoc: (doc) => set({ selectedDoc: doc }),
  selectFolder: (folderId) => {
    set({ selectedFolderId: folderId, searchQuery: '' });
    get().loadDocs(folderId || undefined);
  },
  setSearchQuery: (query) => set({ searchQuery: query }),
  setFilterStarred: (starred) => {
    set({ filterStarred: starred });
    get().loadDocs(get().selectedFolderId || undefined);
  },

  // Create a new document
  createDoc: async (data) => {
    set({ isLoading: true, error: null });
    try {
      const dto = await docsApi.createDoc(data);
      const doc = docDtoToModel(dto);
      set((state) => ({
        documents: [doc, ...state.documents],
        selectedDoc: doc,
        isLoading: false,
      }));
      // Refresh folder counts and stats
      get().loadFolders();
      get().loadStats();
      return doc;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create document';
      set({ error: message, isLoading: false });
      throw error;
    }
  },

  // Update a document
  updateDoc: async (id, data) => {
    try {
      const dto = await docsApi.updateDoc(id, data);
      const doc = docDtoToModel(dto);
      set((state) => ({
        documents: state.documents.map((d) => (d.id === id ? doc : d)),
        selectedDoc: state.selectedDoc?.id === id ? doc : state.selectedDoc,
      }));
      // Refresh folder counts if folder changed
      if (data.folder_id !== undefined) {
        get().loadFolders();
      }
      get().loadStats();
      return doc;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update document';
      set({ error: message });
      throw error;
    }
  },

  // Delete a document
  deleteDoc: async (id) => {
    try {
      await docsApi.deleteDoc(id);
      set((state) => ({
        documents: state.documents.filter((d) => d.id !== id),
        selectedDoc: state.selectedDoc?.id === id ? null : state.selectedDoc,
      }));
      // Refresh folder counts and stats
      get().loadFolders();
      get().loadStats();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete document';
      set({ error: message });
      throw error;
    }
  },

  // Toggle star status
  toggleStarDoc: async (id) => {
    const doc = get().documents.find((d) => d.id === id);
    if (!doc) throw new Error('Document not found');
    
    return get().updateDoc(id, { is_starred: !doc.isStarred });
  },

  // Create a folder
  createFolder: async (data) => {
    try {
      const dto = await docsApi.createFolder(data);
      const folder = folderDtoToModel(dto);
      set((state) => ({
        folders: [...state.folders, folder],
      }));
      return folder;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create folder';
      set({ error: message });
      throw error;
    }
  },

  // Update a folder
  updateFolder: async (id, data) => {
    try {
      const dto = await docsApi.updateFolder(id, data);
      const folder = folderDtoToModel(dto);
      set((state) => ({
        folders: state.folders.map((f) => (f.id === id ? folder : f)),
      }));
      return folder;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update folder';
      set({ error: message });
      throw error;
    }
  },

  // Delete a folder
  deleteFolder: async (id) => {
    try {
      await docsApi.deleteFolder(id);
      set((state) => ({
        folders: state.folders.filter((f) => f.id !== id),
        selectedFolderId: state.selectedFolderId === id ? null : state.selectedFolderId,
      }));
      // Reload docs if we were viewing this folder
      if (get().selectedFolderId === id) {
        get().loadDocs();
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete folder';
      set({ error: message });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));
