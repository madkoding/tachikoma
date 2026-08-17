import { useTranslation } from 'react-i18next';
import clsx from 'clsx';
import { AGENT_TEMPLATES, AgentTemplate, useAgentStore } from '../../stores/agentStore';

interface Props {
  /** When true, selecting a template sets it as the active agent. */
  selectable?: boolean;
  /** Optional external selection state (e.g. wizard). */
  selectedId?: string | null;
  onSelect?: (template: AgentTemplate) => void;
}

export default function TemplateGallery({ selectable = true, selectedId, onSelect }: Props) {
  const { t } = useTranslation();
  const activeId = useAgentStore((s) => s.activeAgentId);
  const setActiveAgent = useAgentStore((s) => s.setActiveAgent);

  const isSelected = (id: string) =>
    (selectedId !== undefined ? selectedId : activeId) === id;

  const handleSelect = (template: AgentTemplate) => {
    if (selectable) setActiveAgent(template.id);
    onSelect?.(template);
  };

  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
      {AGENT_TEMPLATES.map((template) => (
        <button
          key={template.id}
          onClick={() => handleSelect(template)}
          className={clsx(
            'text-left p-4 rounded-xl border transition-all',
            isSelected(template.id)
              ? 'border-cyber-cyan bg-cyber-cyan/10 shadow-[0_0_15px_rgba(0,245,255,0.2)]'
              : 'border-black/40 hover:border-cyber-cyan/50'
          )}
        >
          <div className="flex items-center gap-3 mb-2">
            <span className="text-2xl">{template.icon}</span>
            <div>
              <h3 className="font-medium text-cyber-cyan">{template.name}</h3>
              <p className="text-[10px] font-mono text-cyber-cyan/50">
                {template.tools.length > 0 ? template.tools.join(' · ') : t('templates.noTools')}
              </p>
            </div>
          </div>
          <p className="text-xs text-cyber-cyan/60">{template.description}</p>
        </button>
      ))}
    </div>
  );
}
