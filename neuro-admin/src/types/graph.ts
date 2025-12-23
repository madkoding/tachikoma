import type { Memory } from '../api/client';

export interface GraphNode extends Memory {
  x?: number;
  y?: number;
  z?: number;
  vx?: number;
  vy?: number;
  vz?: number;
  __highlighted?: boolean;
  __birthTime?: number; // Timestamp when node was first seen (for birth animation)
  __updateTime?: number; // Timestamp when node was last updated (for update animation)
}

export interface GraphLink {
  source: string | GraphNode;
  target: string | GraphNode;
  relation: string;
  weight: number;
}
