import { create } from 'zustand';

export type ToastType = 'info' | 'success' | 'error' | 'warning';
export type ToastProgress = number | null; // null = no progress bar

export interface Toast {
  id: string;
  type: ToastType;
  message: string;
  /** Optional action label shown on the right. */
  actionLabel?: string;
  /** Progress 0-100 when a long-running op (download/training). */
  progress?: ToastProgress;
}

interface ToastState {
  toasts: Toast[];
  toast: (message: string, type?: ToastType, opts?: { actionLabel?: string }) => string;
  /** Update progress on an existing toast (by id). null removes the bar. */
  updateProgress: (id: string, progress: ToastProgress) => void;
  dismiss: (id: string) => void;
  clear: () => void;
}

const DURATION_MS = 4000;

let counter = 0;
const nextId = () => `toast-${++counter}`;

export const useToastStore = create<ToastState>((set, get) => ({
  toasts: [],

  toast: (message, type = 'info', opts) => {
    const id = nextId();
    set((state) => ({
      toasts: [...state.toasts, { id, message, type, actionLabel: opts?.actionLabel }],
    }));
    // Auto-dismiss. Error toasts persist a bit longer.
    setTimeout(() => get().dismiss(id), type === 'error' ? 7000 : DURATION_MS);
    return id;
  },

  updateProgress: (id, progress) =>
    set((state) => ({
      toasts: state.toasts.map((t) => (t.id === id ? { ...t, progress } : t)),
    })),

  dismiss: (id) => set((state) => ({ toasts: state.toasts.filter((t) => t.id !== id) })),

  clear: () => set({ toasts: [] }),
}));
