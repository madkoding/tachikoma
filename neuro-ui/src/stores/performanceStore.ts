import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type PerformanceLevel = 'high' | 'medium' | 'low' | 'minimal';

interface PerformanceSettings {
  // Visual effects
  enableSpectrumAnalyzer: boolean;
  enableBlurEffects: boolean;
  enableGlowEffects: boolean;
  enableReflections: boolean;
  enableScanlines: boolean;
  enableMarqueeAnimation: boolean;
  // Performance tuning
  spectrumFPS: number;       // Target FPS for spectrum analyzer
  spectrumBarCount: number;  // Number of bars in spectrum
}

interface PerformanceState {
  level: PerformanceLevel;
  autoDetect: boolean;
  settings: PerformanceSettings;
  
  // Metrics (for auto-detection)
  currentFPS: number;
  fpsHistory: number[];
  
  // Actions
  setLevel: (level: PerformanceLevel) => void;
  setAutoDetect: (auto: boolean) => void;
  updateFPS: (fps: number) => void;
  getSettings: () => PerformanceSettings;
}

// Presets for each performance level
const PERFORMANCE_PRESETS: Record<PerformanceLevel, PerformanceSettings> = {
  high: {
    enableSpectrumAnalyzer: true,
    enableBlurEffects: true,
    enableGlowEffects: true,
    enableReflections: true,
    enableScanlines: true,
    enableMarqueeAnimation: true,
    spectrumFPS: 30,
    spectrumBarCount: 32,
  },
  medium: {
    enableSpectrumAnalyzer: true,
    enableBlurEffects: false,  // Blur is expensive
    enableGlowEffects: true,
    enableReflections: false,
    enableScanlines: false,
    enableMarqueeAnimation: true,
    spectrumFPS: 20,
    spectrumBarCount: 24,
  },
  low: {
    enableSpectrumAnalyzer: true,
    enableBlurEffects: false,
    enableGlowEffects: false,
    enableReflections: false,
    enableScanlines: false,
    enableMarqueeAnimation: false,
    spectrumFPS: 15,
    spectrumBarCount: 16,
  },
  minimal: {
    enableSpectrumAnalyzer: false,  // Disable entirely
    enableBlurEffects: false,
    enableGlowEffects: false,
    enableReflections: false,
    enableScanlines: false,
    enableMarqueeAnimation: false,
    spectrumFPS: 10,
    spectrumBarCount: 8,
  },
};

// Detect initial performance level based on device
function detectInitialLevel(): PerformanceLevel {
  // Check for reduced motion preference - this is an explicit user preference
  if (typeof window !== 'undefined' && window.matchMedia('(prefers-reduced-motion: reduce)').matches) {
    return 'low';
  }
  
  // Check if mobile device
  const isMobile = typeof navigator !== 'undefined' 
    ? /Android|webOS|iPhone|iPad|iPod|BlackBerry|IEMobile|Opera Mini/i.test(navigator.userAgent)
    : false;
  
  // Mobile devices get medium by default (can handle it but save battery)
  if (isMobile) {
    return 'medium';
  }
  
  // Desktop: Default to high - let user lower if needed
  // Modern browsers on desktop can handle these effects
  return 'high';
}

export const usePerformanceStore = create<PerformanceState>()(
  persist(
    (set, get) => ({
      level: 'high', // Will be overridden by persisted or detected
      autoDetect: true,
      settings: PERFORMANCE_PRESETS['high'],
      currentFPS: 60,
      fpsHistory: [],
      
      setLevel: (level) => {
        set({
          level,
          settings: PERFORMANCE_PRESETS[level],
          autoDetect: false, // Disable auto when manually setting
        });
      },
      
      setAutoDetect: (auto) => {
        set({ autoDetect: auto });
        if (auto) {
          // Re-detect when enabling auto
          const newLevel = detectInitialLevel();
          set({
            level: newLevel,
            settings: PERFORMANCE_PRESETS[newLevel],
          });
        }
      },
      
      updateFPS: (fps) => {
        const state = get();
        
        // Keep last 30 FPS readings for display purposes only
        const newHistory = [...state.fpsHistory.slice(-29), fps];
        const avgFPS = newHistory.reduce((a, b) => a + b, 0) / newHistory.length;
        
        set({ currentFPS: Math.round(avgFPS), fpsHistory: newHistory });
        
        // Note: We no longer auto-adjust based on FPS because:
        // 1. Canvas FPS is limited by our own frameInterval setting
        // 2. Low reported FPS doesn't necessarily mean the system is struggling
        // 3. Users should have direct control over quality settings
      },
      
      getSettings: () => get().settings,
    }),
    {
      name: 'neuro-performance',
      version: 2, // Increment to reset stored settings to new defaults
      partialize: (state) => ({
        level: state.level,
        autoDetect: state.autoDetect,
      }),
      onRehydrateStorage: () => (state) => {
        // After rehydrating, apply the correct settings
        if (state) {
          if (state.autoDetect) {
            const detectedLevel = detectInitialLevel();
            state.level = detectedLevel;
            state.settings = PERFORMANCE_PRESETS[detectedLevel];
          } else {
            state.settings = PERFORMANCE_PRESETS[state.level];
          }
        }
      },
      migrate: (persistedState, version) => {
        // Migration: Reset to high quality on version upgrade
        if (version < 2) {
          return {
            level: 'high' as PerformanceLevel,
            autoDetect: false, // Let user control manually
          };
        }
        return persistedState as { level: PerformanceLevel; autoDetect: boolean };
      },
    }
  )
);

// Hook for components to get current settings
export const usePerformanceSettings = () => {
  return usePerformanceStore((state) => state.settings);
};

// Hook for conditional rendering based on performance
export const useCanRender = (feature: keyof PerformanceSettings): boolean => {
  const settings = usePerformanceSettings();
  const value = settings[feature];
  return typeof value === 'boolean' ? value : true;
};

// Performance level labels for UI
export const PERFORMANCE_LABELS: Record<PerformanceLevel, string> = {
  high: 'Alto',
  medium: 'Medio',
  low: 'Bajo',
  minimal: 'Mínimo',
};
