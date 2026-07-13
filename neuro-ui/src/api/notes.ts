import api from './client';

export interface NoteFolderDto {
  id: string;
  name: string;
  color?: string;
  parent_id?: string;
  note_count: number;
  created_at: string;
  updated_at: string;
}

export interface NoteDto {
  id: string;
  title: string;
  content: string;
  folder_id?: string;
  tags: string[];
  color?: string;
  is_pinned: boolean;
  is_archived: boolean;
  created_at: string;
  updated_at: string;
}

export interface CreateNoteRequest {
  title: string;
  content?: string;
  folder_id?: string;
  tags?: string[];
  color?: string;
}

export interface UpdateNoteRequest {
  title?: string;
  content?: string;
  folder_id?: string;
  tags?: string[];
  color?: string;
  is_pinned?: boolean;
  is_archived?: boolean;
}

export interface CreateFolderRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateFolderRequest {
  name?: string;
  color?: string;
  parent_id?: string;
}

export const notesApi = {
  listNotes: async (folderId?: string, includeArchived = false): Promise<NoteDto[]> => {
    const response = await api.get<{ notes: NoteDto[]; total: number }>('/notes', {
      params: { folder_id: folderId, include_archived: includeArchived }
    });
    return response.data.notes || [];
  },

  searchNotes: async (query: string): Promise<NoteDto[]> => {
    const response = await api.get<{ notes: NoteDto[]; total: number }>('/notes/search', {
      params: { q: query }
    });
    return response.data.notes || [];
  },

  getNote: async (id: string): Promise<NoteDto> => {
    const response = await api.get<NoteDto>(`/notes/${id}`);
    return response.data;
  },

  createNote: async (request: CreateNoteRequest): Promise<NoteDto> => {
    const response = await api.post<NoteDto>('/notes', request);
    return response.data;
  },

  updateNote: async (id: string, request: UpdateNoteRequest): Promise<NoteDto> => {
    const response = await api.patch<NoteDto>(`/notes/${id}`, request);
    return response.data;
  },

  deleteNote: async (id: string): Promise<void> => {
    await api.delete(`/notes/${id}`);
  },

  listFolders: async (): Promise<NoteFolderDto[]> => {
    const response = await api.get<{ folders: NoteFolderDto[] } | NoteFolderDto[]>('/notes/folders');
    return Array.isArray(response.data) ? response.data : (response.data.folders || []);
  },

  createFolder: async (request: CreateFolderRequest): Promise<NoteFolderDto> => {
    const response = await api.post<NoteFolderDto>('/notes/folders', request);
    return response.data;
  },

  updateFolder: async (id: string, request: UpdateFolderRequest): Promise<NoteFolderDto> => {
    const response = await api.patch<NoteFolderDto>(`/notes/folders/${id}`, request);
    return response.data;
  },

  deleteFolder: async (id: string): Promise<void> => {
    await api.delete(`/notes/folders/${id}`);
  },
};
