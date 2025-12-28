import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { graphApi, systemApi, ModelInfo } from '../api/client';
import { PieChart, Pie, Cell, ResponsiveContainer, BarChart, Bar, XAxis, YAxis, Tooltip } from 'recharts';

const CYBER_COLORS = ['#00f5ff', '#ff00ff', '#00ff88', '#ffcc00', '#ff3366', '#9966ff'];

export default function DashboardPage() {
  const { t } = useTranslation();

  const { data: stats, isLoading: statsLoading } = useQuery({
    queryKey: ['graph-stats'],
    queryFn: graphApi.getStats,
    refetchInterval: 60000,
  });

  const { data: health, isLoading: healthLoading } = useQuery({
    queryKey: ['system-health'],
    queryFn: systemApi.getHealth,
    refetchInterval: 60000,
  });

  const { data: models, isLoading: modelsLoading } = useQuery({
    queryKey: ['llm-models'],
    queryFn: systemApi.getModels,
    refetchInterval: 60000,
  });

  const isLoading = statsLoading || healthLoading || modelsLoading;

  const memoryTypeData = stats?.nodes_by_type
    ? Object.entries(stats.nodes_by_type).map(([name, value]) => ({ name, value }))
    : [];

  const relationData = stats?.edges_by_type
    ? Object.entries(stats.edges_by_type).map(([name, value]) => ({ name, value }))
    : [];

  return (
    <div className="h-full overflow-auto p-4 space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold neon-cyan glitch" data-text={t('dashboard.title')}>
            {t('dashboard.title')}
          </h1>
          <p className="text-cyber-cyan/70 mt-1 tracking-wider text-sm">
            {t('dashboard.subtitle')}
          </p>
        </div>
        {isLoading && (
          <div className="w-6 h-6 border-2 border-cyber-cyan border-t-transparent rounded-full animate-spin"></div>
        )}
      </div>

      {/* Stats Cards */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
        <CyberStatCard
          title={t('dashboard.totalMemories')}
          value={stats?.total_nodes ?? 0}
          icon={<MemoryIcon />}
          color="cyan"
        />
        <CyberStatCard
          title={t('dashboard.totalRelations')}
          value={stats?.total_edges ?? 0}
          icon={<LinkIcon />}
          color="green"
        />
        <CyberStatCard
          title={t('dashboard.avgConnections')}
          value={stats?.avg_connections?.toFixed(2) ?? '0'}
          icon={<ChartIcon />}
          color="magenta"
        />
      </div>

      {/* LLM Models Section */}
      <div className="cyber-card">
        <h3 className="text-lg font-semibold mb-4 neon-cyan flex items-center gap-2">
          <ModelIcon />
          {t('dashboard.llmModels')}
        </h3>
        {models && models.length > 0 ? (
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            {models.map((model) => (
              <ModelCard key={model.id} model={model} />
            ))}
          </div>
        ) : (
          <div className="text-center py-8 text-cyber-cyan/50">
            <p className="glitch" data-text={t('dashboard.noModels')}>{t('dashboard.noModels')}</p>
          </div>
        )}
      </div>

      {/* Charts */}
      <div className="grid grid-cols-1 sm:grid-cols-2 gap-6">
        {/* Memory Types Pie Chart */}
        <div className="cyber-card">
          <h3 className="text-lg font-semibold mb-4 neon-cyan">
            {t('dashboard.memoryTypes')} | {t('dashboard.distribution')}
          </h3>
          {memoryTypeData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <PieChart>
                <Pie
                  data={memoryTypeData}
                  cx="50%"
                  cy="50%"
                  outerRadius={80}
                  fill="#00f5ff"
                  dataKey="value"
                  label={({ name, percent }) => `${name} (${(percent * 100).toFixed(0)}%)`}
                  stroke="#0a0e14"
                  strokeWidth={2}
                >
                  {memoryTypeData.map((entry, index) => (
                    <Cell key={entry.name} fill={CYBER_COLORS[index % CYBER_COLORS.length]} />
                  ))}
                </Pie>
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: '#0a0e14', 
                    border: '1px solid #00f5ff',
                    borderRadius: '4px',
                    color: '#00f5ff'
                  }}
                />
              </PieChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[250px] flex items-center justify-center text-cyber-cyan/40">
              <span className="glitch" data-text={t('dashboard.awaitingData')}>{t('dashboard.awaitingData')}</span>
            </div>
          )}
        </div>

        {/* Relations Bar Chart */}
        <div className="cyber-card">
          <h3 className="text-lg font-semibold mb-4 neon-magenta">
            {t('dashboard.relations')}
          </h3>
          {relationData.length > 0 ? (
            <ResponsiveContainer width="100%" height={250}>
              <BarChart data={relationData}>
                <XAxis 
                  dataKey="name" 
                  tick={{ fontSize: 10, fill: '#00f5ff' }} 
                  angle={-45} 
                  textAnchor="end" 
                  height={80}
                  stroke="#00f5ff"
                />
                <YAxis stroke="#00f5ff" tick={{ fill: '#00f5ff' }} />
                <Tooltip 
                  contentStyle={{ 
                    backgroundColor: '#0a0e14', 
                    border: '1px solid #ff00ff',
                    borderRadius: '4px',
                    color: '#ff00ff'
                  }}
                />
                <Bar dataKey="value" fill="#ff00ff" />
              </BarChart>
            </ResponsiveContainer>
          ) : (
            <div className="h-[250px] flex flex-col items-center justify-center text-cyber-magenta/40">
              <span className="glitch text-sm" data-text={t('dashboard.noRelationsYet')}>{t('dashboard.noRelationsYet')}</span>
              <span className="text-xs mt-2 opacity-60">{t('dashboard.createConnections')}</span>
            </div>
          )}
        </div>
      </div>

      {/* System Health */}
      <div className="cyber-card">
        <h3 className="text-lg font-semibold mb-4 neon-green flex items-center gap-2">
          <HeartbeatIcon />
          {t('dashboard.systemHealth')} | {t('dashboard.diagnostics')}
        </h3>
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <CyberHealthStatus
            name={t('dashboard.ollamaStatus')}
            online={health?.services?.llm === 'healthy'}
            icon={<BrainIcon />}
            onlineText={t('dashboard.online')}
            offlineText={t('dashboard.offline')}
          />
          <CyberHealthStatus
            name={t('dashboard.surrealStatus')}
            online={health?.services?.database === 'healthy'}
            icon={<DatabaseIcon />}
            onlineText={t('dashboard.online')}
            offlineText={t('dashboard.offline')}
          />
          <CyberHealthStatus
            name={t('dashboard.searxngStatus')}
            online={health?.services?.search === 'healthy'}
            icon={<SearchIcon />}
            onlineText={t('dashboard.online')}
            offlineText={t('dashboard.offline')}
          />
        </div>
        {health && (
          <div className="mt-4 pt-4 border-t border-cyber-cyan/30">
            <p className="text-sm text-cyber-cyan/70 font-mono">
              <span className="neon-cyan">{t('dashboard.version')}:</span> {health.version} | 
              <span className="neon-green ml-2">{t('dashboard.uptime')}:</span> {formatUptime(health.uptime_seconds)}
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

interface CyberStatCardProps {
  readonly title: string;
  readonly value: number | string;
  readonly icon: React.ReactNode;
  readonly color: 'cyan' | 'green' | 'magenta';
}

function CyberStatCard({ title, value, icon, color }: Readonly<CyberStatCardProps>) {
  const colorClasses = {
    cyan: 'border-cyber-cyan text-cyber-cyan shadow-[0_0_15px_rgba(0,245,255,0.3)]',
    green: 'border-cyber-green text-cyber-green shadow-[0_0_15px_rgba(0,255,136,0.3)]',
    magenta: 'border-cyber-magenta text-cyber-magenta shadow-[0_0_15px_rgba(255,0,255,0.3)]',
  };

  const glowClass = {
    cyan: 'neon-cyan',
    green: 'neon-green',
    magenta: 'neon-magenta',
  };

  return (
    <div className={`cyber-card border-l-4 rounded-xl ${colorClasses[color]} hover:scale-105 transition-transform duration-300`}>
      <div className="flex items-center gap-4">
        <div className={`p-3 rounded-lg bg-cyber-bg/50 ${colorClasses[color]}`}>
          {icon}
        </div>
        <div>
          <p className="text-xs uppercase tracking-widest text-cyber-cyan/60 mb-1">{title}</p>
          <p className={`text-3xl font-bold ${glowClass[color]} font-mono`}>{value}</p>
        </div>
      </div>
    </div>
  );
}

interface ModelCardProps {
  readonly model: ModelInfo;
}

function ModelCard({ model }: Readonly<ModelCardProps>) {
  const formatBytes = (bytes?: number): string => {
    if (!bytes) return 'N/A';
    const gb = bytes / (1024 * 1024 * 1024);
    if (gb >= 1) return `${gb.toFixed(2)} GB`;
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(2)} MB`;
  };

  const formatParams = (params?: number): string => {
    if (!params) return 'N/A';
    if (params >= 1_000_000_000) return `${(params / 1_000_000_000).toFixed(1)}B`;
    if (params >= 1_000_000) return `${(params / 1_000_000).toFixed(1)}M`;
    return `${params}`;
  };

  return (
    <div className="cyber-card rounded-xl border border-cyber-cyan/30 hover:border-cyber-cyan transition-colors duration-300 hover:shadow-[0_0_20px_rgba(0,245,255,0.2)]">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="text-xl">{model.is_embedding_model ? '🔢' : '🧠'}</span>
          <span className="font-mono text-cyber-cyan font-semibold truncate max-w-[150px]">
            {model.name}
          </span>
        </div>
        <span className={`text-xs px-2 py-1 rounded-lg ${
          model.is_embedding_model 
            ? 'bg-cyber-magenta/20 text-cyber-magenta border border-cyber-magenta/50' 
            : 'bg-cyber-green/20 text-cyber-green border border-cyber-green/50'
        }`}>
          {model.is_embedding_model ? 'EMBED' : 'LLM'}
        </span>
      </div>
      <div className="space-y-2 text-xs font-mono">
        <div className="flex justify-between text-cyber-cyan/70">
          <span>SIZE:</span>
          <span className="neon-cyan">{formatBytes(model.size_bytes)}</span>
        </div>
        {model.parameters && (
          <div className="flex justify-between text-cyber-cyan/70">
            <span>PARAMS:</span>
            <span className="neon-magenta">{formatParams(model.parameters)}</span>
          </div>
        )}
        {model.context_length && (
          <div className="flex justify-between text-cyber-cyan/70">
            <span>CONTEXT:</span>
            <span className="neon-green">{model.context_length.toLocaleString()}</span>
          </div>
        )}
      </div>
    </div>
  );
}

interface CyberHealthStatusProps {
  readonly name: string;
  readonly online: boolean;
  readonly icon: React.ReactNode;
  readonly onlineText?: string;
  readonly offlineText?: string;
}

function CyberHealthStatus({ name, online, icon, onlineText = 'ONLINE', offlineText = 'OFFLINE' }: Readonly<CyberHealthStatusProps>) {
  return (
    <div className={`flex items-center justify-between p-4 rounded-xl border transition-all duration-300 ${
      online 
        ? 'border-cyber-green/50 bg-cyber-green/10 shadow-[0_0_10px_rgba(0,255,136,0.2)]' 
        : 'border-red-500/50 bg-red-500/10 shadow-[0_0_10px_rgba(255,0,0,0.2)]'
    }`}>
      <div className="flex items-center gap-3">
        <span className={online ? 'text-cyber-green' : 'text-red-500'}>{icon}</span>
        <span className="text-cyber-cyan font-mono text-sm">{name}</span>
      </div>
      <div className="flex items-center gap-2">
        <span className={`w-2 h-2 rounded-full ${
          online 
            ? 'bg-cyber-green animate-pulse shadow-[0_0_10px_#00ff88]' 
            : 'bg-red-500 animate-pulse shadow-[0_0_10px_#ff0000]'
        }`}></span>
        <span className={`text-xs font-mono uppercase ${online ? 'neon-green' : 'text-red-500'}`}>
          {online ? onlineText : offlineText}
        </span>
      </div>
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

// Icons
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

function ModelIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M19.428 15.428a2 2 0 00-1.022-.547l-2.387-.477a6 6 0 00-3.86.517l-.318.158a6 6 0 01-3.86.517L6.05 15.21a2 2 0 00-1.806.547M8 4h8l-1 1v5.172a2 2 0 00.586 1.414l5 5c1.26 1.26.367 3.414-1.415 3.414H4.828c-1.782 0-2.674-2.154-1.414-3.414l5-5A2 2 0 009 10.172V5L8 4z" />
    </svg>
  );
}

function HeartbeatIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M4.318 6.318a4.5 4.5 0 000 6.364L12 20.364l7.682-7.682a4.5 4.5 0 00-6.364-6.364L12 7.636l-1.318-1.318a4.5 4.5 0 00-6.364 0z" />
    </svg>
  );
}

function BrainIcon() {
  return (
    <svg className="w-5 h-5" fill="currentColor" stroke="currentColor" viewBox="0 0 100 100">
      <g fill="currentColor">
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(0 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(45 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(90 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(135 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(180 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(225 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(270 50 50)"/>
        <rect x="42" y="2" width="16" height="14" rx="1" transform="rotate(315 50 50)"/>
      </g>
      <circle cx="50" cy="50" r="38" fill="none" stroke="currentColor" strokeWidth="6"/>
      <circle cx="50" cy="50" r="28" fill="none" stroke="currentColor" strokeWidth="4"/>
      <circle cx="50" cy="50" r="18" fill="none" stroke="currentColor" strokeWidth="3"/>
      <circle cx="50" cy="50" r="6" fill="currentColor"/>
    </svg>
  );
}

function DatabaseIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M4 7v10c0 2.21 3.582 4 8 4s8-1.79 8-4V7M4 7c0 2.21 3.582 4 8 4s8-1.79 8-4M4 7c0-2.21 3.582-4 8-4s8 1.79 8 4m0 5c0 2.21-3.582 4-8 4s-8-1.79-8-4" />
    </svg>
  );
}

function SearchIcon() {
  return (
    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} 
            d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
    </svg>
  );
}
