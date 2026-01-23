import React, { useState } from 'react';
import { SlidersHorizontal, Settings2 } from 'lucide-react';
import { Modal } from '../common/Modal';
import { Equalizer } from './Equalizer';
import { PerformanceSettings } from './PerformanceSettings';

interface EqualizerModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type TabType = 'equalizer' | 'performance';

export const EqualizerModal: React.FC<EqualizerModalProps> = ({
  isOpen,
  onClose,
}) => {
  const [activeTab, setActiveTab] = useState<TabType>('equalizer');

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title={activeTab === 'equalizer' ? 'Ecualizador' : 'Rendimiento'}
      icon={activeTab === 'equalizer' 
        ? <SlidersHorizontal className="w-5 h-5 text-cyan-400" />
        : <Settings2 className="w-5 h-5 text-cyan-400" />
      }
      maxWidth="xl"
    >
      {/* Tabs */}
      <div className="flex border-b border-gray-800 px-4">
        <button
          type="button"
          onClick={() => setActiveTab('equalizer')}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-all border-b-2 -mb-px ${
            activeTab === 'equalizer'
              ? 'border-cyan-500 text-cyan-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          <SlidersHorizontal className="w-4 h-4" />
          Ecualizador
        </button>
        <button
          type="button"
          onClick={() => setActiveTab('performance')}
          className={`flex items-center gap-2 px-4 py-2 text-sm font-medium transition-all border-b-2 -mb-px ${
            activeTab === 'performance'
              ? 'border-cyan-500 text-cyan-400'
              : 'border-transparent text-gray-400 hover:text-white'
          }`}
        >
          <Settings2 className="w-4 h-4" />
          Rendimiento
        </button>
      </div>

      {/* Content */}
      <div className="p-4">
        {activeTab === 'equalizer' && <Equalizer />}
        {activeTab === 'performance' && <PerformanceSettings />}
      </div>
    </Modal>
  );
};

export default EqualizerModal;
