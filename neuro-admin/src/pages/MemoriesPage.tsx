import { useState } from 'react';
import { useTranslation } from 'react-i18next';
import { useQuery, useMutation, useQueryClient } from '@tanstack/react-query';
import { memoryApi, type Memory } from '../api/client';

export default function MemoriesPage() {
  const { t } = useTranslation();
  const queryClient = useQueryClient();
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedMemory, setSelectedMemory] = useState<Memory | null>(null);
  const [isModalOpen, setIsModalOpen] = useState(false);

  const { data: memories, isLoading } = useQuery({
    queryKey: ['memories', searchQuery],
    queryFn: () => searchQuery 
      ? memoryApi.search(searchQuery, 50)
      : memoryApi.getAll(100, 0),
  });

  const deleteMutation = useMutation({
    mutationFn: memoryApi.delete,
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['memories'] });
      queryClient.invalidateQueries({ queryKey: ['graph-stats'] });
    },
  });

  const handleDelete = (id: string) => {
    if (window.confirm(t('memories.confirmDelete'))) {
      deleteMutation.mutate(id);
    }
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
            {t('memories.title')}
          </h1>
          <p className="text-gray-600 dark:text-gray-400 mt-1">
            {t('memories.subtitle')}
          </p>
        </div>
        <button
          onClick={() => setIsModalOpen(true)}
          className="px-4 py-2 bg-neuro-500 text-white rounded-lg hover:bg-neuro-600 transition-colors"
        >
          {t('memories.add')}
        </button>
      </div>

      {/* Search */}
      <div className="bg-white dark:bg-gray-800 p-4 rounded-xl shadow-sm">
        <input
          type="text"
          placeholder={t('common.search')}
          value={searchQuery}
          onChange={(e) => setSearchQuery(e.target.value)}
          className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-neuro-500"
        />
      </div>

      {/* Table */}
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-sm overflow-hidden">
        {isLoading ? (
          <div className="flex items-center justify-center h-64">
            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-neuro-500"></div>
          </div>
        ) : memories && memories.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="w-full">
              <thead className="bg-gray-50 dark:bg-gray-700">
                <tr>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('memories.content')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('memories.type')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('memories.importance')}
                  </th>
                  <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    {t('memories.createdAt')}
                  </th>
                  <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 dark:text-gray-400 uppercase tracking-wider">
                    Actions
                  </th>
                </tr>
              </thead>
              <tbody className="divide-y divide-gray-200 dark:divide-gray-700">
                {memories.map((memory) => (
                  <tr
                    key={memory.id}
                    className="hover:bg-gray-50 dark:hover:bg-gray-700 cursor-pointer"
                    onClick={() => setSelectedMemory(memory)}
                  >
                    <td className="px-6 py-4">
                      <p className="text-sm text-gray-900 dark:text-white truncate max-w-md">
                        {memory.content}
                      </p>
                    </td>
                    <td className="px-6 py-4">
                      <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-neuro-100 dark:bg-neuro-900/30 text-neuro-800 dark:text-neuro-300">
                        {memory.memory_type}
                      </span>
                    </td>
                    <td className="px-6 py-4">
                      <div className="flex items-center">
                        <div className="w-16 bg-gray-200 dark:bg-gray-600 rounded-full h-2">
                          <div
                            className="bg-neuro-500 h-2 rounded-full"
                            style={{ width: `${memory.importance * 100}%` }}
                          />
                        </div>
                        <span className="ml-2 text-sm text-gray-600 dark:text-gray-400">
                          {memory.importance.toFixed(2)}
                        </span>
                      </div>
                    </td>
                    <td className="px-6 py-4 text-sm text-gray-600 dark:text-gray-400">
                      {new Date(memory.created_at).toLocaleDateString()}
                    </td>
                    <td className="px-6 py-4 text-right">
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          handleDelete(memory.id);
                        }}
                        className="text-red-500 hover:text-red-700 text-sm"
                      >
                        {t('memories.delete')}
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <div className="flex items-center justify-center h-64 text-gray-500 dark:text-gray-400">
            {t('memories.noMemories')}
          </div>
        )}
      </div>

      {/* Memory Detail Modal */}
      {selectedMemory && (
        <MemoryDetailModal
          memory={selectedMemory}
          onClose={() => setSelectedMemory(null)}
        />
      )}

      {/* Add Memory Modal */}
      {isModalOpen && (
        <AddMemoryModal
          onClose={() => setIsModalOpen(false)}
          onSuccess={() => {
            setIsModalOpen(false);
            queryClient.invalidateQueries({ queryKey: ['memories'] });
          }}
        />
      )}
    </div>
  );
}

interface MemoryDetailModalProps {
  memory: Memory;
  onClose: () => void;
}

function MemoryDetailModal({ memory, onClose }: MemoryDetailModalProps) {
  const { t } = useTranslation();
  const { data: related } = useQuery({
    queryKey: ['memory-related', memory.id],
    queryFn: () => memoryApi.getRelated(memory.id),
  });

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-2xl w-full mx-4 max-h-[80vh] overflow-auto">
        <div className="p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              Memory Details
            </h2>
            <button
              onClick={onClose}
              className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div className="space-y-4">
            <div>
              <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                {t('memories.content')}
              </label>
              <p className="mt-1 text-gray-900 dark:text-white">{memory.content}</p>
            </div>

            <div className="grid grid-cols-2 gap-4">
              <div>
                <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                  {t('memories.type')}
                </label>
                <p className="mt-1 text-gray-900 dark:text-white">{memory.memory_type}</p>
              </div>
              <div>
                <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                  {t('memories.importance')}
                </label>
                <p className="mt-1 text-gray-900 dark:text-white">{memory.importance.toFixed(2)}</p>
              </div>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                {t('memories.createdAt')}
              </label>
              <p className="mt-1 text-gray-900 dark:text-white">
                {new Date(memory.created_at).toLocaleString()}
              </p>
            </div>

            <div>
              <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                ID
              </label>
              <p className="mt-1 text-xs text-gray-500 dark:text-gray-400 font-mono break-all">
                {memory.id}
              </p>
            </div>

            {related && related.length > 0 && (
              <div>
                <label className="text-sm font-medium text-gray-500 dark:text-gray-400">
                  {t('memories.relations')} ({related.length})
                </label>
                <div className="mt-2 space-y-2">
                  {related.map((r) => (
                    <div
                      key={r.id}
                      className="p-2 bg-gray-50 dark:bg-gray-700 rounded text-sm"
                    >
                      <span className="text-neuro-500">{r.memory_type}</span>
                      <span className="text-gray-600 dark:text-gray-400"> - </span>
                      <span className="text-gray-900 dark:text-white">{r.content.substring(0, 100)}...</span>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

interface AddMemoryModalProps {
  onClose: () => void;
  onSuccess: () => void;
}

function AddMemoryModal({ onClose, onSuccess }: AddMemoryModalProps) {
  const { t } = useTranslation();
  const [content, setContent] = useState('');
  const [memoryType, setMemoryType] = useState('fact');
  const [importance, setImportance] = useState(0.5);

  const createMutation = useMutation({
    mutationFn: memoryApi.create,
    onSuccess: () => {
      onSuccess();
    },
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    createMutation.mutate({
      content,
      memory_type: memoryType,
      importance,
      metadata: {},
    });
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-xl max-w-lg w-full mx-4">
        <form onSubmit={handleSubmit} className="p-6">
          <div className="flex items-center justify-between mb-4">
            <h2 className="text-xl font-semibold text-gray-900 dark:text-white">
              {t('memories.add')}
            </h2>
            <button
              type="button"
              onClick={onClose}
              className="p-1 rounded hover:bg-gray-100 dark:hover:bg-gray-700"
            >
              <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          <div className="space-y-4">
            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                {t('memories.content')}
              </label>
              <textarea
                value={content}
                onChange={(e) => setContent(e.target.value)}
                rows={4}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-neuro-500"
                required
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                {t('memories.type')}
              </label>
              <select
                value={memoryType}
                onChange={(e) => setMemoryType(e.target.value)}
                className="w-full px-4 py-2 border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-700 text-gray-900 dark:text-white focus:outline-none focus:ring-2 focus:ring-neuro-500"
              >
                <option value="fact">fact</option>
                <option value="preference">preference</option>
                <option value="context">context</option>
                <option value="conversation">conversation</option>
                <option value="task">task</option>
                <option value="entity">entity</option>
              </select>
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">
                {t('memories.importance')}: {importance.toFixed(2)}
              </label>
              <input
                type="range"
                min="0"
                max="1"
                step="0.01"
                value={importance}
                onChange={(e) => setImportance(parseFloat(e.target.value))}
                className="w-full"
              />
            </div>
          </div>

          <div className="flex justify-end gap-3 mt-6">
            <button
              type="button"
              onClick={onClose}
              className="px-4 py-2 text-gray-700 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg"
            >
              {t('common.cancel')}
            </button>
            <button
              type="submit"
              disabled={createMutation.isPending}
              className="px-4 py-2 bg-neuro-500 text-white rounded-lg hover:bg-neuro-600 disabled:opacity-50"
            >
              {createMutation.isPending ? t('common.loading') : t('common.save')}
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
