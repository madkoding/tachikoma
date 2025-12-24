import { Outlet } from 'react-router-dom';
import MainSidebar from './MainSidebar';

export default function Layout() {
  return (
    <div className="h-screen bg-cyber-bg flex overflow-hidden">
      {/* Main Navigation Sidebar */}
      <MainSidebar />

      {/* Main Content */}
      <main className="flex-1 flex flex-col h-full overflow-hidden">
        <Outlet />
      </main>
    </div>
  );
}
