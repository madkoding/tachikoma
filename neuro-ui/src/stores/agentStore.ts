import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export interface AgentTemplate {
  id: string;
  name: string;
  icon: string;
  description: string;
  /** System prompt injected into every chat message for this agent. */
  systemPrompt: string;
  /** Tools the agent relies on (informational). */
  tools: string[];
  /** Minimum VRAM in GB for the recommended model. */
  minVramGb: number;
}

export const AGENT_TEMPLATES: AgentTemplate[] = [
  {
    id: 'research-assistant',
    name: 'Asistente de Investigación',
    icon: '🔍',
    description: 'Busca en la web y resume información con contexto de tu memoria.',
    systemPrompt:
      'Eres un asistente de investigación. Cuando te pregunten algo, busca información actualizada en la web y responde con un resumen claro, citando las fuentes. Usa tu memoria para recordar temas que ya investigaste.',
    tools: ['search_web', 'remember'],
    minVramGb: 4,
  },
  {
    id: 'data-analyst',
    name: 'Analista de Datos',
    icon: '📊',
    description: 'Analiza datos, genera reportes y explica resultados con claridad.',
    systemPrompt:
      'Eres un analista de datos. Explica números y tendencias de forma clara, genera reportes estructurados y sugiere visualizaciones cuando sea útil.',
    tools: ['file', 'remember'],
    minVramGb: 4,
  },
  {
    id: 'code-reviewer',
    name: 'Code Reviewer',
    icon: '💻',
    description: 'Lee código, sugiere mejoras y explica decisiones técnicas.',
    systemPrompt:
      'Eres un revisor de código senior. Analiza el código, señala problemas de calidad, seguridad y rendimiento, y sugiere mejoras concretas. Explica tus decisiones.',
    tools: ['file', 'execute_command'],
    minVramGb: 8,
  },
  {
    id: 'customer-support',
    name: 'Soporte al Cliente',
    icon: '🎧',
    description: 'Responde consultas con memoria de conversaciones y FAQ.',
    systemPrompt:
      'Eres un agente de soporte al cliente. Responde con empatía, usa tu memoria para recordar interacciones previas y ofrece soluciones paso a paso.',
    tools: ['remember'],
    minVramGb: 4,
  },
  {
    id: 'creative-writer',
    name: 'Escritor Creativo',
    icon: '✍️',
    description: 'Genera texto con un estilo personalizable y creativo.',
    systemPrompt:
      'Eres un escritor creativo. Genera texto con estilo, imaginación y coherencia. Adapta tu tono al pedido del usuario y ofrece variaciones cuando sea útil.',
    tools: [],
    minVramGb: 4,
  },
];

interface AgentState {
  activeAgentId: string | null;
  setActiveAgent: (id: string | null) => void;
}

export const useAgentStore = create<AgentState>()(
  persist(
    (set) => ({
      activeAgentId: null,
      setActiveAgent: (id) => set({ activeAgentId: id }),
    }),
    {
      name: 'tachikoma-agent',
    }
  )
);

export function getActiveTemplate(): AgentTemplate | null {
  const id = useAgentStore.getState().activeAgentId;
  return AGENT_TEMPLATES.find((t) => t.id === id) ?? null;
}
