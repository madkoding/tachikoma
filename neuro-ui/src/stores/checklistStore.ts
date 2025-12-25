import { create } from 'zustand';
import { 
  checklistsApi, 
  ChecklistDto, 
  ChecklistWithItemsDto,
  ChecklistItemDto,
  CreateChecklistRequest,
  CreateChecklistItemRequest,
} from '../api/client';

// =============================================================================
// Types - Frontend models (camelCase)
// =============================================================================

export interface ChecklistItem {
  id: string;
  content: string;
  isCompleted: boolean;
  completedAt?: Date;
  order: number;
  createdAt: Date;
  updatedAt: Date;
}

export interface Checklist {
  id: string;
  title: string;
  description?: string;
  items: ChecklistItem[];
  priority: 1 | 2 | 3 | 4 | 5;
  order: number;
  dueDate?: Date;
  notificationInterval?: number;
  isArchived: boolean;
  totalItems: number;
  completedItems: number;
  createdAt: Date;
  updatedAt: Date;
}

// =============================================================================
// Converters - API DTO (snake_case) to Frontend Model (camelCase)
// =============================================================================

function itemDtoToModel(dto: ChecklistItemDto): ChecklistItem {
  return {
    id: dto.id,
    content: dto.content,
    isCompleted: dto.is_completed,
    completedAt: dto.completed_at ? new Date(dto.completed_at) : undefined,
    order: dto.order,
    createdAt: new Date(dto.created_at),
    updatedAt: dto.updated_at ? new Date(dto.updated_at) : new Date(dto.created_at),
  };
}

function checklistDtoToModel(dto: ChecklistDto, items: ChecklistItem[] = []): Checklist {
  return {
    id: dto.id,
    title: dto.title,
    description: dto.description,
    items,
    priority: (dto.priority || 3) as 1 | 2 | 3 | 4 | 5,
    order: 0, // Will be set by list position
    dueDate: dto.due_date ? new Date(dto.due_date) : undefined,
    notificationInterval: dto.notification_interval,
    isArchived: dto.is_archived,
    totalItems: dto.total_items,
    completedItems: dto.completed_items,
    createdAt: new Date(dto.created_at),
    updatedAt: new Date(dto.updated_at),
  };
}

function checklistWithItemsDtoToModel(dto: ChecklistWithItemsDto): Checklist {
  const items = dto.items.map(itemDtoToModel);
  return checklistDtoToModel(dto, items);
}

// =============================================================================
// Markdown Parser
// =============================================================================

function detectPriority(text: string): 1 | 2 | 3 | 4 | 5 {
  const lowerText = text.toLowerCase();
  
  if (text.includes('🔴') || text.includes('❗') || text.includes('🚨')) return 1;
  if (text.includes('🟠') || text.includes('⚠️')) return 2;
  if (text.includes('🟡') || text.includes('⚡')) return 3;
  if (text.includes('🟢') || text.includes('✅')) return 4;
  if (text.includes('🔵') || text.includes('💡')) return 5;
  
  if (lowerText.includes('urgent') || lowerText.includes('critical') || lowerText.includes('inmediata')) return 1;
  if (lowerText.includes('important') || lowerText.includes('importante')) return 2;
  
  return 3;
}

function cleanMarkdownText(text: string): string {
  return text
    .replace(/\*\*(.+?)\*\*/g, '$1')
    .replace(/\*(.+?)\*/g, '$1')
    .replace(/__(.+?)__/g, '$1')
    .replace(/_(.+?)_/g, '$1')
    .replace(/`(.+?)`/g, '$1')
    .replace(/\[(.+?)\]\(.+?\)/g, '$1')
    .trim();
}

function parseCheckboxLine(line: string): { content: string; isCompleted: boolean } | null {
  const checkboxMatch = line.match(/^[-*]\s*\[([ xX])\]\s*(.+)$/);
  if (!checkboxMatch) return null;
  
  const isCompleted = checkboxMatch[1].toLowerCase() === 'x';
  let content = checkboxMatch[2].trim();
  
  const boldTitleMatch = content.match(/^\*\*(.+?)\*\*:?\s*(.*)$/);
  if (boldTitleMatch) {
    const title = boldTitleMatch[1].trim();
    const description = boldTitleMatch[2].trim();
    content = description ? `${title}: ${description}` : title;
  } else {
    content = cleanMarkdownText(content);
  }
  
  return { content, isCompleted };
}

interface ParsedSection {
  title: string;
  description?: string;
  priority: 1 | 2 | 3 | 4 | 5;
  items: { content: string; isCompleted: boolean; order: number }[];
}

function parseMarkdownAdvanced(markdown: string): { sections: ParsedSection[]; mainTitle?: string } {
  const lines = markdown.split('\n');
  const result: { sections: ParsedSection[]; mainTitle?: string } = { sections: [] };
  
  let mainTitle: string | undefined;
  let currentSection: ParsedSection | null = null;
  
  for (const line of lines) {
    const trimmedLine = line.trim();
    
    if (!trimmedLine || trimmedLine === '---' || trimmedLine.match(/^-{3,}$/)) {
      continue;
    }
    
    if (trimmedLine.startsWith('# ') && !trimmedLine.startsWith('## ')) {
      mainTitle = cleanMarkdownText(trimmedLine.replace(/^#\s*/, ''));
      continue;
    }
    
    if (trimmedLine.startsWith('## ')) {
      if (currentSection && currentSection.items.length > 0) {
        result.sections.push(currentSection);
      }
      
      const sectionTitle = cleanMarkdownText(trimmedLine.replace(/^##\s*/, ''));
      currentSection = {
        title: sectionTitle,
        priority: detectPriority(trimmedLine),
        items: [],
      };
      continue;
    }
    
    if (trimmedLine.startsWith('>') && currentSection) {
      const descriptionText = cleanMarkdownText(trimmedLine.replace(/^>\s*/, ''));
      currentSection.description = currentSection.description 
        ? `${currentSection.description} ${descriptionText}`
        : descriptionText;
      continue;
    }
    
    const checkboxResult = parseCheckboxLine(trimmedLine);
    if (checkboxResult) {
      if (!currentSection) {
        currentSection = {
          title: mainTitle || 'Imported Checklist',
          priority: 3,
          items: [],
        };
      }
      
      currentSection.items.push({
        content: checkboxResult.content,
        isCompleted: checkboxResult.isCompleted,
        order: currentSection.items.length,
      });
    }
  }
  
  if (currentSection && currentSection.items.length > 0) {
    result.sections.push(currentSection);
  }
  
  result.mainTitle = mainTitle;
  return result;
}

// =============================================================================
// Store Interface
// =============================================================================

interface ChecklistState {
  checklists: Checklist[];
  selectedChecklistId: string | null;
  isLoading: boolean;
  error: string | null;

  // API Actions
  fetchChecklists: () => Promise<void>;
  fetchChecklist: (id: string) => Promise<Checklist | null>;
  createChecklist: (title: string, description?: string, priority?: number) => Promise<Checklist>;
  updateChecklist: (id: string, updates: { title?: string; description?: string; priority?: number; isArchived?: boolean }) => Promise<void>;
  deleteChecklist: (id: string) => Promise<void>;
  
  // Item API Actions
  addItem: (checklistId: string, content: string) => Promise<ChecklistItem>;
  updateItem: (checklistId: string, itemId: string, updates: { content?: string; isCompleted?: boolean; order?: number }) => Promise<void>;
  deleteItem: (checklistId: string, itemId: string) => Promise<void>;
  toggleItem: (checklistId: string, itemId: string) => Promise<void>;
  
  // Local state actions
  setSelectedChecklist: (id: string | null) => void;
  reorderChecklists: (checklists: Checklist[]) => void;
  reorderItems: (checklistId: string, items: ChecklistItem[]) => void;
  
  // Import from markdown
  importFromMarkdown: (markdown: string, title?: string) => Promise<Checklist>;
  importMultipleFromMarkdown: (markdown: string) => Promise<Checklist[]>;
  
  // State management
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

// =============================================================================
// Store Implementation
// =============================================================================

export const useChecklistStore = create<ChecklistState>()((set, get) => ({
  checklists: [],
  selectedChecklistId: null,
  isLoading: false,
  error: null,

  // ==========================================================================
  // Fetch all checklists
  // ==========================================================================
  fetchChecklists: async () => {
    set({ isLoading: true, error: null });
    try {
      const response = await checklistsApi.list(1, 100, true);
      const checklists = response.data.map((dto, index) => ({
        ...checklistDtoToModel(dto),
        order: index,
      }));
      set({ checklists, isLoading: false });
    } catch (error) {
      console.error('Failed to fetch checklists:', error);
      set({ 
        error: error instanceof Error ? error.message : 'Failed to fetch checklists',
        isLoading: false 
      });
    }
  },

  // ==========================================================================
  // Fetch single checklist with items
  // ==========================================================================
  fetchChecklist: async (id: string) => {
    try {
      const dto = await checklistsApi.get(id);
      const checklist = checklistWithItemsDtoToModel(dto);
      
      // Update in local state
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === id ? { ...checklist, order: c.order } : c
        ),
      }));
      
      return checklist;
    } catch (error) {
      console.error('Failed to fetch checklist:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to fetch checklist' });
      return null;
    }
  },

  // ==========================================================================
  // Create checklist
  // ==========================================================================
  createChecklist: async (title: string, description?: string, priority?: number) => {
    set({ error: null });
    try {
      const request: CreateChecklistRequest = {
        title,
        description,
        priority: priority || 3,
      };
      const dto = await checklistsApi.create(request);
      const checklist = checklistWithItemsDtoToModel(dto);
      
      set((state) => ({
        checklists: [{ ...checklist, order: 0 }, ...state.checklists.map(c => ({ ...c, order: c.order + 1 }))],
      }));
      
      return checklist;
    } catch (error) {
      console.error('Failed to create checklist:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to create checklist' });
      throw error;
    }
  },

  // ==========================================================================
  // Update checklist
  // ==========================================================================
  updateChecklist: async (id, updates) => {
    set({ error: null });
    try {
      const dto = await checklistsApi.update(id, {
        title: updates.title,
        description: updates.description,
        priority: updates.priority,
        is_archived: updates.isArchived,
      });
      const updatedChecklist = checklistWithItemsDtoToModel(dto);
      
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === id ? { ...updatedChecklist, order: c.order } : c
        ),
      }));
    } catch (error) {
      console.error('Failed to update checklist:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to update checklist' });
      throw error;
    }
  },

  // ==========================================================================
  // Delete checklist
  // ==========================================================================
  deleteChecklist: async (id) => {
    set({ error: null });
    try {
      await checklistsApi.delete(id);
      set((state) => ({
        checklists: state.checklists.filter((c) => c.id !== id),
        selectedChecklistId: state.selectedChecklistId === id ? null : state.selectedChecklistId,
      }));
    } catch (error) {
      console.error('Failed to delete checklist:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to delete checklist' });
      throw error;
    }
  },

  // ==========================================================================
  // Add item to checklist
  // ==========================================================================
  addItem: async (checklistId, content) => {
    set({ error: null });
    try {
      const checklist = get().checklists.find(c => c.id === checklistId);
      const order = checklist?.items.length || 0;
      
      const request: CreateChecklistItemRequest = {
        content,
        is_completed: false,
        order,
      };
      const dto = await checklistsApi.addItem(checklistId, request);
      const item = itemDtoToModel(dto);
      
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === checklistId
            ? { ...c, items: [...c.items, item], totalItems: c.totalItems + 1 }
            : c
        ),
      }));
      
      return item;
    } catch (error) {
      console.error('Failed to add item:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to add item' });
      throw error;
    }
  },

  // ==========================================================================
  // Update item
  // ==========================================================================
  updateItem: async (checklistId, itemId, updates) => {
    set({ error: null });
    try {
      const dto = await checklistsApi.updateItem(checklistId, itemId, {
        content: updates.content,
        is_completed: updates.isCompleted,
        order: updates.order,
      });
      const item = itemDtoToModel(dto);
      
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === checklistId
            ? {
                ...c,
                items: c.items.map((i) => (i.id === itemId ? item : i)),
              }
            : c
        ),
      }));
    } catch (error) {
      console.error('Failed to update item:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to update item' });
      throw error;
    }
  },

  // ==========================================================================
  // Delete item
  // ==========================================================================
  deleteItem: async (checklistId, itemId) => {
    set({ error: null });
    try {
      await checklistsApi.deleteItem(checklistId, itemId);
      
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === checklistId
            ? {
                ...c,
                items: c.items.filter((i) => i.id !== itemId),
                totalItems: c.totalItems - 1,
              }
            : c
        ),
      }));
    } catch (error) {
      console.error('Failed to delete item:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to delete item' });
      throw error;
    }
  },

  // ==========================================================================
  // Toggle item completion
  // ==========================================================================
  toggleItem: async (checklistId, itemId) => {
    set({ error: null });
    try {
      const dto = await checklistsApi.toggleItem(checklistId, itemId);
      const item = itemDtoToModel(dto);
      
      set((state) => ({
        checklists: state.checklists.map((c) =>
          c.id === checklistId
            ? {
                ...c,
                items: c.items.map((i) => (i.id === itemId ? item : i)),
                completedItems: item.isCompleted 
                  ? c.completedItems + 1 
                  : c.completedItems - 1,
              }
            : c
        ),
      }));
    } catch (error) {
      console.error('Failed to toggle item:', error);
      set({ error: error instanceof Error ? error.message : 'Failed to toggle item' });
      throw error;
    }
  },

  // ==========================================================================
  // Local state actions
  // ==========================================================================
  setSelectedChecklist: (id) => set({ selectedChecklistId: id }),

  reorderChecklists: (reorderedChecklists) => {
    const orderMap = new Map(reorderedChecklists.map((c, index) => [c.id, index]));
    
    set((state) => ({
      checklists: state.checklists
        .map((checklist) => {
          const newOrder = orderMap.get(checklist.id);
          if (newOrder !== undefined) {
            return { ...checklist, order: newOrder };
          }
          return checklist;
        })
        .sort((a, b) => a.order - b.order),
    }));
  },

  reorderItems: (checklistId, items) => {
    set((state) => ({
      checklists: state.checklists.map((c) =>
        c.id === checklistId ? { ...c, items } : c
      ),
    }));
    
    // Update order on server for each item
    items.forEach(async (item, index) => {
      if (item.order !== index) {
        try {
          await checklistsApi.updateItem(checklistId, item.id, { order: index });
        } catch (error) {
          console.error('Failed to update item order:', error);
        }
      }
    });
  },

  // ==========================================================================
  // Import from markdown
  // ==========================================================================
  importFromMarkdown: async (markdown, customTitle) => {
    const { sections, mainTitle } = parseMarkdownAdvanced(markdown);
    
    if (sections.length === 0) {
      throw new Error('No valid checklist items found in markdown');
    }
    
    // Merge all sections into one checklist
    const allItems: CreateChecklistItemRequest[] = [];
    let order = 0;
    for (const section of sections) {
      for (const item of section.items) {
        allItems.push({
          content: item.content,
          is_completed: item.isCompleted,
          order: order++,
        });
      }
    }
    
    const request: CreateChecklistRequest = {
      title: customTitle || mainTitle || sections[0].title,
      description: sections[0].description,
      priority: sections[0].priority,
      items: allItems,
    };
    
    const dto = await checklistsApi.create(request);
    const checklist = checklistWithItemsDtoToModel(dto);
    
    set((state) => ({
      checklists: [{ ...checklist, order: 0 }, ...state.checklists.map(c => ({ ...c, order: c.order + 1 }))],
    }));
    
    return checklist;
  },

  importMultipleFromMarkdown: async (markdown) => {
    const { sections } = parseMarkdownAdvanced(markdown);
    
    if (sections.length === 0) {
      throw new Error('No valid checklist items found in markdown');
    }
    
    const createdChecklists: Checklist[] = [];
    
    for (const section of sections) {
      const request: CreateChecklistRequest = {
        title: section.title,
        description: section.description,
        priority: section.priority,
        items: section.items.map((item, index) => ({
          content: item.content,
          is_completed: item.isCompleted,
          order: index,
        })),
      };
      
      const dto = await checklistsApi.create(request);
      const checklist = checklistWithItemsDtoToModel(dto);
      createdChecklists.push(checklist);
    }
    
    set((state) => ({
      checklists: [
        ...createdChecklists.map((c, i) => ({ ...c, order: i })),
        ...state.checklists.map(c => ({ ...c, order: c.order + createdChecklists.length })),
      ],
    }));
    
    return createdChecklists;
  },

  // ==========================================================================
  // State management
  // ==========================================================================
  setLoading: (isLoading) => set({ isLoading }),
  setError: (error) => set({ error }),
}));
