import { Routes, Route } from 'react-router-dom';
import Layout from './components/Layout';
import DashboardPage from './pages/DashboardPage';
import GraphPage from './pages/GraphPage';
import MemoriesPage from './pages/MemoriesPage';

export default function App() {
  return (
    <Routes>
      <Route path="/" element={<Layout />}>
        <Route index element={<DashboardPage />} />
        <Route path="graph" element={<GraphPage />} />
        <Route path="memories" element={<MemoriesPage />} />
      </Route>
    </Routes>
  );
}
