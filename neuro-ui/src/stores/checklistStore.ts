import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface ChecklistItem {
  id: string;
  content: string;
  isCompleted: boolean;
  completedAt?: Date;
  order: number;
  createdAt: Date;
}

export interface Checklist {
  id: string;
  title: string;
  description?: string;
  items: ChecklistItem[];
  priority: 1 | 2 | 3 | 4 | 5;
  dueDate?: Date;
  notificationInterval?: number; // in minutes
  lastReminded?: Date;
  isArchived: boolean;
  createdAt: Date;
  updatedAt: Date;
}

interface ChecklistState {
  checklists: Checklist[];
  selectedChecklistId: string | null;
  isLoading: boolean;
  error: string | null;

  // Actions
  setChecklists: (checklists: Checklist[]) => void;
  addChecklist: (checklist: Checklist) => void;
  updateChecklist: (id: string, updates: Partial<Checklist>) => void;
  deleteChecklist: (id: string) => void;
  setSelectedChecklist: (id: string | null) => void;
  
  // Item actions
  addItem: (checklistId: string, item: ChecklistItem) => void;
  updateItem: (checklistId: string, itemId: string, updates: Partial<ChecklistItem>) => void;
  deleteItem: (checklistId: string, itemId: string) => void;
  toggleItem: (checklistId: string, itemId: string) => void;
  reorderItems: (checklistId: string, items: ChecklistItem[]) => void;
  
  // Import from markdown
  importFromMarkdown: (markdown: string, title?: string) => Checklist;
  
  // State management
  setLoading: (loading: boolean) => void;
  setError: (error: string | null) => void;
}

// Helper to generate UUID
function generateUUID(): string {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (c) => {
    const r = (Math.random() * 16) | 0;
    const v = c === 'x' ? r : (r & 0x3) | 0x8;
    return v.toString(16);
  });
}

// Parse markdown checklist
function parseMarkdownChecklist(markdown: string): { title: string; items: Omit<ChecklistItem, 'id' | 'createdAt'>[] } {
  const lines = markdown.split('\n').filter(line => line.trim());
  let title = 'Imported Checklist';
  const items: Omit<ChecklistItem, 'id' | 'createdAt'>[] = [];
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i].trim();
    
    // Check for title (# heading or first non-checkbox line)
    if (line.startsWith('#')) {
      title = line.replace(/^#+\s*/, '');
      continue;
    }
    
    // Parse checkbox items: - [ ] or - [x] or * [ ] or * [x]
    const checkboxMatch = line.match(/^[-*]\s*\[([ xX])\]\s*(.+)$/);
    if (checkboxMatch) {
      const isCompleted = checkboxMatch[1].toLowerCase() === 'x';
      const content = checkboxMatch[2].trim();
      items.push({
        content,
        isCompleted,
        completedAt: isCompleted ? new Date() : undefined,
        order: items.length,
      });
    }
  }
  
  return { title, items };
}

export const useChecklistStore = create<ChecklistState>()(
  persist(
    (set, get) => ({
      checklists: [],
      selectedChecklistId: null,
      isLoading: false,
      error: null,

      setChecklists: (checklists) => set({ checklists }),

      addChecklist: (checklist) =>
        set((state) => ({
          checklists: [checklist, ...state.checklists],
        })),

      updateChecklist: (id, updates) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === id
              ? { ...checklist, ...updates, updatedAt: new Date() }
              : checklist
          ),
        })),

      deleteChecklist: (id) =>
        set((state) => ({
          checklists: state.checklists.filter((c) => c.id !== id),
          selectedChecklistId:
            state.selectedChecklistId === id ? null : state.selectedChecklistId,
        })),

      setSelectedChecklist: (id) => set({ selectedChecklistId: id }),

      addItem: (checklistId, item) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === checklistId
              ? {
                  ...checklist,
                  items: [...checklist.items, item],
                  updatedAt: new Date(),
                }
              : checklist
          ),
        })),

      updateItem: (checklistId, itemId, updates) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === checklistId
              ? {
                  ...checklist,
                  items: checklist.items.map((item) =>
                    item.id === itemId ? { ...item, ...updates } : item
                  ),
                  updatedAt: new Date(),
                }
              : checklist
          ),
        })),

      deleteItem: (checklistId, itemId) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === checklistId
              ? {
                  ...checklist,
                  items: checklist.items.filter((item) => item.id !== itemId),
                  updatedAt: new Date(),
                }
              : checklist
          ),
        })),

      toggleItem: (checklistId, itemId) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === checklistId
              ? {
                  ...checklist,
                  items: checklist.items.map((item) =>
                    item.id === itemId
                      ? {
                          ...item,
                          isCompleted: !item.isCompleted,
                          completedAt: !item.isCompleted ? new Date() : undefined,
                        }
                      : item
                  ),
                  updatedAt: new Date(),
                }
              : checklist
          ),
        })),

      reorderItems: (checklistId, items) =>
        set((state) => ({
          checklists: state.checklists.map((checklist) =>
            checklist.id === checklistId
              ? { ...checklist, items, updatedAt: new Date() }
              : checklist
          ),
        })),

      importFromMarkdown: (markdown, customTitle) => {
        const { title, items } = parseMarkdownChecklist(markdown);
        const now = new Date();
        
        const newChecklist: Checklist = {
          id: generateUUID(),
          title: customTitle || title,
          items: items.map((item, index) => ({
            ...item,
            id: generateUUID(),
            order: index,
            createdAt: now,
          })),
          priority: 3,
          isArchived: false,
          createdAt: now,
          updatedAt: now,
        };

        get().addChecklist(newChecklist);
        return newChecklist;
      },

      setLoading: (isLoading) => set({ isLoading }),
      setError: (error) => set({ error }),
    }),
    {
      name: 'neuro-checklists',
      partialize: (state) => ({
        checklists: state.checklists,
        selectedChecklistId: state.selectedChecklistId,
      }),
    }
  )
);
