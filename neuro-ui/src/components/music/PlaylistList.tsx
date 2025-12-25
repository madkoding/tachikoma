import React, { useState } from 'react';
import { 
  Plus, 
  MoreVertical, 
  Play, 
  Trash2, 
  Edit2, 
  Music,
  Clock,
  ListMusic,
  Shuffle,
  Loader2
} from 'lucide-react';
import { useMusicStore, formatDurationLong } from '../../stores/musicStore';
import { PlaylistDto } from '../../api/client';

interface PlaylistListProps {
  onSelectPlaylist: (playlist: PlaylistDto) => void;
  selectedPlaylistId?: string;
  onCreatePlaylist: () => void;
}

export const PlaylistList: React.FC<PlaylistListProps> = ({
  onSelectPlaylist,
  selectedPlaylistId,
  onCreatePlaylist,
}) => {
  const { playlists, isLoadingPlaylists, deletePlaylist, pollingPlaylistId } = useMusicStore();
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);

  const handleDelete = async (id: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirmDelete === id) {
      await deletePlaylist(id);
      setConfirmDelete(null);
      setMenuOpen(null);
    } else {
      setConfirmDelete(id);
      setTimeout(() => setConfirmDelete(null), 3000);
    }
  };

  if (isLoadingPlaylists) {
    return (
      <div className="p-4 space-y-3">
        {[1, 2, 3].map(i => (
          <div key={i} className="h-20 bg-gray-800/50 animate-pulse" />
        ))}
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col">
      {/* Header */}
      <div className="p-4 border-b border-cyan-500/20">
        <div className="flex items-center justify-between">
          <h2 className="text-lg font-bold text-cyan-400 flex items-center gap-2 font-cyber">
            <ListMusic className="w-5 h-5" />
            Playlists
          </h2>
          <button
            onClick={onCreatePlaylist}
            className="p-2 bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 transition-all group"
          >
            <Plus className="w-5 h-5 group-hover:rotate-90 transition-transform" />
          </button>
        </div>
      </div>

      {/* Playlist list */}
      <div className="flex-1 overflow-y-auto p-2 space-y-2">
        {playlists.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-48 text-gray-500">
            <Music className="w-12 h-12 mb-3 opacity-30" />
            <p className="text-sm">No hay playlists</p>
            <button
              onClick={onCreatePlaylist}
              className="mt-3 text-cyan-400 text-sm hover:underline"
            >
              Crear una playlist
            </button>
          </div>
        ) : (
          playlists.map((playlist) => (
            <div
              key={playlist.id}
              onClick={() => onSelectPlaylist(playlist)}
              className={`
                relative group p-3 cursor-pointer transition-all
                ${selectedPlaylistId === playlist.id
                  ? 'bg-cyan-500/20 border border-cyan-500/50'
                  : 'bg-gray-800/50 border border-transparent hover:bg-gray-800 hover:border-gray-700'
                }
              `}
            >
              {/* Cover */}
              <div className="flex gap-3">
                <div className="w-14 h-14 bg-gray-700 flex-shrink-0 overflow-hidden relative">
                  {playlist.cover_url ? (
                    <img
                      src={playlist.cover_url}
                      alt={playlist.name}
                      className="w-full h-full object-cover"
                    />
                  ) : pollingPlaylistId === playlist.id ? (
                    <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyan-500/20 to-purple-500/20">
                      <Loader2 className="w-6 h-6 text-cyan-400 animate-spin" />
                    </div>
                  ) : (
                    <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyan-500/20 to-purple-500/20">
                      <Music className="w-6 h-6 text-gray-500" />
                    </div>
                  )}
                  
                  {/* Play overlay on hover */}
                  <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                    <Play className="w-6 h-6 text-white" />
                  </div>
                </div>

                {/* Info */}
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-white truncate">
                    {playlist.name}
                  </div>
                  <div className="flex items-center gap-2 text-xs text-gray-400 mt-1">
                    <span className="flex items-center gap-1">
                      <Music className="w-3 h-3" />
                      {playlist.song_count}
                    </span>
                    <span className="flex items-center gap-1">
                      <Clock className="w-3 h-3" />
                      {formatDurationLong(playlist.total_duration)}
                    </span>
                    {playlist.shuffle && (
                      <Shuffle className="w-3 h-3 text-cyan-400" />
                    )}
                  </div>
                  {playlist.description && (
                    <div className="text-xs text-gray-500 mt-1 truncate">
                      {playlist.description}
                    </div>
                  )}
                </div>

                {/* Menu button */}
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    setMenuOpen(menuOpen === playlist.id ? null : playlist.id);
                  }}
                  className="p-1 opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white"
                >
                  <MoreVertical className="w-4 h-4" />
                </button>
              </div>

              {/* Dropdown menu */}
              {menuOpen === playlist.id && (
                <>
                  <div 
                    className="fixed inset-0 z-10"
                    onClick={() => setMenuOpen(null)}
                  />
                  <div className="absolute right-2 top-12 z-20 bg-gray-800 border border-gray-700 shadow-xl overflow-hidden min-w-[140px]">
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        // TODO: Open edit modal
                        setMenuOpen(null);
                      }}
                      className="w-full px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2"
                    >
                      <Edit2 className="w-4 h-4" />
                      Editar
                    </button>
                    <button
                      onClick={(e) => handleDelete(playlist.id, e)}
                      className={`w-full px-3 py-2 text-left text-sm flex items-center gap-2 ${
                        confirmDelete === playlist.id
                          ? 'bg-red-500/20 text-red-400'
                          : 'text-gray-300 hover:bg-gray-700'
                      }`}
                    >
                      <Trash2 className="w-4 h-4" />
                      {confirmDelete === playlist.id ? 'Confirmar' : 'Eliminar'}
                    </button>
                  </div>
                </>
              )}

              {/* Suggestion badge */}
              {playlist.is_suggestions && (
                <div className="absolute top-2 right-2 px-2 py-0.5 bg-purple-500/20 text-purple-400 text-[10px] font-medium">
                  Sugerencias IA
                </div>
              )}
            </div>
          ))
        )}
      </div>
    </div>
  );
};

export default PlaylistList;
