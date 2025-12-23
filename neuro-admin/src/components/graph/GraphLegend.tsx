import { useTranslation } from 'react-i18next';
import { NODE_COLORS, RELATION_COLORS } from '../../constants/graph';

export default function GraphLegend() {
  const { t } = useTranslation();

  return (
    <div className="flex flex-col sm:flex-row flex-wrap gap-2 md:gap-4 cyber-card mx-2 md:mx-4 mb-2 md:mb-4 p-2 md:p-4 shrink-0">
      {/* Node Types */}
      <div className="flex flex-wrap gap-2 md:gap-3 items-center flex-1">
        <span className="text-xs md:text-sm font-medium text-cyber-cyan font-mono">
          {t('graph.nodeTypes')}:
        </span>
        {Object.entries(NODE_COLORS)
          .filter(([k]) => k !== 'default')
          .map(([type, color]) => (
            <span key={type} className="flex items-center gap-1 text-xs text-cyber-cyan/70">
              <span
                className="w-2 h-2 md:w-3 md:h-3 rounded-full"
                style={{ backgroundColor: color, boxShadow: `0 0 8px ${color}` }}
              />
              <span className="hidden sm:inline">{t(`graph.types.${type}`, type)}</span>
            </span>
          ))}
      </div>

      {/* Separator */}
      <div className="hidden sm:block w-px bg-cyber-cyan/20 self-stretch" />

      {/* Relation Types */}
      <div className="flex flex-wrap gap-2 md:gap-3 items-center flex-1">
        <span className="text-xs md:text-sm font-medium text-cyber-magenta font-mono">
          {t('graph.relationTypes', 'Relations')}:
        </span>
        {Object.entries(RELATION_COLORS).map(([relation, color]) => (
          <span key={relation} className="flex items-center gap-1 text-xs text-cyber-cyan/70">
            <span
              className="w-2 h-2 md:w-3 md:h-3 rounded-full"
              style={{ backgroundColor: color, boxShadow: `0 0 8px ${color}` }}
            />
            <span className="hidden sm:inline">{t(`graph.relations.${relation}`, relation)}</span>
          </span>
        ))}
      </div>
    </div>
  );
}
