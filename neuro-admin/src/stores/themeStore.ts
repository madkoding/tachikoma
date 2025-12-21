import { create } from 'zustand';

interface ThemeStore {
  isDark: boolean;
  toggle: () => void;
  setDark: (dark: boolean) => void;
}

export const useThemeStore = create<ThemeStore>((set) => ({
  isDark: window.matchMedia('(prefers-color-scheme: dark)').matches,
  toggle: () =>
    set((state) => {
      const newDark = !state.isDark;
      if (newDark) {
        document.documentElement.classList.add('dark');
      } else {
        document.documentElement.classList.remove('dark');
      }
      return { isDark: newDark };
    }),
  setDark: (dark) => {
    if (dark) {
      document.documentElement.classList.add('dark');
    } else {
      document.documentElement.classList.remove('dark');
    }
    set({ isDark: dark });
  },
}));

// Initialize theme on load
if (useThemeStore.getState().isDark) {
  document.documentElement.classList.add('dark');
}
