import { Routes, Route } from 'react-router-dom';
import { lazy, Suspense } from 'react';
import Layout from './components/Layout';
import DashboardPage from './pages/DashboardPage';

const GraphPage = lazy(() => import('./pages/GraphPage'));

function GraphPageLoader() {
  return (
    <div className="h-full flex items-center justify-center">
      <div className="w-8 h-8 border-2 border-cyber-cyan border-t-transparent rounded-full animate-spin"></div>
    </div>
  );
}

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<DashboardPage />} />
        <Route path="graph" element={
          <Suspense fallback={<GraphPageLoader />}>
            <GraphPage />
          </Suspense>
        } />
      </Route>
    </Routes>
  );
}
