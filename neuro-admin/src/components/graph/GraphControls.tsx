import { useTranslation } from 'react-i18next';

interface GraphControlsProps {
  readonly searchQuery: string;
  readonly onSearchChange: (value: string) => void;
  readonly filterType: string;
  readonly onFilterChange: (value: string) => void;
  readonly memoryTypes: string[];
  readonly onResetView: () => void;
}

export default function GraphControls({
  searchQuery,
  onSearchChange,
  filterType,
  onFilterChange,
  memoryTypes,
  onResetView,
}: GraphControlsProps) {
  const { t } = useTranslation();

  return (
    <div className="flex flex-wrap items-center gap-2 md:gap-4 cyber-card mx-2 md:mx-4 p-2 md:p-4 shrink-0">
      <div className="flex-1 min-w-[150px] md:min-w-[200px]">
        <input
          type="text"
          placeholder={t('graph.search')}
          value={searchQuery}
          onChange={(e) => onSearchChange(e.target.value)}
          className="cyber-input w-full text-sm"
        />
      </div>

      <div className="w-auto">
        <select
          value={filterType}
          onChange={(e) => onFilterChange(e.target.value)}
          className="cyber-input text-sm"
        >
          <option value="all">{t('graph.allTypes')}</option>
          {memoryTypes.map((type) => (
            <option key={type} value={type}>
              {t(`graph.types.${type}`, type)}
            </option>
          ))}
        </select>
      </div>

      <button onClick={onResetView} className="cyber-button p-2" title={t('graph.resetView')}>
        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6"
          />
        </svg>
      </button>
    </div>
  );
}
