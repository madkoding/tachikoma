import api from './client';

export interface KanbanCardDto {
  id: string;
  column_id: string;
  title: string;
  description?: string;
  color?: string;
  labels: string[];
  due_date?: string;
  order: number;
  created_at: string;
  updated_at: string;
}

export interface KanbanColumnDto {
  id: string;
  board_id: string;
  name: string;
  color?: string;
  wip_limit?: number;
  order: number;
  cards: KanbanCardDto[];
  created_at: string;
  updated_at: string;
}

export interface KanbanBoardDto {
  id: string;
  name: string;
  description?: string;
  color?: string;
  is_archived: boolean;
  columns: KanbanColumnDto[];
  created_at: string;
  updated_at: string;
}

export interface KanbanBoardSummaryDto {
  id: string;
  name: string;
  description?: string;
  color?: string;
  is_archived: boolean;
  column_count: number;
  card_count: number;
  created_at: string;
  updated_at: string;
}

export interface CreateBoardRequest {
  name: string;
  description?: string;
  color?: string;
  with_default_columns?: boolean;
}

export interface UpdateBoardRequest {
  name?: string;
  description?: string;
  color?: string;
  is_archived?: boolean;
}

export interface CreateColumnRequest {
  name: string;
  color?: string;
  wip_limit?: number;
}

export interface UpdateColumnRequest {
  name?: string;
  color?: string;
  wip_limit?: number;
}

export interface CreateCardRequest {
  title: string;
  description?: string;
  color?: string;
  labels?: string[];
  due_date?: string;
}

export interface UpdateCardRequest {
  title?: string;
  description?: string;
  color?: string;
  labels?: string[];
  due_date?: string;
}

export interface MoveCardRequest {
  target_column_id: string;
  target_order: number;
}

export const kanbanApi = {
  listBoards: async (): Promise<KanbanBoardSummaryDto[]> => {
    const response = await api.get<KanbanBoardSummaryDto[]>('/kanban/boards');
    return response.data;
  },

  getBoard: async (boardId: string): Promise<KanbanBoardDto> => {
    const response = await api.get<KanbanBoardDto>(`/kanban/boards/${boardId}`);
    return response.data;
  },

  createBoard: async (request: CreateBoardRequest): Promise<KanbanBoardDto> => {
    const response = await api.post<KanbanBoardDto>('/kanban/boards', request);
    return response.data;
  },

  updateBoard: async (boardId: string, request: UpdateBoardRequest): Promise<KanbanBoardDto> => {
    const response = await api.patch<KanbanBoardDto>(`/kanban/boards/${boardId}`, request);
    return response.data;
  },

  deleteBoard: async (boardId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}`);
  },

  createColumn: async (boardId: string, request: CreateColumnRequest): Promise<KanbanColumnDto> => {
    const response = await api.post<KanbanColumnDto>(`/kanban/boards/${boardId}/columns`, request);
    return response.data;
  },

  updateColumn: async (boardId: string, columnId: string, request: UpdateColumnRequest): Promise<KanbanColumnDto> => {
    const response = await api.patch<KanbanColumnDto>(`/kanban/boards/${boardId}/columns/${columnId}`, request);
    return response.data;
  },

  deleteColumn: async (boardId: string, columnId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}/columns/${columnId}`);
  },

  createCard: async (boardId: string, columnId: string, request: CreateCardRequest): Promise<KanbanCardDto> => {
    const response = await api.post<KanbanCardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards`, request);
    return response.data;
  },

  updateCard: async (boardId: string, columnId: string, cardId: string, request: UpdateCardRequest): Promise<KanbanCardDto> => {
    const response = await api.patch<KanbanCardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}`, request);
    return response.data;
  },

  deleteCard: async (boardId: string, columnId: string, cardId: string): Promise<void> => {
    await api.delete(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}`);
  },

  moveCard: async (boardId: string, columnId: string, cardId: string, request: MoveCardRequest): Promise<KanbanBoardDto> => {
    const response = await api.put<KanbanBoardDto>(`/kanban/boards/${boardId}/columns/${columnId}/cards/${cardId}/move`, request);
    return response.data;
  },
};
