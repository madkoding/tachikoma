import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { graphApi, systemApi } from '../api/client';
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip } from 'recharts';

const COLORS = ['#0ea5e9', '#22c55e', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899'];

export default function DashboardPage() {
  const { t } = useTranslation();

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['graph-stats'],
    queryFn: graphApi.getStats,
    refetchInterval: 30000,
  });

  const { data: health, isLoading: healthLoading } = useQuery({
    queryKey: ['system-health'],
    queryFn: systemApi.getHealth,
    refetchInterval: 10000,
  });

  if (statsLoading || healthLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-neuro-500"></div>
      </div>
    );
  }

  const memoryTypeData = stats?.nodes_by_type
    ? Object.entries(stats.nodes_by_type).map(([name, value]) => ({ name, value }))
    : [];

  const relationData = stats?.edges_by_relation
    ? Object.entries(stats.edges_by_relation).map(([name, value]) => ({ name, value }))
    : [];

  return (
    <div className="space-y-8">
      {/* Header */}
      <div>
        <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
          {t('dashboard.title')}
        </h1>
        <p className="text-gray-600 dark:text-gray-400 mt-1">
          {t('dashboard.subtitle')}
        </p>
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <StatCard
          title={t('dashboard.totalMemories')}
          value={stats?.total_nodes ?? 0}
          icon={<MemoryIcon />}
          color="neuro"
        />
        <StatCard
          title={t('dashboard.totalRelations')}
          value={stats?.total_edges ?? 0}
          icon={<LinkIcon />}
          color="green"
        />
        <StatCard
          title={t('dashboard.avgConnections')}
          value={stats?.avg_connections?.toFixed(2) ?? '0'}
          icon={<ChartIcon />}
          color="purple"
        />
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-6">
        {/* Memory Types Pie Chart */}
        <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
          <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
            {t('dashboard.memoryTypes')}
          </h3>
          {memoryTypeData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={memoryTypeData}
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  fill="#8884d8"
                  dataKey="value"
                  label={({ name, percent }) => `${name} (${(percent * 100).toFixed(0)}%)`}
                >
                  {memoryTypeData.map((_, index) => (
                    <Cell key={`cell-${index}`} fill={COLORS[index % COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[250px] flex items-center justify-center text-gray-400">
              No data available
            </div>
          )}
        </div>

        {/* Relations Bar Chart */}
        <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
          <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
            Relations Distribution
          </h3>
          {relationData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={relationData}>
                <XAxis dataKey="name" tick={{ fontSize: 10 }} angle={-45} textAnchor="end" height={80} />
                <YAxis />
                <Tooltip />
                <Bar dataKey="value" fill="#0ea5e9" />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[250px] flex items-center justify-center text-gray-400">
              No data available
            </div>
          )}
        </div>
      </div>

      {/* System Health */}
      <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
        <h3 className="text-lg font-semibold mb-4 text-gray-900 dark:text-white">
          {t('dashboard.systemHealth')}
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <HealthStatus
            name={t('dashboard.ollamaStatus')}
            online={health?.ollama ?? false}
            t={t}
          />
          <HealthStatus
            name={t('dashboard.surrealStatus')}
            online={health?.surrealdb ?? false}
            t={t}
          />
          <HealthStatus
            name={t('dashboard.searxngStatus')}
            online={health?.searxng ?? false}
            t={t}
          />
        </div>
        {health && (
          <div className="mt-4 pt-4 border-t border-gray-200 dark:border-gray-700">
            <p className="text-sm text-gray-600 dark:text-gray-400">
              Memory Usage: {health.memory_usage_mb.toFixed(0)} MB | 
              Uptime: {formatUptime(health.uptime_seconds)}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

interface StatCardProps {
  title: string;
  value: number | string;
  icon: React.ReactNode;
  color: 'neuro' | 'green' | 'purple';
}

function StatCard({ title, value, icon, color }: StatCardProps) {
  const colorClasses = {
    neuro: 'bg-neuro-50 dark:bg-neuro-900/20 text-neuro-500',
    green: 'bg-green-50 dark:bg-green-900/20 text-green-500',
    purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-500',
  };

  return (
    <div className="bg-white dark:bg-gray-800 rounded-xl p-6 shadow-sm">
      <div className="flex items-center gap-4">
        <div className={`p-3 rounded-lg ${colorClasses[color]}`}>
          {icon}
        </div>
        <div>
          <p className="text-sm text-gray-600 dark:text-gray-400">{title}</p>
          <p className="text-2xl font-bold text-gray-900 dark:text-white">{value}</p>
        </div>
      </div>
    </div>
  );
}

interface HealthStatusProps {
  name: string;
  online: boolean;
  t: (key: string) => string;
}

function HealthStatus({ name, online, t }: HealthStatusProps) {
  return (
    <div className="flex items-center justify-between p-4 rounded-lg bg-gray-50 dark:bg-gray-700">
      <span className="text-gray-700 dark:text-gray-300">{name}</span>
      <span className={`flex items-center gap-2 ${online ? 'text-green-500' : 'text-red-500'}`}>
        <span className={`w-2 h-2 rounded-full ${online ? 'bg-green-500' : 'bg-red-500'}`}></span>
        {online ? t('dashboard.online') : t('dashboard.offline')}
      </span>
    </div>
  );
}

function formatUptime(seconds: number): string {
  const days = Math.floor(seconds / 86400);
  const hours = Math.floor((seconds % 86400) / 3600);
  const minutes = Math.floor((seconds % 3600) / 60);
  if (days > 0) return `${days}d ${hours}h`;
  if (hours > 0) return `${hours}h ${minutes}m`;
  return `${minutes}m`;
}

function MemoryIcon() {
  return (
    <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M9 3v2m6-2v2M9 19v2m6-2v2M5 9H3m2 6H3m18-6h-2m2 6h-2M7 19h10a2 2 0 002-2V7a2 2 0 00-2-2H7a2 2 0 00-2 2v10a2 2 0 002 2zM9 9h6v6H9V9z" />
    </svg>
  );
}

function LinkIcon() {
  return (
    <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
    </svg>
  );
}

function ChartIcon() {
  return (
    <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z" />
    </svg>
  );
}
