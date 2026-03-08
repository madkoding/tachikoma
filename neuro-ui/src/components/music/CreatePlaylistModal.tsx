import React, { useState } from 'react';
import { Music } from 'lucide-react';
import { useMusicStore } from '../../stores/musicStore';
import { CreatePlaylistRequest } from '../../api/client';
import { Modal } from '../common/Modal';

interface CreatePlaylistModalProps {
  isOpen: boolean;
  onClose: () => void;
  onCreated?: (playlistId: string) => void;
}

export const CreatePlaylistModal: React.FC<CreatePlaylistModalProps> = ({
  isOpen,
  onClose,
  onCreated,
}) => {
  const { createPlaylist } = useMusicStore();
  const [name, setName] = useState('');
  const [description, setDescription] = useState('');
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!name.trim()) return;

    setIsSubmitting(true);
    try {
      const request: CreatePlaylistRequest = {
        name: name.trim(),
        description: description.trim() || undefined,
      };
      
      const playlist = await createPlaylist(request);
      onCreated?.(playlist.id);
      onClose();
      setName('');
      setDescription('');
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <Modal
      isOpen={isOpen}
      onClose={onClose}
      title="Nueva Playlist"
      icon={<Music className="w-5 h-5 text-cyan-400" />}
      maxWidth="md"
    >
      <form onSubmit={handleSubmit} className="p-4 space-y-4">
        {/* Name */}
        <div>
          <label className="block text-sm text-gray-400 mb-1">
            Nombre *
          </label>
          <input
            type="text"
            value={name}
            onChange={(e) => setName(e.target.value)}
            placeholder="Mi playlist"
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 text-white placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition-all"
            autoFocus
          />
        </div>

        {/* Description */}
        <div>
          <label className="block text-sm text-gray-400 mb-1">
            Descripción (opcional)
          </label>
          <textarea
            value={description}
            onChange={(e) => setDescription(e.target.value)}
            placeholder="Una descripción para tu playlist..."
            rows={3}
            className="w-full px-3 py-2 bg-gray-800 border border-gray-700 text-white placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition-all resize-none"
          />
        </div>

        {/* Actions */}
        <div className="flex justify-end gap-3 pt-2">
          <button
            type="button"
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cancelar
          </button>
          <button
            type="submit"
            disabled={!name.trim() || isSubmitting}
            className="px-4 py-2 bg-cyan-500 text-black font-medium hover:bg-cyan-400 disabled:opacity-50 disabled:cursor-not-allowed transition-all"
          >
            {isSubmitting ? 'Creando...' : 'Crear Playlist'}
          </button>
        </div>
      </form>
    </Modal>
  );
};

export default CreatePlaylistModal;
