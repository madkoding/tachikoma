import React, { useState } from 'react';
import { Volume2, Music2, Zap, Waves, Mic } from 'lucide-react';
import { useMusicStore, EQUALIZER_FREQUENCIES, EQUALIZER_PRESETS } from '../../stores/musicStore';

interface EqualizerProps {
  className?: string;
}

export const Equalizer: React.FC<EqualizerProps> = ({ className = '' }) => {
  const { 
    equalizer, 
    audioFilters,
    setEqualizerBand, 
    loadEqualizerPreset, 
    updateEqualizer,
    toggleHighpass,
    toggleLowpass,
    toggleLoudness,
    toggleBassBoost,
    toggleStereoWide,
    toggleVocalEnhancer,
  } = useMusicStore();
  const [isDragging, setIsDragging] = useState<number | null>(null);

  const handleBandChange = (band: number, value: number) => {
    setEqualizerBand(band, value);
  };

  const handleMouseDown = (band: number) => {
    setIsDragging(band);
  };

  const handleMouseUp = async () => {
    if (isDragging !== null) {
      setIsDragging(null);
      // Save to backend
      await updateEqualizer(equalizer);
    }
  };

  const handlePresetClick = async (preset: string) => {
    // loadEqualizerPreset updates the store, then we get the updated state
    await loadEqualizerPreset(preset);
    // Get the latest equalizer state from the store after the preset is loaded
    const updatedEqualizer = useMusicStore.getState().equalizer;
    await updateEqualizer(updatedEqualizer);
  };

  // Get color based on gain value
  const getSliderColor = (value: number) => {
    if (value > 0) {
      const intensity = Math.abs(value) / 12;
      return `rgba(0, 255, 255, ${0.5 + intensity * 0.5})`;
    } else if (value < 0) {
      const intensity = Math.abs(value) / 12;
      return `rgba(255, 0, 128, ${0.5 + intensity * 0.5})`;
    }
    return 'rgba(100, 100, 100, 0.5)';
  };

  const getGlowColor = (value: number) => {
    if (value > 0) {
      return 'rgba(0, 255, 255, 0.4)';
    } else if (value < 0) {
      return 'rgba(255, 0, 128, 0.4)';
    }
    return 'transparent';
  };

  return (
    <div className={`bg-gray-900/80 border border-cyan-500/30 p-4 ${className}`}>
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-cyan-400 font-bold text-sm tracking-wider uppercase font-cyber">
          Ecualizador 8 Bandas
        </h3>
        
        {/* Enable toggle */}
        <button
          type="button"
          onClick={() => updateEqualizer({ ...equalizer, enabled: !equalizer.enabled })}
          className={`px-3 py-1 text-xs font-medium transition-all font-mono ${
            equalizer.enabled 
              ? 'bg-cyan-500 text-black' 
              : 'bg-gray-700 text-gray-400'
          }`}
        >
          {equalizer.enabled ? 'ON' : 'OFF'}
        </button>
      </div>

      {/* Audio Filters Section */}
      <div className="mb-4 p-2 bg-gray-800/50 border border-gray-700">
        {/* Filter buttons grid */}
        <div className="grid grid-cols-6 gap-2">
          {/* Highpass Filter */}
          <button
            type="button"
            onClick={toggleHighpass}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.highpassEnabled
                ? 'bg-cyber-cyan text-black shadow-md shadow-cyber-cyan/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title={`Highpass Filter: ${audioFilters.highpassFreq}Hz - Elimina sub-graves`}
          >
            <Volume2 className="w-4 h-4 mb-0.5" />
            <span>HPF</span>
          </button>

          {/* Lowpass Filter */}
          <button
            type="button"
            onClick={toggleLowpass}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.lowpassEnabled
                ? 'bg-cyber-purple text-white shadow-md shadow-cyber-purple/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title={`Lowpass Filter: ${audioFilters.lowpassFreq >= 1000 ? `${(audioFilters.lowpassFreq / 1000).toFixed(0)}kHz` : `${audioFilters.lowpassFreq}Hz`} - Elimina agudos`}
          >
            <Music2 className="w-4 h-4 mb-0.5" />
            <span>LPF</span>
          </button>

          {/* Loudness */}
          <button
            type="button"
            onClick={toggleLoudness}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.loudnessEnabled
                ? 'bg-orange-500 text-black shadow-md shadow-orange-500/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title="Loudness: Compensa pérdida de graves/agudos a bajo volumen"
          >
            <Volume2 className="w-4 h-4 mb-0.5" />
            <span>LOUD</span>
          </button>

          {/* Bass Boost */}
          <button
            type="button"
            onClick={toggleBassBoost}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.bassBoostEnabled
                ? 'bg-red-500 text-white shadow-md shadow-red-500/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title="Bass Boost: Potencia los sub-graves (60Hz)"
          >
            <Zap className="w-4 h-4 mb-0.5" />
            <span>BASS</span>
          </button>

          {/* Stereo Wide */}
          <button
            type="button"
            onClick={toggleStereoWide}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.stereoWideEnabled
                ? 'bg-blue-500 text-white shadow-md shadow-blue-500/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title="Stereo Wide: Amplía la imagen estéreo"
          >
            <Waves className="w-4 h-4 mb-0.5" />
            <span>3D</span>
          </button>

          {/* Vocal Enhancer */}
          <button
            type="button"
            onClick={toggleVocalEnhancer}
            className={`aspect-square flex flex-col items-center justify-center p-1.5 text-[9px] font-medium transition-all font-mono rounded ${
              audioFilters.vocalEnhancerEnabled
                ? 'bg-green-500 text-black shadow-md shadow-green-500/30'
                : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            title="Vocal Enhancer: Realza frecuencias de voz"
          >
            <Mic className="w-4 h-4 mb-0.5" />
            <span>VOCAL</span>
          </button>
        </div>
      </div>

      {/* Presets */}
      <div className="flex flex-wrap gap-2 mb-4">
        {EQUALIZER_PRESETS.map((preset) => (
          <button
            key={preset.name}
            onClick={() => handlePresetClick(preset.name)}
            className={`px-3 py-1 text-xs font-medium transition-all ${
              equalizer.preset === preset.name
                ? 'bg-purple-500 text-white'
                : 'bg-gray-800 text-gray-400 hover:bg-gray-700 hover:text-white'
            }`}
          >
            {preset.label}
          </button>
        ))}
      </div>

      {/* Sliders */}
      <div 
        className="flex items-end gap-2"
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
      >
        {equalizer.bands.slice(0, 8).map((value, index) => (
          <div key={index} className="flex-1 flex flex-col items-center">
            {/* dB value */}
            <span className="text-[10px] text-gray-500 mb-1 h-4">
              {value > 0 ? '+' : ''}{value.toFixed(0)}
            </span>
            
            {/* Slider track */}
            <div 
              className="relative w-full h-32 flex flex-col items-center cursor-pointer"
              onMouseDown={() => handleMouseDown(index)}
              onMouseMove={(e) => {
                if (isDragging === index) {
                  const rect = e.currentTarget.getBoundingClientRect();
                  const y = e.clientY - rect.top;
                  const percent = 1 - (y / rect.height);
                  const newValue = (percent * 24) - 12; // -12 to +12
                  handleBandChange(index, newValue);
                }
              }}
            >
              {/* Track background */}
              <div className="absolute w-2 h-full bg-gray-800 rounded-[5px]">
                {/* Center line */}
                <div className="absolute top-1/2 left-0 right-0 h-px bg-gray-600" />
                
                {/* Fill from center */}
                <div
                  className="absolute left-0 right-0 transition-all rounded-[5px]"
                  style={{
                    background: getSliderColor(value),
                    boxShadow: `0 0 10px ${getGlowColor(value)}`,
                    ...(value >= 0 
                      ? { bottom: '50%', height: `${(value / 12) * 50}%` }
                      : { top: '50%', height: `${(Math.abs(value) / 12) * 50}%` }
                    ),
                  }}
                />
              </div>
              
              {/* Knob */}
              <div
                className="absolute w-4 h-4 bg-white shadow-lg transition-all z-10 rounded-[5px]"
                style={{
                  top: `${((12 - value) / 24) * 100}%`,
                  transform: 'translateY(-50%)',
                  boxShadow: `0 0 10px ${getSliderColor(value)}, 0 2px 4px rgba(0,0,0,0.5)`,
                }}
              />
              
              {/* Scale markers */}
              <div className="absolute left-1/2 transform -translate-x-1/2 w-8 h-full flex flex-col justify-between pointer-events-none">
                {['+12', '+6', '0', '-6', '-12'].map((label) => (
                  <div key={label} className="flex items-center justify-end w-full">
                    <div className="w-1 h-px bg-gray-700" />
                  </div>
                ))}
              </div>
            </div>
            
            {/* Frequency label */}
            <span className="text-[10px] text-cyan-400/70 mt-2 font-mono">
              {EQUALIZER_FREQUENCIES[index]}
            </span>
          </div>
        ))}
      </div>

      {/* Hz labels on edges */}
      <div className="flex justify-between mt-2 text-[10px] text-gray-500">
        <span>Hz</span>
        <span>Frecuencia</span>
        <span>Hz</span>
      </div>

      {/* Grid overlay */}
      <div 
        className="absolute inset-0 pointer-events-none opacity-5"
        style={{
          backgroundImage: `
            linear-gradient(to right, cyan 1px, transparent 1px),
            linear-gradient(to bottom, cyan 1px, transparent 1px)
          `,
          backgroundSize: '20px 20px',
        }}
      />
    </div>
  );
};

export default Equalizer;
