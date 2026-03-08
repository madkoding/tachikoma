import { create } from 'zustand';
import {
  kanbanApi,
  KanbanBoardDto,
  KanbanBoardSummaryDto,
  KanbanColumnDto,
  KanbanCardDto,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export interface KanbanCard {
  id: string;
  columnId: string;
  title: string;
  description?: string;
  color?: string;
  labels: string[];
  dueDate?: Date;
  order: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface KanbanColumn {
  id: string;
  boardId: string;
  name: string;
  color?: string;
  wipLimit?: number;
  order: number;
  cards: KanbanCard[];
  createdAt: Date;
  updatedAt: Date;
}

export interface KanbanBoard {
  id: string;
  name: string;
  description?: string;
  color?: string;
  isArchived: boolean;
  columns: KanbanColumn[];
  createdAt: Date;
  updatedAt: Date;
}

export interface KanbanBoardSummary {
  id: string;
  name: string;
  description?: string;
  color?: string;
  isArchived: boolean;
  columnCount: number;
  cardCount: number;
  createdAt: Date;
  updatedAt: Date;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function cardDtoToModel(dto: KanbanCardDto): KanbanCard {
  return {
    id: dto.id,
    columnId: dto.column_id,
    title: dto.title,
    description: dto.description,
    color: dto.color,
    labels: dto.labels,
    dueDate: dto.due_date ? new Date(dto.due_date) : undefined,
    order: dto.order,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function columnDtoToModel(dto: KanbanColumnDto): KanbanColumn {
  return {
    id: dto.id,
    boardId: dto.board_id,
    name: dto.name,
    color: dto.color,
    wipLimit: dto.wip_limit,
    order: dto.order,
    cards: dto.cards.map(cardDtoToModel),
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function boardDtoToModel(dto: KanbanBoardDto): KanbanBoard {
  return {
    id: dto.id,
    name: dto.name,
    description: dto.description,
    color: dto.color,
    isArchived: dto.is_archived,
    columns: dto.columns.map(columnDtoToModel),
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function boardSummaryDtoToModel(dto: KanbanBoardSummaryDto): KanbanBoardSummary {
  return {
    id: dto.id,
    name: dto.name,
    description: dto.description,
    color: dto.color,
    isArchived: dto.is_archived,
    columnCount: dto.column_count,
    cardCount: dto.card_count,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

// =============================================================================
// Store State & Actions
// =============================================================================

interface KanbanState {
  // State
  boards: KanbanBoardSummary[];
  currentBoard: KanbanBoard | null;
  isLoading: boolean;
  error: string | null;
  
  // Drag state
  draggedCard: KanbanCard | null;
  draggedFromColumn: string | null;
  
  // Actions - Boards
  fetchBoards: () => Promise<void>;
  fetchBoard: (boardId: string) => Promise<void>;
  createBoard: (name: string, description?: string, color?: string) => Promise<KanbanBoard>;
  updateBoard: (boardId: string, updates: { name?: string; description?: string; color?: string; isArchived?: boolean }) => Promise<void>;
  deleteBoard: (boardId: string) => Promise<void>;
  
  // Actions - Columns
  createColumn: (name: string, color?: string, wipLimit?: number) => Promise<void>;
  updateColumn: (columnId: string, updates: { name?: string; color?: string; wipLimit?: number }) => Promise<void>;
  deleteColumn: (columnId: string) => Promise<void>;
  
  // Actions - Cards
  createCard: (columnId: string, title: string, description?: string, color?: string, labels?: string[]) => Promise<void>;
  updateCard: (columnId: string, cardId: string, updates: { title?: string; description?: string; color?: string; labels?: string[] }) => Promise<void>;
  deleteCard: (columnId: string, cardId: string) => Promise<void>;
  moveCard: (fromColumnId: string, cardId: string, toColumnId: string, toOrder: number) => Promise<void>;
  
  // Actions - Drag & Drop
  setDraggedCard: (card: KanbanCard | null, fromColumn: string | null) => void;
  
  // Optimistic update helper
  optimisticMoveCard: (fromColumnId: string, cardId: string, toColumnId: string, toOrder: number) => void;
}

export const useKanbanStore = create<KanbanState>((set, get) => ({
  // Initial state
  boards: [],
  currentBoard: null,
  isLoading: false,
  error: null,
  draggedCard: null,
  draggedFromColumn: null,
  
  // ==========================================================================
  // Board Actions
  // ==========================================================================
  
  fetchBoards: async () => {
    try {
      set({ isLoading: true, error: null });
      const dtos = await kanbanApi.listBoards();
      set({ boards: dtos.map(boardSummaryDtoToModel) });
    } catch (error) {
      console.error('Error fetching boards:', error);
      set({ error: 'Failed to fetch boards' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  fetchBoard: async (boardId: string) => {
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.getBoard(boardId);
      set({ currentBoard: boardDtoToModel(dto) });
    } catch (error) {
      console.error('Error fetching board:', error);
      set({ error: 'Failed to fetch board' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  createBoard: async (name: string, description?: string, color?: string) => {
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.createBoard({
        name,
        description,
        color,
        with_default_columns: true,
      });
      const board = boardDtoToModel(dto);
      
      // Add to boards list
      set((state) => ({
        boards: [board, ...state.boards] as KanbanBoardSummary[],
        currentBoard: board,
      }));
      
      return board;
    } catch (error) {
      console.error('Error creating board:', error);
      set({ error: 'Failed to create board' });
      throw error;
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateBoard: async (boardId: string, updates) => {
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.updateBoard(boardId, {
        name: updates.name,
        description: updates.description,
        color: updates.color,
        is_archived: updates.isArchived,
      });
      const board = boardDtoToModel(dto);
      
      set((state) => ({
        currentBoard: state.currentBoard?.id === boardId ? board : state.currentBoard,
        boards: state.boards.map((b) =>
          b.id === boardId ? { ...b, ...updates } : b
        ),
      }));
    } catch (error) {
      console.error('Error updating board:', error);
      set({ error: 'Failed to update board' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  deleteBoard: async (boardId: string) => {
    try {
      set({ isLoading: true, error: null });
      await kanbanApi.deleteBoard(boardId);
      
      set((state) => ({
        boards: state.boards.filter((b) => b.id !== boardId),
        currentBoard: state.currentBoard?.id === boardId ? null : state.currentBoard,
      }));
    } catch (error) {
      console.error('Error deleting board:', error);
      set({ error: 'Failed to delete board' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // ==========================================================================
  // Column Actions
  // ==========================================================================
  
  createColumn: async (name: string, color?: string, wipLimit?: number) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.createColumn(currentBoard.id, {
        name,
        color,
        wip_limit: wipLimit,
      });
      const column = columnDtoToModel(dto);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: [...state.currentBoard.columns, column],
        } : null,
      }));
    } catch (error) {
      console.error('Error creating column:', error);
      set({ error: 'Failed to create column' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateColumn: async (columnId: string, updates) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.updateColumn(currentBoard.id, columnId, {
        name: updates.name,
        color: updates.color,
        wip_limit: updates.wipLimit,
      });
      const column = columnDtoToModel(dto);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: state.currentBoard.columns.map((c) =>
            c.id === columnId ? { ...column, cards: c.cards } : c
          ),
        } : null,
      }));
    } catch (error) {
      console.error('Error updating column:', error);
      set({ error: 'Failed to update column' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  deleteColumn: async (columnId: string) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      await kanbanApi.deleteColumn(currentBoard.id, columnId);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: state.currentBoard.columns.filter((c) => c.id !== columnId),
        } : null,
      }));
    } catch (error) {
      console.error('Error deleting column:', error);
      set({ error: 'Failed to delete column' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  // ==========================================================================
  // Card Actions
  // ==========================================================================
  
  createCard: async (columnId: string, title: string, description?: string, color?: string, labels?: string[]) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.createCard(currentBoard.id, columnId, {
        title,
        description,
        color,
        labels,
      });
      const card = cardDtoToModel(dto);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: state.currentBoard.columns.map((c) =>
            c.id === columnId ? { ...c, cards: [...c.cards, card] } : c
          ),
        } : null,
      }));
    } catch (error) {
      console.error('Error creating card:', error);
      set({ error: 'Failed to create card' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  updateCard: async (columnId: string, cardId: string, updates) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      const dto = await kanbanApi.updateCard(currentBoard.id, columnId, cardId, {
        title: updates.title,
        description: updates.description,
        color: updates.color,
        labels: updates.labels,
      });
      const card = cardDtoToModel(dto);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: state.currentBoard.columns.map((c) =>
            c.id === columnId ? {
              ...c,
              cards: c.cards.map((card_) => card_.id === cardId ? card : card_),
            } : c
          ),
        } : null,
      }));
    } catch (error) {
      console.error('Error updating card:', error);
      set({ error: 'Failed to update card' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  deleteCard: async (columnId: string, cardId: string) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      set({ isLoading: true, error: null });
      await kanbanApi.deleteCard(currentBoard.id, columnId, cardId);
      
      set((state) => ({
        currentBoard: state.currentBoard ? {
          ...state.currentBoard,
          columns: state.currentBoard.columns.map((c) =>
            c.id === columnId ? {
              ...c,
              cards: c.cards.filter((card) => card.id !== cardId),
            } : c
          ),
        } : null,
      }));
    } catch (error) {
      console.error('Error deleting card:', error);
      set({ error: 'Failed to delete card' });
    } finally {
      set({ isLoading: false });
    }
  },
  
  moveCard: async (fromColumnId: string, cardId: string, toColumnId: string, toOrder: number) => {
    const { currentBoard } = get();
    if (!currentBoard) return;
    
    try {
      // Optimistic update first
      get().optimisticMoveCard(fromColumnId, cardId, toColumnId, toOrder);
      
      // Then sync with server
      const dto = await kanbanApi.moveCard(currentBoard.id, fromColumnId, cardId, {
        target_column_id: toColumnId,
        target_order: toOrder,
      });
      
      // Update with server response
      set({ currentBoard: boardDtoToModel(dto) });
    } catch (error) {
      console.error('Error moving card:', error);
      set({ error: 'Failed to move card' });
      // Refresh board to get correct state
      get().fetchBoard(currentBoard.id);
    }
  },
  
  // ==========================================================================
  // Drag & Drop
  // ==========================================================================
  
  setDraggedCard: (card: KanbanCard | null, fromColumn: string | null) => {
    set({ draggedCard: card, draggedFromColumn: fromColumn });
  },
  
  optimisticMoveCard: (fromColumnId: string, cardId: string, toColumnId: string, toOrder: number) => {
    set((state) => {
      if (!state.currentBoard) return state;
      
      // Find the card
      const fromColumn = state.currentBoard.columns.find((c) => c.id === fromColumnId);
      const card = fromColumn?.cards.find((c) => c.id === cardId);
      if (!card) return state;
      
      // Remove from source
      const newColumns = state.currentBoard.columns.map((col) => {
        if (col.id === fromColumnId) {
          return {
            ...col,
            cards: col.cards.filter((c) => c.id !== cardId),
          };
        }
        return col;
      });
      
      // Add to target
      const finalColumns = newColumns.map((col) => {
        if (col.id === toColumnId) {
          const newCards = [...col.cards];
          const movedCard = { ...card, columnId: toColumnId, order: toOrder };
          newCards.splice(toOrder, 0, movedCard);
          // Re-order
          return {
            ...col,
            cards: newCards.map((c, i) => ({ ...c, order: i })),
          };
        }
        return col;
      });
      
      return {
        ...state,
        currentBoard: {
          ...state.currentBoard,
          columns: finalColumns,
        },
      };
    });
  },
}));

// =============================================================================
// Utility Functions
// =============================================================================

export const COLUMN_COLORS = [
  { name: 'Indigo', value: '#6366f1' },
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Cyan', value: '#06b6d4' },
  { name: 'Green', value: '#22c55e' },
  { name: 'Yellow', value: '#eab308' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Red', value: '#ef4444' },
  { name: 'Pink', value: '#ec4899' },
  { name: 'Purple', value: '#a855f7' },
];

export const CARD_COLORS = [
  { name: 'Default', value: undefined },
  { name: 'Blue', value: '#3b82f6' },
  { name: 'Green', value: '#22c55e' },
  { name: 'Yellow', value: '#eab308' },
  { name: 'Orange', value: '#f97316' },
  { name: 'Red', value: '#ef4444' },
  { name: 'Purple', value: '#a855f7' },
];
