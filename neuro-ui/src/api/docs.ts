import api from './client';

export type DocType = 'text' | 'markdown' | 'code' | 'spreadsheet' | 'pdf';

export interface DocFolderDto {
  id: string;
  name: string;
  color?: string;
  parent_id?: string;
  doc_count: number;
  created_at: string;
  updated_at: string;
}

export interface DocumentDto {
  id: string;
  title: string;
  content: string;
  folder_id?: string;
  doc_type: DocType;
  mime_type?: string;
  size_bytes: number;
  is_starred: boolean;
  is_shared: boolean;
  shared_with: string[];
  created_at: string;
  updated_at: string;
}

export interface StorageStatsDto {
  total_documents: number;
  total_size_bytes: number;
  by_type: Record<DocType, number>;
}

export interface CreateDocRequest {
  title: string;
  content?: string;
  folder_id?: string;
  doc_type?: DocType;
  mime_type?: string;
}

export interface UpdateDocRequest {
  title?: string;
  content?: string;
  folder_id?: string;
  doc_type?: DocType;
  is_starred?: boolean;
  is_shared?: boolean;
  shared_with?: string[];
}

export interface CreateDocFolderRequest {
  name: string;
  color?: string;
  parent_id?: string;
}

export interface UpdateDocFolderRequest {
  name?: string;
  color?: string;
  parent_id?: string;
}

export const docsApi = {
  listDocs: async (folderId?: string): Promise<DocumentDto[]> => {
    const response = await api.get<{ documents: DocumentDto[]; total: number }>('/docs', {
      params: { folder_id: folderId }
    });
    return response.data.documents || [];
  },

  searchDocs: async (query: string): Promise<DocumentDto[]> => {
    const response = await api.get<{ documents: DocumentDto[]; total: number }>('/docs/search', {
      params: { q: query }
    });
    return response.data.documents || [];
  },

  getStats: async (): Promise<StorageStatsDto> => {
    const response = await api.get<StorageStatsDto>('/docs/stats');
    return response.data;
  },

  getDoc: async (id: string): Promise<DocumentDto> => {
    const response = await api.get<DocumentDto>(`/docs/${id}`);
    return response.data;
  },

  createDoc: async (request: CreateDocRequest): Promise<DocumentDto> => {
    const response = await api.post<DocumentDto>('/docs', request);
    return response.data;
  },

  updateDoc: async (id: string, request: UpdateDocRequest): Promise<DocumentDto> => {
    const response = await api.patch<DocumentDto>(`/docs/${id}`, request);
    return response.data;
  },

  deleteDoc: async (id: string): Promise<void> => {
    await api.delete(`/docs/${id}`);
  },

  listFolders: async (): Promise<DocFolderDto[]> => {
    const response = await api.get<{ folders: DocFolderDto[] } | DocFolderDto[]>('/docs/folders');
    return Array.isArray(response.data) ? response.data : (response.data.folders || []);
  },

  createFolder: async (request: CreateDocFolderRequest): Promise<DocFolderDto> => {
    const response = await api.post<DocFolderDto>('/docs/folders', request);
    return response.data;
  },

  updateFolder: async (id: string, request: UpdateDocFolderRequest): Promise<DocFolderDto> => {
    const response = await api.patch<DocFolderDto>(`/docs/folders/${id}`, request);
    return response.data;
  },

  deleteFolder: async (id: string): Promise<void> => {
    await api.delete(`/docs/folders/${id}`);
  },
};
