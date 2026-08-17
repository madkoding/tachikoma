import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { PLUGIN_CATALOG, usePluginStore } from '../stores/pluginStore';
import { useToastStore } from '../stores/toastStore';

const TYPE_ICON: Record<string, string> = {
  tool: '\u{1F527}',
  memory_connector: '\u{1F5C4}\u{FE0F}',
  model_adapter: '\u{1F9E0}',
};

export default function PluginsPage() {
  const { t } = useTranslation();
  const installed = usePluginStore((s) => s.installed);
  const install = usePluginStore((s) => s.install);
  const uninstall = usePluginStore((s) => s.uninstall);
  const toast = useToastStore((s) => s.toast);

  const toggle = (id: string) => {
    if (installed.includes(id)) {
      uninstall(id);
      toast(t('plugins.uninstalled'), 'info');
    } else {
      install(id);
      toast(t('plugins.installed'), 'success');
    }
  };

  return (
    <div className="h-full overflow-auto p-4 space-y-6">
      <div>
        <h1 className="text-3xl font-bold neon-cyan">{t('plugins.title')}</h1>
        <p className="text-cyber-cyan/70 mt-1 tracking-wider text-sm">{t('plugins.subtitle')}</p>
      </div>

      <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
        {PLUGIN_CATALOG.map((plugin) => {
          const isInstalled = installed.includes(plugin.id);
          return (
            <div
              key={plugin.id}
              className={clsx(
                'cyber-card p-4 rounded-xl border',
                isInstalled
                  ? 'border-cyber-green/60 shadow-[0_0_15px_rgba(0,255,136,0.2)]'
                  : 'border-cyber-cyan/20'
              )}
            >
              <div className="flex items-start justify-between mb-2">
                <div className="flex items-center gap-2">
                  <span className="text-2xl">{TYPE_ICON[plugin.type] ?? '\u{1F9E0}'}</span>
                  <div>
                    <p className="font-medium text-cyber-cyan">{plugin.name}</p>
                    <p className="text-[10px] font-mono text-cyber-cyan/50">v{plugin.version}</p>
                  </div>
                </div>
                <span className="text-[10px] px-2 py-0.5 rounded border border-cyber-magenta/40 text-cyber-magenta">
                  {plugin.type.toUpperCase()}
                </span>
              </div>

              <p className="text-xs text-cyber-cyan/60 mb-3">{plugin.description}</p>

              <div className="text-[10px] font-mono text-cyber-cyan/50 mb-3">
                {plugin.tools.length > 0 ? (
                  <span>TOOLS: {plugin.tools.join(', ')}</span>
                ) : (
                  <span>NO TOOLS</span>
                )}
              </div>

              <button
                onClick={() => toggle(plugin.id)}
                className={clsx(
                  'w-full py-1.5 rounded text-sm font-medium border transition-all',
                  isInstalled
                    ? 'border-cyber-green/50 text-cyber-green hover:bg-cyber-green/10'
                    : 'border-cyber-cyan/40 text-cyber-cyan hover:bg-cyber-cyan/10'
                )}
              >
                {isInstalled ? t('plugins.uninstall') : t('plugins.install')}
              </button>
            </div>
          );
        })}
      </div>
    </div>
  );
}
