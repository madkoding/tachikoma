import { Outlet } from 'react-router-dom';
import MainSidebar from './MainSidebar';
import ToastContainer from '../common/ToastContainer';
import StatusBar from '../common/StatusBar';

export default function Layout() {
  return (
    <div className="h-screen bg-cyber-bg flex overflow-hidden">
      {/* Main Navigation Sidebar */}
      <MainSidebar />

      {/* Main Content */}
      <main className="flex-1 flex flex-col h-full overflow-hidden">
        <div className="flex-1 overflow-hidden">
          <Outlet />
        </div>
        <StatusBar />
      </main>

      {/* Global toast notifications */}
      <ToastContainer />
    </div>
  );
}
