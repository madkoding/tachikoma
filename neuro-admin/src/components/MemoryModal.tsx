import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import type { GraphNode } from '../types/graph';
import { NODE_COLORS, MEMORY_TYPES } from '../constants/graph';

interface MemoryModalProps {
  readonly node: GraphNode | null;
  readonly onClose: () => void;
  readonly onSave: (id: string, content: string, memoryType: string, importance: number) => void;
  readonly onDelete: (id: string) => void;
  readonly isLoading: boolean;
}

export default function MemoryModal({ node, onClose, onSave, onDelete, isLoading }: MemoryModalProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState(node?.content || '');
  const [memoryType, setMemoryType] = useState(node?.memory_type || 'fact');
  const [importance, setImportance] = useState(node?.importance_score || 0.5);
  const [isDeleting, setIsDeleting] = useState(false);

  useEffect(() => {
    if (node) {
      setContent(node.content);
      setMemoryType(node.memory_type);
      setImportance(node.importance_score || 0.5);
    }
  }, [node]);

  if (!node) return null;

  const handleSave = () => {
    onSave(node.id, content, memoryType, importance);
  };

  const handleDelete = () => {
    if (isDeleting) {
      onDelete(node.id);
    } else {
      setIsDeleting(true);
      setTimeout(() => setIsDeleting(false), 3000);
    }
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop - click outside to close */}
      <button
        type="button"
        className="absolute inset-0 bg-black/80 backdrop-blur-sm cursor-default border-0"
        onClick={onClose}
        aria-label="Close modal"
      />
      
      {/* Modal */}
      <div className="relative w-full max-w-2xl cyber-card-modal animate-modal-in">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-cyber-cyan/30">
          <div className="flex items-center gap-3">
            <span
              className="w-4 h-4 rounded-full animate-pulse"
              style={{ 
                backgroundColor: NODE_COLORS[node.memory_type] || NODE_COLORS.default,
                boxShadow: `0 0 15px ${NODE_COLORS[node.memory_type] || NODE_COLORS.default}`
              }}
            />
            <h2 className="text-xl font-bold neon-cyan font-cyber tracking-wider">
              {t('graph.nodeDetails')}
            </h2>
          </div>
          <button
            onClick={onClose}
            className="p-2 rounded-lg hover:bg-cyber-cyan/10 text-cyber-cyan transition-all"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>
        
        {/* Content */}
        <div className="p-6 space-y-6">
          {/* Content field */}
          <div>
            <label className="block text-sm text-cyber-cyan/70 font-mono mb-2 uppercase tracking-wider">
              {t('graph.content')}
            </label>
            <textarea
              value={content}
              onChange={(e) => setContent(e.target.value)}
              className="cyber-input w-full h-32 resize-none"
              placeholder={t('memories.contentPlaceholder')}
            />
          </div>
          
          {/* Type selector */}
          <div className="grid grid-cols-2 gap-4">
            <div>
              <label className="block text-sm text-cyber-cyan/70 font-mono mb-2 uppercase tracking-wider">
                {t('memories.type')}
              </label>
              <select
                value={memoryType}
                onChange={(e) => setMemoryType(e.target.value)}
                className="cyber-input w-full"
              >
                {MEMORY_TYPES.map((type) => (
                  <option key={type} value={type}>{t(`graph.types.${type}`, type)}</option>
                ))}
              </select>
            </div>
            
            {/* Importance slider */}
            <div>
              <label className="block text-sm text-cyber-cyan/70 font-mono mb-2 uppercase tracking-wider">
                {t('graph.importance')}: <span className="neon-green">{importance.toFixed(2)}</span>
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={importance}
                onChange={(e) => setImportance(Number.parseFloat(e.target.value))}
                className="cyber-slider w-full"
              />
            </div>
          </div>
          
          {/* Metadata */}
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div className="cyber-card-inner p-3">
              <span className="text-cyber-cyan/50 font-mono text-xs uppercase">ID</span>
              <p className="text-cyber-cyan/70 font-mono text-xs break-all mt-1">{node.id}</p>
            </div>
            <div className="cyber-card-inner p-3">
              <span className="text-cyber-cyan/50 font-mono text-xs uppercase">{t('graph.created')}</span>
              <p className="text-cyber-cyan/70 font-mono text-xs mt-1">
                {new Date(node.created_at).toLocaleString()}
              </p>
            </div>
          </div>
          
          {/* Access count */}
          <div className="cyber-card-inner p-3">
            <span className="text-cyber-cyan/50 font-mono text-xs uppercase">{t('graph.accessCount')}</span>
            <p className="text-cyber-magenta font-mono mt-1">{node.access_count || 0}</p>
          </div>
        </div>
        
        {/* Footer */}
        <div className="flex items-center justify-between p-4 border-t border-cyber-cyan/30">
          <button
            onClick={handleDelete}
            disabled={isLoading}
            className={`cyber-button-danger px-4 py-2 ${isDeleting ? 'animate-pulse' : ''}`}
          >
            {isDeleting ? t('memories.confirmDelete') : t('memories.delete')}
          </button>
          <div className="flex gap-3">
            <button
              onClick={onClose}
              className="cyber-button-secondary px-4 py-2"
            >
              {t('common.cancel')}
            </button>
            <button
              onClick={handleSave}
              disabled={isLoading || !content.trim()}
              className="cyber-button px-6 py-2"
            >
              {isLoading ? t('common.loading') : t('common.save')}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
