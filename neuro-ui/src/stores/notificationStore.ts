import { create } from 'zustand';

// Mapping from tool names to sidebar sections
const TOOL_TO_SECTION: Record<string, string> = {
  // Checklist tools
  'create_checklist': '/checklists',
  'update_checklist': '/checklists',
  'delete_checklist': '/checklists',
  'add_checklist_item': '/checklists',
  'toggle_checklist_item': '/checklists',
  
  // Music tools
  'create_playlist': '/music',
  'add_song': '/music',
  'delete_playlist': '/music',
  'play_music': '/music',
  
  // Future tools can be added here:
  // 'create_note': '/notes',
  // 'create_event': '/calendar',
  // 'create_reminder': '/reminders',
  // 'create_goal': '/goals',
  // 'create_habit': '/habits',
  // 'create_kanban': '/kanban',
};

// Duration for notification glow effect (in ms)
const NOTIFICATION_DURATION = 5000;

export interface NotificationState {
  // Active notifications by section path
  activeNotifications: Record<string, number>; // path -> timestamp
  
  // Actions
  notifySection: (section: string) => void;
  notifyFromTool: (toolName: string) => void;
  notifyFromTools: (toolNames: string[]) => void;
  clearNotification: (section: string) => void;
  clearAllNotifications: () => void;
  hasNotification: (section: string) => boolean;
}

export const useNotificationStore = create<NotificationState>((set, get) => ({
  activeNotifications: {},

  notifySection: (section: string) => {
    const now = Date.now();
    set((state) => ({
      activeNotifications: {
        ...state.activeNotifications,
        [section]: now,
      },
    }));

    // Auto-clear notification after duration
    setTimeout(() => {
      const currentState = get();
      // Only clear if it's the same notification (not overwritten by newer one)
      if (currentState.activeNotifications[section] === now) {
        get().clearNotification(section);
      }
    }, NOTIFICATION_DURATION);
  },

  notifyFromTool: (toolName: string) => {
    const section = TOOL_TO_SECTION[toolName];
    if (section) {
      get().notifySection(section);
    }
  },

  notifyFromTools: (toolNames: string[]) => {
    // Use Set to avoid duplicate notifications for same section
    const sections = new Set<string>();
    
    for (const toolName of toolNames) {
      const section = TOOL_TO_SECTION[toolName];
      if (section) {
        sections.add(section);
      }
    }
    
    // Notify all affected sections
    sections.forEach((section) => {
      get().notifySection(section);
    });
  },

  clearNotification: (section: string) => {
    set((state) => {
      const { [section]: _, ...rest } = state.activeNotifications;
      return { activeNotifications: rest };
    });
  },

  clearAllNotifications: () => {
    set({ activeNotifications: {} });
  },

  hasNotification: (section: string) => {
    return !!get().activeNotifications[section];
  },
}));
