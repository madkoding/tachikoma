import { useTranslation } from 'react-i18next';
import TemplateGallery from '../components/agents/TemplateGallery';
import { getActiveTemplate } from '../stores/agentStore';

export default function AgentsPage() {
  const { t } = useTranslation();
  const active = getActiveTemplate();

  return (
    <div className="h-full overflow-auto p-4 space-y-6">
      <div>
        <h1 className="text-3xl font-bold neon-cyan">{t('templates.title')}</h1>
        <p className="text-cyber-cyan/70 mt-1 tracking-wider text-sm">{t('templates.subtitle')}</p>
      </div>

      {active && (
        <div className="cyber-card p-4 rounded-xl border border-cyber-green/40">
          <p className="text-xs uppercase tracking-widest text-cyber-green/70 mb-1">
            {t('templates.active')}
          </p>
          <p className="text-lg font-medium">
            {active.icon} {active.name}
          </p>
        </div>
      )}

      <TemplateGallery />
    </div>
  );
}
