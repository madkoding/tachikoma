import React from 'react';
import { Gauge, Zap, Monitor, Smartphone, Sparkles } from 'lucide-react';
import { 
  usePerformanceStore, 
  PerformanceLevel, 
  PERFORMANCE_LABELS 
} from '../../stores/performanceStore';

interface PerformanceSettingsProps {
  className?: string;
}

const LEVEL_ICONS: Record<PerformanceLevel, React.ReactNode> = {
  high: <Sparkles className="w-4 h-4" />,
  medium: <Monitor className="w-4 h-4" />,
  low: <Smartphone className="w-4 h-4" />,
  minimal: <Zap className="w-4 h-4" />,
};

const LEVEL_DESCRIPTIONS: Record<PerformanceLevel, string> = {
  high: 'Todos los efectos visuales activos',
  medium: 'Sin blur, efectos de brillo activos',
  low: 'Efectos mínimos, mejor rendimiento',
  minimal: 'Sin animaciones, máximo rendimiento',
};

const LEVEL_COLORS: Record<PerformanceLevel, string> = {
  high: 'bg-green-500',
  medium: 'bg-yellow-500',
  low: 'bg-orange-500',
  minimal: 'bg-red-500',
};

export const PerformanceSettings: React.FC<PerformanceSettingsProps> = ({ className = '' }) => {
  const { level, autoDetect, currentFPS, setLevel, setAutoDetect } = usePerformanceStore();
  const levels: PerformanceLevel[] = ['high', 'medium', 'low', 'minimal'];

  return (
    <div className={`bg-gray-900/80 border border-cyan-500/30 p-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <div className="flex items-center gap-2">
          <Gauge className="w-5 h-5 text-cyan-400" />
          <h3 className="text-cyan-400 font-bold text-sm tracking-wider uppercase font-cyber">
            Rendimiento
          </h3>
        </div>
        
        {/* Current FPS indicator */}
        <div className="flex items-center gap-2">
          <div className={`w-2 h-2 rounded-full ${currentFPS > 30 ? 'bg-green-500' : currentFPS > 20 ? 'bg-yellow-500' : 'bg-red-500'}`} />
          <span className="text-xs text-gray-400 font-mono">{currentFPS} FPS</span>
        </div>
      </div>

      {/* Auto-detect toggle */}
      <div className="flex items-center justify-between mb-4 p-2 bg-gray-800/50 rounded">
        <div>
          <div className="text-sm text-white font-medium">Detección automática</div>
          <div className="text-xs text-gray-400">Detecta si es móvil o escritorio</div>
        </div>
        <button
          type="button"
          onClick={() => setAutoDetect(!autoDetect)}
          className={`relative w-12 h-6 rounded-full transition-colors ${
            autoDetect ? 'bg-cyan-500' : 'bg-gray-600'
          }`}
        >
          <div
            className={`absolute top-1 w-4 h-4 bg-white rounded-full transition-transform ${
              autoDetect ? 'translate-x-7' : 'translate-x-1'
            }`}
          />
        </button>
      </div>

      {/* Level selector */}
      <div className="space-y-2">
        <div className="text-xs text-gray-400 mb-2">Nivel de calidad visual</div>
        <div className="grid grid-cols-2 gap-2">
          {levels.map((lvl) => (
            <button
              key={lvl}
              type="button"
              onClick={() => setLevel(lvl)}
              disabled={autoDetect}
              className={`p-3 rounded border transition-all ${
                level === lvl
                  ? 'bg-cyan-500/20 border-cyan-500 text-cyan-400'
                  : autoDetect
                    ? 'bg-gray-800/30 border-gray-700 text-gray-500 cursor-not-allowed'
                    : 'bg-gray-800/50 border-gray-700 text-gray-400 hover:border-gray-500 hover:text-white'
              }`}
            >
              <div className="flex items-center gap-2 mb-1">
                {LEVEL_ICONS[lvl]}
                <span className="font-medium text-sm">{PERFORMANCE_LABELS[lvl]}</span>
                {level === lvl && (
                  <div className={`w-2 h-2 rounded-full ${LEVEL_COLORS[lvl]} ml-auto`} />
                )}
              </div>
              <div className="text-[10px] text-left opacity-70">
                {LEVEL_DESCRIPTIONS[lvl]}
              </div>
            </button>
          ))}
        </div>
      </div>

      {/* Current settings summary */}
      <div className="mt-4 p-2 bg-gray-800/30 rounded text-xs text-gray-500">
        <div className="grid grid-cols-2 gap-1">
          <span>Spectrum Analyzer:</span>
          <span className={usePerformanceStore.getState().settings.enableSpectrumAnalyzer ? 'text-green-400' : 'text-red-400'}>
            {usePerformanceStore.getState().settings.enableSpectrumAnalyzer ? 'Activo' : 'Inactivo'}
          </span>
          <span>Efectos de blur:</span>
          <span className={usePerformanceStore.getState().settings.enableBlurEffects ? 'text-green-400' : 'text-red-400'}>
            {usePerformanceStore.getState().settings.enableBlurEffects ? 'Activo' : 'Inactivo'}
          </span>
          <span>Efectos de brillo:</span>
          <span className={usePerformanceStore.getState().settings.enableGlowEffects ? 'text-green-400' : 'text-red-400'}>
            {usePerformanceStore.getState().settings.enableGlowEffects ? 'Activo' : 'Inactivo'}
          </span>
          <span>FPS objetivo:</span>
          <span className="text-cyan-400">{usePerformanceStore.getState().settings.spectrumFPS}</span>
        </div>
      </div>
    </div>
  );
};

export default PerformanceSettings;
