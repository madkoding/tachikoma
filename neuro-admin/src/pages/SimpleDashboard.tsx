import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { Link } from 'react-router-dom';
import { graphApi, systemApi } from '../api/client';

export default function SimpleDashboard() {
  const { t } = useTranslation();

  const { data: stats } = useQuery({
    queryKey: ['graph-stats'],
    queryFn: graphApi.getStats,
    refetchInterval: 30000,
  });

  const { data: health } = useQuery({
    queryKey: ['system-health'],
    queryFn: systemApi.getHealth,
    refetchInterval: 30000,
  });

  const { data: models } = useQuery({
    queryKey: ['llm-models'],
    queryFn: systemApi.getModels,
    refetchInterval: 30000,
  });

  const status =
    health?.status === 'healthy'
      ? 'healthy'
      : health?.status === 'degraded'
      ? 'degraded'
      : 'offline';

  const statusClass = {
    healthy: 'border-cyber-green/50 bg-cyber-green/10 text-cyber-green',
    degraded: 'border-cyber-yellow/50 bg-cyber-yellow/10 text-cyber-yellow',
    offline: 'border-cyber-red/50 bg-cyber-red/10 text-cyber-red',
  }[status];

  const statusLabel = {
    healthy: t('simple.healthy'),
    degraded: t('simple.degraded'),
    offline: t('simple.offline'),
  }[status];

  return (
    <div className="h-full overflow-auto p-4 space-y-6">
      <div>
        <h1 className="text-3xl font-bold neon-cyan">{t('simple.title')}</h1>
        <p className="text-cyber-cyan/70 mt-1 tracking-wider text-sm">{t('simple.subtitle')}</p>
      </div>

      {/* Status + 2 stats — max 5 elements on screen */}
      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
        <div className={`cyber-card p-4 rounded-xl border ${statusClass}`}>
          <p className="text-xs uppercase tracking-widest opacity-70 mb-1">
            {t('simple.resources')}
          </p>
          <p className="text-xl font-bold">{statusLabel}</p>
        </div>

        <div className="cyber-card p-4 rounded-xl border border-cyber-cyan/30">
          <p className="text-xs uppercase tracking-widest text-cyber-cyan/60 mb-1">
            {t('simple.modelsLoaded')}
          </p>
          <p className="text-3xl font-bold neon-cyan font-mono">{models?.length ?? 0}</p>
        </div>

        <div className="cyber-card p-4 rounded-xl border border-cyber-magenta/30">
          <p className="text-xs uppercase tracking-widest text-cyber-magenta/60 mb-1">
            {t('simple.activeAgents')}
          </p>
          <p className="text-3xl font-bold neon-magenta font-mono">{stats?.total_nodes ?? 0}</p>
        </div>
      </div>

      {/* Quick actions */}
      <div className="cyber-card p-4 rounded-xl">
        <h3 className="text-lg font-semibold mb-4 neon-cyan">{t('simple.quickActions')}</h3>
        <div className="grid grid-cols-1 sm:grid-cols-3 gap-3">
          <Link
            to="/graph"
            className="cyber-button text-center"
          >
            {t('simple.openGraph')}
          </Link>
          <Link
            to="/"
            className="cyber-button text-center"
          >
            {t('simple.openChat')}
          </Link>
          <button className="cyber-button text-center opacity-70">
            {t('simple.viewSettings')}
          </button>
        </div>
      </div>
    </div>
  );
}
