import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type ModelSize = 'small' | 'medium' | 'large';

interface OnboardingState {
  onboarded: boolean;
  modelSize: ModelSize | null;
  agentName: string | null;
  complete: (opts?: { modelSize?: ModelSize; agentName?: string }) => void;
  reset: () => void;
}

export const useOnboardingStore = create<OnboardingState>()(
  persist(
    (set) => ({
      onboarded: false,
      modelSize: null,
      agentName: null,
      complete: (opts) =>
        set((s) => ({
          onboarded: true,
          modelSize: opts?.modelSize ?? s.modelSize,
          agentName: opts?.agentName ?? s.agentName,
        })),
      reset: () => set({ onboarded: false, modelSize: null, agentName: null }),
    }),
    {
      name: 'tachikoma-onboarding',
    }
  )
);
