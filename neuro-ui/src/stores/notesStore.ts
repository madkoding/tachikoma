import { create } from 'zustand';
import {
  notesApi,
  NoteDto,
  NoteFolderDto,
  CreateNoteRequest,
  UpdateNoteRequest,
  CreateFolderRequest,
  UpdateFolderRequest,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export interface NoteFolder {
  id: string;
  name: string;
  color?: string;
  parentId?: string;
  noteCount: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface Note {
  id: string;
  title: string;
  content: string;
  folderId?: string;
  tags: string[];
  color?: string;
  isPinned: boolean;
  isArchived: boolean;
  createdAt: Date;
  updatedAt: Date;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function folderDtoToModel(dto: NoteFolderDto): NoteFolder {
  return {
    id: dto.id,
    name: dto.name,
    color: dto.color,
    parentId: dto.parent_id,
    noteCount: dto.note_count,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function noteDtoToModel(dto: NoteDto): Note {
  return {
    id: dto.id,
    title: dto.title,
    content: dto.content,
    folderId: dto.folder_id,
    tags: dto.tags,
    color: dto.color,
    isPinned: dto.is_pinned,
    isArchived: dto.is_archived,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

// =============================================================================
// Store Interface
// =============================================================================

interface NotesState {
  // Data
  notes: Note[];
  folders: NoteFolder[];
  selectedNote: Note | null;
  selectedFolderId: string | null;
  
  // UI State
  isLoading: boolean;
  error: string | null;
  searchQuery: string;
  showArchived: boolean;
  
  // Actions
  loadNotes: (folderId?: string) => Promise<void>;
  loadFolders: () => Promise<void>;
  searchNotes: (query: string) => Promise<void>;
  selectNote: (note: Note | null) => void;
  selectFolder: (folderId: string | null) => void;
  setSearchQuery: (query: string) => void;
  setShowArchived: (show: boolean) => void;
  
  createNote: (data: CreateNoteRequest) => Promise<Note>;
  updateNote: (id: string, data: UpdateNoteRequest) => Promise<Note>;
  deleteNote: (id: string) => Promise<void>;
  togglePinNote: (id: string) => Promise<Note>;
  toggleArchiveNote: (id: string) => Promise<Note>;
  
  createFolder: (data: CreateFolderRequest) => Promise<NoteFolder>;
  updateFolder: (id: string, data: UpdateFolderRequest) => Promise<NoteFolder>;
  deleteFolder: (id: string) => Promise<void>;
  
  clearError: () => void;
}

// =============================================================================
// Store Implementation
// =============================================================================

export const useNotesStore = create<NotesState>((set, get) => ({
  // Initial state
  notes: [],
  folders: [],
  selectedNote: null,
  selectedFolderId: null,
  isLoading: false,
  error: null,
  searchQuery: '',
  showArchived: false,

  // Load notes with optional folder filter
  loadNotes: async (folderId?: string) => {
    set({ isLoading: true, error: null });
    try {
      const { showArchived } = get();
      const dtos = await notesApi.listNotes(folderId, showArchived);
      const notes = dtos.map(noteDtoToModel);
      set({ notes, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to load notes';
      set({ error: message, isLoading: false });
    }
  },

  // Load all folders
  loadFolders: async () => {
    try {
      const dtos = await notesApi.listFolders();
      const folders = dtos.map(folderDtoToModel);
      set({ folders });
    } catch (error) {
      console.error('Failed to load folders:', error);
    }
  },

  // Search notes
  searchNotes: async (query: string) => {
    if (!query.trim()) {
      return get().loadNotes(get().selectedFolderId || undefined);
    }
    
    set({ isLoading: true, error: null, searchQuery: query });
    try {
      const dtos = await notesApi.searchNotes(query);
      const notes = dtos.map(noteDtoToModel);
      set({ notes, isLoading: false });
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to search notes';
      set({ error: message, isLoading: false });
    }
  },

  selectNote: (note) => set({ selectedNote: note }),
  selectFolder: (folderId) => {
    set({ selectedFolderId: folderId, searchQuery: '' });
    get().loadNotes(folderId || undefined);
  },
  setSearchQuery: (query) => set({ searchQuery: query }),
  setShowArchived: (show) => {
    set({ showArchived: show });
    get().loadNotes(get().selectedFolderId || undefined);
  },

  // Create a new note
  createNote: async (data) => {
    set({ isLoading: true, error: null });
    try {
      const dto = await notesApi.createNote(data);
      const note = noteDtoToModel(dto);
      set((state) => ({
        notes: [note, ...state.notes],
        selectedNote: note,
        isLoading: false,
      }));
      // Refresh folder counts
      get().loadFolders();
      return note;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to create note';
      set({ error: message, isLoading: false });
      throw error;
    }
  },

  // Update a note
  updateNote: async (id, data) => {
    try {
      const dto = await notesApi.updateNote(id, data);
      const note = noteDtoToModel(dto);
      set((state) => ({
        notes: state.notes.map((n) => (n.id === id ? note : n)),
        selectedNote: state.selectedNote?.id === id ? note : state.selectedNote,
      }));
      // Refresh folder counts if folder changed
      if (data.folder_id !== undefined) {
        get().loadFolders();
      }
      return note;
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to update note';
      set({ error: message });
      throw error;
    }
  },

  // Delete a note
  deleteNote: async (id) => {
    try {
      await notesApi.deleteNote(id);
      set((state) => ({
        notes: state.notes.filter((n) => n.id !== id),
        selectedNote: state.selectedNote?.id === id ? null : state.selectedNote,
      }));
      // Refresh folder counts
      get().loadFolders();
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete note';
      set({ error: message });
      throw error;
    }
  },

  // Toggle pin status
  togglePinNote: async (id) => {
    const note = get().notes.find((n) => n.id === id);
    if (!note) throw new Error('Note not found');
    
    return get().updateNote(id, { is_pinned: !note.isPinned });
  },

  // Toggle archive status
  toggleArchiveNote: async (id) => {
    const note = get().notes.find((n) => n.id === id);
    if (!note) throw new Error('Note not found');
    
    return get().updateNote(id, { is_archived: !note.isArchived });
  },

  // Create a folder
  createFolder: async (data) => {
    try {
      const dto = await notesApi.createFolder(data);
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
      const dto = await notesApi.updateFolder(id, data);
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
      await notesApi.deleteFolder(id);
      set((state) => ({
        folders: state.folders.filter((f) => f.id !== id),
        selectedFolderId: state.selectedFolderId === id ? null : state.selectedFolderId,
      }));
      // Reload notes if we were viewing this folder
      if (get().selectedFolderId === id) {
        get().loadNotes();
      }
    } catch (error) {
      const message = error instanceof Error ? error.message : 'Failed to delete folder';
      set({ error: message });
      throw error;
    }
  },

  clearError: () => set({ error: null }),
}));
