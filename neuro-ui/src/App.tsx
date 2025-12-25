import { BrowserRouter, Routes, Route } from 'react-router-dom';
import { useEffect } from 'react';
import { Layout } from './components/layout';
import { NoConnectionScreen } from './components/NoConnectionScreen';
import { useConnectionStatus } from './hooks/useConnectionStatus';
import ChatPage from './pages/ChatPage';
import ChecklistsPage from './pages/ChecklistsPage';
import KanbanPage from './pages/KanbanPage';
import NotesPage from './pages/NotesPage';
import DocsPage from './pages/DocsPage';
import CalendarPage from './pages/CalendarPage';
import PomodoroPage from './pages/PomodoroPage';
import MusicPage from './pages/MusicPage';
import ImagesPage from './pages/ImagesPage';
import { MiniPlayer, AudioPlayer } from './components/music';
import { useThemeStore } from './stores/themeStore';

function App() {
  const { theme } = useThemeStore();
  const { isConnected, isChecking, error, retry } = useConnectionStatus();

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
          <Route element={<Layout />}>
            <Route path="/" element={<ChatPage />} />
            <Route path="/chat/:conversationId?" element={<ChatPage />} />
            <Route path="/checklists" element={<ChecklistsPage />} />
            <Route path="/kanban" element={<KanbanPage />} />
            <Route path="/notes" element={<NotesPage />} />
            <Route path="/docs" element={<DocsPage />} />
            <Route path="/calendar" element={<CalendarPage />} />
            <Route path="/pomodoro" element={<PomodoroPage />} />
            <Route path="/music" element={<MusicPage />} />
            <Route path="/images" element={<ImagesPage />} />
          </Route>
        </Routes>
        
        {/* Global mini player - shows on all pages except /music */}
        <MiniPlayer />
        
        {/* Global audio player - always mounted for continuous playback */}
        <AudioPlayer />

        {/* No connection screen - covers everything when backend is unavailable */}
        {!isConnected && (
          <NoConnectionScreen 
            error={error}
            onRetry={retry}
            isChecking={isChecking}
          />
        )}
      </div>
    </BrowserRouter>
  );
}

export default App;
