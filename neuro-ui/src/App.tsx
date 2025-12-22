import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { useEffect } from 'react';
import ChatPage from './pages/ChatPage';
import { useThemeStore } from './stores/themeStore';

function App() {
  const { theme } = useThemeStore();

  // Apply theme to document
  useEffect(() => {
    const root = window.document.documentElement;
    root.classList.remove('light', 'dark');

    if (theme === 'system') {
      const systemTheme = window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light';
      root.classList.add(systemTheme);
    } else {
      root.classList.add(theme);
    }
  }, [theme]);

  return (
    <BrowserRouter>
      <div className="min-h-screen bg-gray-50 dark:bg-gray-900 text-gray-900 dark:text-gray-100">
        <Routes>
          <Route path="/" element={<ChatPage />} />
          <Route path="/chat/:conversationId?" element={<ChatPage />} />
        </Routes>
      </div>
    </BrowserRouter>
  );
}

export default App;
