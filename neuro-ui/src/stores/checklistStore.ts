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

// Result of parsing markdown - can be single or multiple checklists
export interface ParsedMarkdownResult {
  checklists: Array<{
    title: string;
    description?: string;
    priority: 1 | 2 | 3 | 4 | 5;
    items: Omit<ChecklistItem, 'id' | 'createdAt'>[];
  }>;
  mainTitle?: string;
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
  
  // Import from markdown (returns first checklist for backwards compatibility)
  importFromMarkdown: (markdown: string, title?: string) => Checklist;
  // Import multiple checklists from markdown with sections
  importMultipleFromMarkdown: (markdown: string) => Checklist[];
  
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

// Detect priority from emoji or keywords in title
function detectPriority(text: string): 1 | 2 | 3 | 4 | 5 {
  const lowerText = text.toLowerCase();
  
  // Priority emojis
  if (text.includes('🔴') || text.includes('❗') || text.includes('🚨')) return 1;
  if (text.includes('🟠') || text.includes('⚠️')) return 2;
  if (text.includes('🟡') || text.includes('⚡')) return 3;
  if (text.includes('🟢') || text.includes('✅')) return 4;
  if (text.includes('🔵') || text.includes('💡')) return 5;
  
  // Keywords
  if (lowerText.includes('urgent') || lowerText.includes('critical') || lowerText.includes('inmediata')) return 1;
  if (lowerText.includes('important') || lowerText.includes('importante')) return 2;
  if (lowerText.includes('sprint 0') || lowerText.includes('now')) return 1;
  
  return 3; // Default medium priority
}

// Clean markdown formatting from text
function cleanMarkdownText(text: string): string {
  return text
    .replace(/\*\*(.+?)\*\*/g, '$1')  // Bold **text**
    .replace(/\*(.+?)\*/g, '$1')       // Italic *text*
    .replace(/__(.+?)__/g, '$1')       // Bold __text__
    .replace(/_(.+?)_/g, '$1')         // Italic _text_
    .replace(/`(.+?)`/g, '$1')         // Code `text`
    .replace(/\[(.+?)\]\(.+?\)/g, '$1') // Links [text](url)
    .trim();
}

// Parse a checkbox line with enhanced format support
function parseCheckboxLine(line: string): { content: string; isCompleted: boolean } | null {
  // Match: - [ ] **Title:** Description  OR  - [ ] Simple item  OR  * [x] Item
  const checkboxMatch = line.match(/^[-*]\s*\[([ xX])\]\s*(.+)$/);
  if (!checkboxMatch) return null;
  
  const isCompleted = checkboxMatch[1].toLowerCase() === 'x';
  let content = checkboxMatch[2].trim();
  
  // Handle **Title:** Description format - keep the structure but clean markdown
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

// Advanced markdown parser that handles sections
function parseMarkdownAdvanced(markdown: string): ParsedMarkdownResult {
  const lines = markdown.split('\n');
  const result: ParsedMarkdownResult = { checklists: [] };
  
  let mainTitle: string | undefined;
  let currentSection: {
    title: string;
    description?: string;
    priority: 1 | 2 | 3 | 4 | 5;
    items: Omit<ChecklistItem, 'id' | 'createdAt'>[];
  } | null = null;
  
  for (let i = 0; i < lines.length; i++) {
    const line = lines[i];
    const trimmedLine = line.trim();
    
    // Skip empty lines and horizontal rules
    if (!trimmedLine || trimmedLine === '---' || trimmedLine.match(/^-{3,}$/)) {
      continue;
    }
    
    // Main title (# heading)
    if (trimmedLine.startsWith('# ') && !trimmedLine.startsWith('## ')) {
      mainTitle = cleanMarkdownText(trimmedLine.replace(/^#\s*/, ''));
      continue;
    }
    
    // Section header (## heading) - starts a new checklist
    if (trimmedLine.startsWith('## ')) {
      // Save previous section if it has items
      if (currentSection && currentSection.items.length > 0) {
        result.checklists.push(currentSection);
      }
      
      const sectionTitle = cleanMarkdownText(trimmedLine.replace(/^##\s*/, ''));
      currentSection = {
        title: sectionTitle,
        priority: detectPriority(trimmedLine),
        items: [],
      };
      continue;
    }
    
    // Subsection header (### heading) - can add to description or be treated as item group
    if (trimmedLine.startsWith('### ') && currentSection) {
      // Add as a separator item or skip
      continue;
    }
    
    // Blockquote - use as section description
    if (trimmedLine.startsWith('>') && currentSection) {
      const descriptionText = cleanMarkdownText(trimmedLine.replace(/^>\s*/, ''));
      currentSection.description = currentSection.description 
        ? `${currentSection.description} ${descriptionText}`
        : descriptionText;
      continue;
    }
    
    // Checkbox items
    const checkboxResult = parseCheckboxLine(trimmedLine);
    if (checkboxResult) {
      // If no section yet, create a default one
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
        completedAt: checkboxResult.isCompleted ? new Date() : undefined,
        order: currentSection.items.length,
      });
      continue;
    }
    
    // Regular list items (without checkbox) in a section with checkboxes - skip
    // Stack tecnológico items, etc.
    if (trimmedLine.match(/^[-*]\s+\*\*/) && currentSection) {
      // This is likely a non-checkbox list item like "* **Lenguaje:** Rust"
      // Skip it as it's not a task
      continue;
    }
  }
  
  // Don't forget the last section
  if (currentSection && currentSection.items.length > 0) {
    result.checklists.push(currentSection);
  }
  
  result.mainTitle = mainTitle;
  
  return result;
}

// Simple parser for backwards compatibility
function parseMarkdownChecklist(markdown: string): { title: string; description?: string; priority: 1 | 2 | 3 | 4 | 5; items: Omit<ChecklistItem, 'id' | 'createdAt'>[] } {
  const result = parseMarkdownAdvanced(markdown);
  
  // If we have multiple sections, merge them all into one
  if (result.checklists.length > 0) {
    const allItems: Omit<ChecklistItem, 'id' | 'createdAt'>[] = [];
    let order = 0;
    
    for (const checklist of result.checklists) {
      for (const item of checklist.items) {
        allItems.push({ ...item, order: order++ });
      }
    }
    
    return {
      title: result.mainTitle || result.checklists[0].title,
      description: result.checklists[0].description,
      priority: result.checklists[0].priority,
      items: allItems,
    };
  }
  
  return {
    title: result.mainTitle || 'Imported Checklist',
    priority: 3,
    items: [],
  };
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
        const { title, description, priority, items } = parseMarkdownChecklist(markdown);
        const now = new Date();
        
        const newChecklist: Checklist = {
          id: generateUUID(),
          title: customTitle || title,
          description,
          items: items.map((item, index) => ({
            ...item,
            id: generateUUID(),
            order: index,
            createdAt: now,
          })),
          priority,
          isArchived: false,
          createdAt: now,
          updatedAt: now,
        };

        get().addChecklist(newChecklist);
        return newChecklist;
      },

      importMultipleFromMarkdown: (markdown) => {
        const result = parseMarkdownAdvanced(markdown);
        const now = new Date();
        const createdChecklists: Checklist[] = [];

        for (const section of result.checklists) {
          const newChecklist: Checklist = {
            id: generateUUID(),
            title: section.title,
            description: section.description,
            items: section.items.map((item, index) => ({
              ...item,
              id: generateUUID(),
              order: index,
              createdAt: now,
            })),
            priority: section.priority,
            isArchived: false,
            createdAt: now,
            updatedAt: now,
          };

          get().addChecklist(newChecklist);
          createdChecklists.push(newChecklist);
        }

        return createdChecklists;
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
