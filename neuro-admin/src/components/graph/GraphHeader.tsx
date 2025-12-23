import { useTranslation } from 'react-i18next';

type ConnectionStatus = 'connecting' | 'connected' | 'disconnected' | 'error';

interface GraphHeaderProps {
  readonly nodeCount: number;
  readonly linkCount: number;
  readonly connectionStatus?: ConnectionStatus;
}

function ConnectionIndicator({ status }: { readonly status: ConnectionStatus }) {
  const { t } = useTranslation();
  
  const statusConfig = {
    connecting: {
      color: 'bg-yellow-500',
      pulse: true,
      text: t('graph.status.connecting', 'Conectando...'),
    },
    connected: {
      color: 'bg-green-500',
      pulse: true,
      text: t('graph.status.live', 'En vivo'),
    },
    disconnected: {
      color: 'bg-gray-500',
      pulse: false,
      text: t('graph.status.disconnected', 'Desconectado'),
    },
    error: {
      color: 'bg-red-500',
      pulse: true,
      text: t('graph.status.error', 'Error'),
    },
  };

  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-1.5 px-2 py-1 rounded-full bg-cyber-dark/50 border border-cyber-cyan/20">
      <span className="relative flex h-2 w-2">
        {config.pulse && (
          <span className={`animate-ping absolute inline-flex h-full w-full rounded-full ${config.color} opacity-75`} />
        )}
        <span className={`relative inline-flex rounded-full h-2 w-2 ${config.color}`} />
      </span>
      <span className="text-xs font-mono text-cyber-cyan/80">{config.text}</span>
    </div>
  );
}

export default function GraphHeader({ nodeCount, linkCount, connectionStatus = 'disconnected' }: GraphHeaderProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-2 p-2 md:p-3 shrink-0">
      <div className="flex items-center gap-4">
        <div>
          <h1 className="text-lg md:text-xl font-bold neon-cyan font-cyber tracking-wider">
            {t('graph.title')}
          </h1>
          <p className="text-cyber-cyan/60 font-mono text-xs hidden sm:block">
            {t('graph.subtitle')}
          </p>
        </div>
        <ConnectionIndicator status={connectionStatus} />
      </div>
      <div className="flex items-center gap-2 md:gap-4 text-xs md:text-sm">
        <span className="text-cyber-cyan/70 font-mono">
          {t('graph.nodeCount')}: <span className="neon-green">{nodeCount}</span>
        </span>
        <span className="text-cyber-cyan/70 font-mono">
          {t('graph.edgeCount')}: <span className="neon-magenta">{linkCount}</span>
        </span>
      </div>
    </div>
  );
}
