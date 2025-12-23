export const NODE_COLORS: Record<string, string> = {
  // Core types
  fact: '#00f5ff',           // Cyan - Hechos
  preference: '#00ff88',     // Green - Preferencias
  context: '#f59e0b',        // Amber - Contexto
  conversation: '#ff00ff',   // Magenta - Conversación
  task: '#ef4444',           // Red - Tareas
  entity: '#ec4899',         // Pink - Entidades
  
  // Extended types
  procedure: '#a855f7',      // Purple - Procedimientos
  insight: '#06b6d4',        // Teal - Insights
  issue: '#f43f5e',          // Rose - Problemas
  semantic_tag: '#84cc16',   // Lime - Tags
  external_knowledge: '#0ea5e9', // Sky - Conocimiento externo
  code_snippet: '#facc15',   // Yellow - Código
  
  // New types
  goal: '#8b5cf6',           // Violet - Metas
  skill: '#14b8a6',          // Teal - Habilidades
  event: '#f97316',          // Orange - Eventos
  opinion: '#6366f1',        // Indigo - Opiniones
  experience: '#d946ef',     // Fuchsia - Experiencias
  
  general: '#9ca3af',        // Gray - General
  default: '#6b7280',        // Gray darker - Default
};

export const RELATION_COLORS: Record<string, string> = {
  related_to: '#94a3b8',
  causes: '#ef4444',
  part_of: '#22c55e',
  follows: '#8b5cf6',
  contradicts: '#dc2626',
  supports: '#10b981',
  derived_from: '#f97316',
  same_as: '#6366f1',
  context_of: '#f59e0b',
  references: '#64748b',
  supersedes: '#0891b2',
  has_property: '#eab308',
  used_for: '#3b82f6',
  capable_of: '#a855f7',
  located_in: '#ec4899',
  created_by: '#14b8a6',
  similar_to: '#818cf8',
  default: '#475569',
};

export const MEMORY_TYPES = [
  'fact', 
  'preference', 
  'context',
  'conversation',
  'task',
  'entity',
  'procedure',
  'semantic_tag',
  'issue',
  'insight',
  'external_knowledge',
  'code_snippet',
  'goal',
  'skill',
  'event',
  'opinion',
  'experience',
  'general',
];
