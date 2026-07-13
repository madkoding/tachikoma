import api, { PaginatedResponse } from './client';

export interface ChecklistItemDto {
  id: string;
  content: string;
  is_completed: boolean;
  completed_at?: string;
  order: number;
  created_at: string;
  updated_at?: string;
}

export interface ChecklistDto {
  id: string;
  title: string;
  description?: string;
  priority: number;
  due_date?: string;
  notification_interval?: number;
  is_archived: boolean;
  total_items: number;
  completed_items: number;
  created_at: string;
  updated_at: string;
}

export interface ChecklistWithItemsDto extends ChecklistDto {
  items: ChecklistItemDto[];
}

export interface CreateChecklistRequest {
  title: string;
  description?: string;
  priority?: number;
  due_date?: string;
  notification_interval?: number;
  items?: CreateChecklistItemRequest[];
}

export interface CreateChecklistItemRequest {
  content: string;
  is_completed?: boolean;
  order?: number;
}

export interface UpdateChecklistRequest {
  title?: string;
  description?: string;
  priority?: number;
  due_date?: string;
  notification_interval?: number;
  is_archived?: boolean;
}

export interface UpdateChecklistItemRequest {
  content?: string;
  is_completed?: boolean;
  order?: number;
}

export interface ImportMarkdownRequest {
  markdown: string;
  title?: string;
}

export const checklistsApi = {
  list: async (page = 1, perPage = 50, includeArchived = false): Promise<PaginatedResponse<ChecklistDto>> => {
    const response = await api.get<PaginatedResponse<ChecklistDto>>('/checklists', {
      params: { page, per_page: perPage, include_archived: includeArchived }
    });
    return response.data;
  },

  get: async (id: string): Promise<ChecklistWithItemsDto> => {
    const response = await api.get<ChecklistWithItemsDto>(`/checklists/${id}`);
    return response.data;
  },

  create: async (request: CreateChecklistRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.post<ChecklistWithItemsDto>('/checklists', request);
    return response.data;
  },

  update: async (id: string, request: UpdateChecklistRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.patch<ChecklistWithItemsDto>(`/checklists/${id}`, request);
    return response.data;
  },

  delete: async (id: string): Promise<void> => {
    await api.delete(`/checklists/${id}`);
  },

  addItem: async (checklistId: string, request: CreateChecklistItemRequest): Promise<ChecklistItemDto> => {
    const response = await api.post<ChecklistItemDto>(`/checklists/${checklistId}/items`, request);
    return response.data;
  },

  updateItem: async (checklistId: string, itemId: string, request: UpdateChecklistItemRequest): Promise<ChecklistItemDto> => {
    const response = await api.patch<ChecklistItemDto>(`/checklists/${checklistId}/items/${itemId}`, request);
    return response.data;
  },

  deleteItem: async (checklistId: string, itemId: string): Promise<void> => {
    await api.delete(`/checklists/${checklistId}/items/${itemId}`);
  },

  toggleItem: async (checklistId: string, itemId: string): Promise<ChecklistItemDto> => {
    const response = await api.post<ChecklistItemDto>(`/checklists/${checklistId}/items/${itemId}/toggle`);
    return response.data;
  },

  importMarkdown: async (request: ImportMarkdownRequest): Promise<ChecklistWithItemsDto> => {
    const response = await api.post<ChecklistWithItemsDto>('/checklists/import/markdown', request);
    return response.data;
  },
};
