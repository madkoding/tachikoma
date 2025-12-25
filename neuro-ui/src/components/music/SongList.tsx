import React, { useState } from 'react';
import { 
  Play, 
  Pause, 
  MoreVertical, 
  Trash2, 
  Edit2, 
  Music,
  GripVertical,
  Clock,
  ExternalLink
} from 'lucide-react';
import { useMusicStore, formatDuration } from '../../stores/musicStore';
import { SongDto, PlaylistWithSongsDto } from '../../api/client';

interface SongListProps {
  playlist: PlaylistWithSongsDto;
  onEditSong?: (song: SongDto) => void;
}

export const SongList: React.FC<SongListProps> = ({ playlist, onEditSong }) => {
  const { 
    player, 
    playSong, 
    togglePlay, 
    deleteSong,
    reorderSongs 
  } = useMusicStore();
  
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [draggedSong, setDraggedSong] = useState<string | null>(null);
  const [dragOverSong, setDragOverSong] = useState<string | null>(null);

  const handlePlay = (song: SongDto) => {
    if (player.currentSong?.id === song.id) {
      togglePlay();
    } else {
      playSong(song, playlist);
    }
  };

  const handleDelete = async (songId: string, e: React.MouseEvent) => {
    e.stopPropagation();
    if (confirmDelete === songId) {
      await deleteSong(playlist.id, songId);
      setConfirmDelete(null);
      setMenuOpen(null);
    } else {
      setConfirmDelete(songId);
      setTimeout(() => setConfirmDelete(null), 3000);
    }
  };

  // Drag and drop handlers
  const handleDragStart = (e: React.DragEvent, songId: string) => {
    setDraggedSong(songId);
    e.dataTransfer.effectAllowed = 'move';
  };

  const handleDragOver = (e: React.DragEvent, songId: string) => {
    e.preventDefault();
    if (songId !== draggedSong) {
      setDragOverSong(songId);
    }
  };

  const handleDragEnd = async () => {
    if (draggedSong && dragOverSong && draggedSong !== dragOverSong) {
      const songs = [...playlist.songs];
      const draggedIndex = songs.findIndex(s => s.id === draggedSong);
      const dropIndex = songs.findIndex(s => s.id === dragOverSong);
      
      const [removed] = songs.splice(draggedIndex, 1);
      songs.splice(dropIndex, 0, removed);
      
      const newOrder = songs.map(s => s.id);
      await reorderSongs(playlist.id, newOrder);
    }
    
    setDraggedSong(null);
    setDragOverSong(null);
  };

  if (playlist.songs.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center h-48 text-gray-500">
        <Music className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">No hay canciones en esta playlist</p>
        <p className="text-xs mt-1">Busca en YouTube para agregar canciones</p>
      </div>
    );
  }

  return (
    <div className="divide-y divide-gray-800/50">
      {playlist.songs.map((song, index) => {
        const isPlaying = player.currentSong?.id === song.id && player.isPlaying;
        const isCurrentSong = player.currentSong?.id === song.id;
        const isDragging = draggedSong === song.id;
        const isDragOver = dragOverSong === song.id;

        return (
          <div
            key={song.id}
            draggable
            onDragStart={(e) => handleDragStart(e, song.id)}
            onDragOver={(e) => handleDragOver(e, song.id)}
            onDragEnd={handleDragEnd}
            onClick={() => handlePlay(song)}
            className={`
              group flex items-center gap-2 sm:gap-3 p-2 sm:p-3 cursor-pointer transition-all
              ${isCurrentSong 
                ? 'bg-cyan-500/10' 
                : 'hover:bg-gray-800/50'
              }
              ${isDragging ? 'opacity-50' : ''}
              ${isDragOver ? 'border-t-2 border-cyan-500' : ''}
            `}
          >
            {/* Drag handle & Number - hidden on mobile */}
            <div className="hidden sm:flex w-8 items-center justify-center">
              <span className="group-hover:hidden text-gray-500 text-sm font-mono">
                {isCurrentSong && isPlaying ? (
                  <div className="flex items-end gap-0.5 h-4">
                    {[1, 2, 3].map(i => (
                      <div
                        key={i}
                        className="w-1 bg-cyan-400 rounded-full animate-pulse"
                        style={{
                          height: `${Math.random() * 100}%`,
                          animationDelay: `${i * 0.15}s`,
                        }}
                      />
                    ))}
                  </div>
                ) : (
                  index + 1
                )}
              </span>
              <GripVertical className="hidden group-hover:block w-4 h-4 text-gray-500 cursor-grab" />
            </div>

            {/* Cover */}
            <div className="w-10 h-10 sm:w-12 sm:h-12 bg-gray-800 overflow-hidden flex-shrink-0 relative">
              {song.cover_url || song.thumbnail_url ? (
                <img
                  src={song.cover_url || song.thumbnail_url}
                  alt={song.title}
                  className="w-full h-full object-cover"
                />
              ) : (
                <div className="w-full h-full flex items-center justify-center">
                  <Music className="w-4 h-4 sm:w-5 sm:h-5 text-gray-600" />
                </div>
              )}
              
              {/* Play/Pause overlay */}
              <div className="absolute inset-0 bg-black/60 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center">
                {isPlaying ? (
                  <Pause className="w-4 h-4 sm:w-5 sm:h-5 text-white" />
                ) : (
                  <Play className="w-4 h-4 sm:w-5 sm:h-5 text-white" />
                )}
              </div>
            </div>

            {/* Title & Artist */}
            <div className="flex-1 min-w-0">
              <div className={`font-medium truncate overflow-hidden text-sm sm:text-base ${isCurrentSong ? 'text-cyan-400' : 'text-white'}`}>
                <span className={isCurrentSong ? 'inline-block animate-marquee' : ''}>{song.title}</span>
              </div>
              <div className="text-xs sm:text-sm text-gray-400 truncate">
                {song.artist || 'Artista desconocido'}
              </div>
            </div>

            {/* Duration */}
            <div className="flex items-center gap-1 text-xs led-time flex-shrink-0">
              <Clock className="w-3 h-3 hidden sm:block" />
              {formatDuration(song.duration)}
            </div>

            {/* Play count - hidden on mobile */}
            {song.play_count > 0 && (
              <div className="hidden sm:block text-xs text-gray-600">
                {song.play_count}×
              </div>
            )}

            {/* Menu */}
            <button
              onClick={(e) => {
                e.stopPropagation();
                setMenuOpen(menuOpen === song.id ? null : song.id);
              }}
              className="p-1 sm:opacity-0 group-hover:opacity-100 transition-opacity text-gray-400 hover:text-white"
            >
              <MoreVertical className="w-4 h-4" />
            </button>

            {/* Dropdown menu */}
            {menuOpen === song.id && (
              <>
                <div 
                  className="fixed inset-0 z-10"
                  onClick={() => setMenuOpen(null)}
                />
                <div className="absolute right-10 top-1/2 -translate-y-1/2 z-20 bg-gray-800 border border-gray-700 shadow-xl overflow-hidden min-w-[160px]">
                  <a
                    href={song.youtube_url}
                    target="_blank"
                    rel="noopener noreferrer"
                    onClick={(e) => e.stopPropagation()}
                    className="w-full px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2"
                  >
                    <ExternalLink className="w-4 h-4" />
                    Ver en YouTube
                  </a>
                  {onEditSong && (
                    <button
                      onClick={(e) => {
                        e.stopPropagation();
                        onEditSong(song);
                        setMenuOpen(null);
                      }}
                      className="w-full px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2"
                    >
                      <Edit2 className="w-4 h-4" />
                      Editar
                    </button>
                  )}
                  <button
                    onClick={(e) => handleDelete(song.id, e)}
                    className={`w-full px-3 py-2 text-left text-sm flex items-center gap-2 ${
                      confirmDelete === song.id
                        ? 'bg-red-500/20 text-red-400'
                        : 'text-gray-300 hover:bg-gray-700'
                    }`}
                  >
                    <Trash2 className="w-4 h-4" />
                    {confirmDelete === song.id ? 'Confirmar' : 'Eliminar'}
                  </button>
                </div>
              </>
            )}
          </div>
        );
      })}
    </div>
  );
};

export default SongList;
