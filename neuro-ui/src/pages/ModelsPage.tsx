import { useTranslation } from 'react-i18next';
import { useEffect, useState } from 'react';
import clsx from 'clsx';
import { MODEL_CATALOG, OPENAI_CATALOG, OLLAMA_CLOUD_CATALOG, useModelsStore } from '../stores/modelsStore';
import { useOnboardingStore } from '../stores/onboardingStore';

export default function ModelsPage() {
  const { t } = useTranslation();
  const installed = useModelsStore((s) => s.installed);
  const downloading = useModelsStore((s) => s.downloading);
  const install = useModelsStore((s) => s.install);
  const activeModel = useModelsStore((s) => s.activeModel);
  const setActive = useModelsStore((s) => s.setActive);
  const compatible = useModelsStore((s) => s.compatible);
  const provider = useModelsStore((s) => s.provider);
  const detectProvider = useModelsStore((s) => s.detectProvider);

  const modelSize = useOnboardingStore((s) => s.modelSize);
  const [onlyCompatible, setOnlyCompatible] = useState(false);
  const vramGb = modelSize === 'large' ? 16 : modelSize === 'medium' ? 8 : 4;

  useEffect(() => {
    void detectProvider();
  }, [detectProvider]);

  const catalog =
    provider === 'openai' ? OPENAI_CATALOG
    : provider === 'ollama_cloud' ? OLLAMA_CLOUD_CATALOG
    : MODEL_CATALOG;
  const filtered = onlyCompatible
    ? catalog.filter((m) => compatible(m, vramGb))
    : catalog;

  return (
    <div className="h-full overflow-auto p-4 space-y-6">
      <div className="flex items-center justify-between flex-wrap gap-3">
        <div>
          <h1 className="text-3xl font-bold neon-cyan">{t('models.title')}</h1>
          <p className="text-cyber-cyan/70 mt-1 tracking-wider text-sm">{t('models.subtitle')}</p>
        </div>
        <label className="flex items-center gap-2 text-sm text-cyber-cyan/70 cursor-pointer">
          <input
            type="checkbox"
            checked={onlyCompatible}
            onChange={(e) => setOnlyCompatible(e.target.checked)}
            className="accent-cyber-cyan"
          />
          {t('models.compatibleOnly')}
        </label>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {filtered.map((model) => {
          const isInstalled = installed.includes(model.ollama_name);
          const isActive = activeModel === model.ollama_name;
          const progress = downloading[model.ollama_name];
          return (
            <div
              key={model.id}
              className={clsx(
                'cyber-card p-4 rounded-xl border',
                isActive ? 'border-cyber-green/60 shadow-[0_0_15px_rgba(0,255,136,0.2)]' : 'border-cyber-cyan/20'
              )}
            >
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <span className="text-2xl">{model.icon}</span>
                  <div>
                    <p className="font-medium text-cyber-cyan">{model.name}</p>
                    <p className="text-[10px] font-mono text-cyber-cyan/50">{model.license}</p>
                  </div>
                </div>
                <span className="text-[10px] px-2 py-0.5 rounded border border-cyber-magenta/40 text-cyber-magenta">
                  {model.size_tier.toUpperCase()}
                </span>
              </div>

              <p className="text-xs text-cyber-cyan/60 mb-3">{model.description}</p>

              <div className="text-[10px] font-mono text-cyber-cyan/50 mb-3 space-y-0.5">
                <div className="flex justify-between">
                  <span>SIZE</span>
                  <span>{formatBytes(model.size_bytes)}</span>
                </div>
                <div className="flex justify-between">
                  <span>VRAM</span>
                  <span>≥ {model.vram_required_gb} GB</span>
                </div>
                <div className="flex justify-between">
                  <span>TAGS</span>
                  <span>{model.tags.join(', ')}</span>
                </div>
              </div>

              {typeof progress === 'number' && progress < 100 ? (
                <div className="h-1.5 w-full bg-black/40 rounded overflow-hidden">
                  <div className="h-full bg-cyber-cyan" style={{ width: `${progress}%` }} />
                </div>
              ) : isInstalled ? (
                <button
                  onClick={() => setActive(model.ollama_name)}
                  className={clsx(
                    'w-full py-1.5 rounded text-sm font-medium border transition-all',
                    isActive
                      ? 'border-cyber-green bg-cyber-green/10 text-cyber-green'
                      : 'border-cyber-cyan/40 text-cyber-cyan hover:bg-cyber-cyan/10'
                  )}
                >
                  {isActive ? t('models.active') : t('models.activate')}
                </button>
              ) : (
                <button
                  onClick={() => void install(model)}
                  className="w-full py-1.5 rounded text-sm font-medium border border-cyber-cyan/40 text-cyber-cyan hover:bg-cyber-cyan/10 transition-all"
                >
                  {t('models.download')}
                </button>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function formatBytes(bytes: number): string {
  const gb = bytes / 1024 / 1024 / 1024;
  return gb >= 1 ? `${gb.toFixed(1)} GB` : `${Math.round(bytes / 1024 / 1024)} MB`;
}
