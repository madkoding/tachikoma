import { useTranslation } from 'react-i18next';

interface GraphHeaderProps {
  readonly nodeCount: number;
  readonly linkCount: number;
}

export default function GraphHeader({ nodeCount, linkCount }: GraphHeaderProps) {
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
