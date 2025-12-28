import React, { useState, useMemo } from 'react';
import { 
  Play, 
  Pause, 
  MoreVertical, 
  Trash2, 
  Edit2, 
  Music,
  GripVertical,
  Clock,
  ExternalLink,
  Sparkles,
  ArrowUpDown,
  TrendingUp,
  SortAsc,
  Timer,
  Disc3,
  User,
  Heart,
  ImageIcon
} from 'lucide-react';
import { useMusicStore, usePlayerState, formatDuration } from '../../stores/musicStore';
import { SongDto, PlaylistWithSongsDto } from '../../api/client';
import { AnimatedLedDigits } from '../common';

type SortOption = 'most_played' | 'alphabetical' | 'duration' | 'album' | 'artist';

const SORT_OPTIONS: { value: SortOption; label: string; icon: React.ReactNode }[] = [
  { value: 'most_played', label: 'Más escuchadas', icon: <TrendingUp className="w-4 h-4" /> },
  { value: 'alphabetical', label: 'Alfabético', icon: <SortAsc className="w-4 h-4" /> },
  { value: 'duration', label: 'Duración', icon: <Timer className="w-4 h-4" /> },
  { value: 'album', label: 'Álbum', icon: <Disc3 className="w-4 h-4" /> },
  { value: 'artist', label: 'Artista', icon: <User className="w-4 h-4" /> },
];

interface SongListProps {
  playlist: PlaylistWithSongsDto;
  onEditSong?: (song: SongDto) => void;
}

export const SongList: React.FC<SongListProps> = ({ playlist, onEditSong }) => {
  // Use optimized selector for player state
  const player = usePlayerState();
  
  // Get other actions from store directly (these don't change, so won't cause re-renders)
  const { 
    playSong, 
    togglePlay, 
    deleteSong,
    reorderSongs,
    newSongIds,
    markSongAsSeen,
    toggleSongLike,
    fetchSongCover
  } = useMusicStore();
  
  const [menuOpen, setMenuOpen] = useState<string | null>(null);
  const [confirmDelete, setConfirmDelete] = useState<string | null>(null);
  const [draggedSong, setDraggedSong] = useState<string | null>(null);
  const [dragOverSong, setDragOverSong] = useState<string | null>(null);
  const [sortBy, setSortBy] = useState<SortOption>('most_played');
  const [showSortMenu, setShowSortMenu] = useState(false);
  const [likingIds, setLikingIds] = useState<Set<string>>(new Set());
  const [fetchingCoverIds, setFetchingCoverIds] = useState<Set<string>>(new Set());

  // Sort songs based on selected option
  const sortedSongs = useMemo(() => {
    const songs = [...playlist.songs];
    
    switch (sortBy) {
      case 'most_played':
        return songs.sort((a, b) => b.play_count - a.play_count);
      case 'alphabetical':
        return songs.sort((a, b) => a.title.localeCompare(b.title));
      case 'duration':
        return songs.sort((a, b) => a.duration - b.duration);
      case 'album':
        return songs.sort((a, b) => (a.album || '').localeCompare(b.album || ''));
      case 'artist':
        return songs.sort((a, b) => (a.artist || '').localeCompare(b.artist || ''));
      default:
        return songs;
    }
  }, [playlist.songs, sortBy]);

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

  const currentSortOption = SORT_OPTIONS.find(opt => opt.value === sortBy);

  return (
    <div>
      {/* Sort selector */}
      <div className="flex items-center justify-end px-2 sm:px-3 py-2 border-b border-gray-800/50">
        <div className="relative">
          <button
            onClick={() => setShowSortMenu(!showSortMenu)}
            className="flex items-center gap-2 px-3 py-1.5 text-xs text-gray-400 hover:text-white bg-gray-800/50 hover:bg-gray-800 rounded-md transition-all"
          >
            <ArrowUpDown className="w-3.5 h-3.5" />
            <span className="hidden sm:inline">{currentSortOption?.label}</span>
            {currentSortOption?.icon}
          </button>
          
          {showSortMenu && (
            <>
              <div 
                className="fixed inset-0 z-40" 
                onClick={() => setShowSortMenu(false)}
              />
              <div className="absolute right-0 top-full mt-1 w-48 bg-gray-900 border border-gray-700 rounded-lg shadow-xl z-50 overflow-hidden">
                {SORT_OPTIONS.map((option) => (
                  <button
                    key={option.value}
                    onClick={() => {
                      setSortBy(option.value);
                      setShowSortMenu(false);
                    }}
                    className={`
                      w-full flex items-center gap-3 px-3 py-2 text-sm transition-all
                      ${sortBy === option.value 
                        ? 'bg-cyan-500/20 text-cyan-400' 
                        : 'text-gray-300 hover:bg-gray-800 hover:text-white'
                      }
                    `}
                  >
                    {option.icon}
                    {option.label}
                  </button>
                ))}
              </div>
            </>
          )}
        </div>
      </div>
      
      {/* Song list */}
      <div className="divide-y divide-gray-800/50">
        {sortedSongs.map((song, index) => {
          const isPlaying = player.currentSong?.id === song.id && player.isPlaying;
          const isCurrentSong = player.currentSong?.id === song.id;
          const isDragging = draggedSong === song.id;
          const isDragOver = dragOverSong === song.id;
          const isNew = newSongIds.has(song.id);

          return (
            <div
              key={song.id}
              draggable
              onDragStart={(e) => handleDragStart(e, song.id)}
              onDragOver={(e) => handleDragOver(e, song.id)}
              onDragEnd={handleDragEnd}
              onClick={() => handlePlay(song)}
              onAnimationEnd={() => isNew && markSongAsSeen(song.id)}
              className={`
                group flex items-center gap-2 sm:gap-3 p-2 sm:p-3 cursor-pointer transition-all
                ${isCurrentSong 
                  ? 'bg-cyan-500/10' 
                  : 'hover:bg-gray-800/50'
                }
                ${isDragging ? 'opacity-50' : ''}
                ${isDragOver ? 'border-t-2 border-cyan-500' : ''}
                ${isNew ? 'animate-slide-in-glow' : ''}
              `}
            >
            {/* New song indicator */}
            {isNew && (
              <div className="absolute left-0 top-0 bottom-0 w-1 bg-gradient-to-b from-cyan-400 via-purple-500 to-pink-500 animate-pulse" />
            )}
            
            {/* Drag handle & Number - hidden on mobile */}
            <div className="hidden sm:flex w-8 items-center justify-center">
              <span className="group-hover:hidden text-gray-500 text-sm font-mono">
                {isCurrentSong && isPlaying ? (
                  <div className="flex items-end gap-0.5 h-4">
                    {[1, 2, 3].map(i => (
                      <div
                        key={i}
                        className="w-1 bg-cyan-400 rounded-full animate-equalizer-bar"
                        style={{
                          animationDelay: `${i * 0.15}s`,
                        }}
                      />
                    ))}
                  </div>
                ) : isNew ? (
                  <Sparkles className="w-4 h-4 text-cyan-400 animate-pulse" />
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

            {/* Play count - hidden on mobile, positioned before duration */}
            {song.play_count > 0 && (
              <div className="hidden sm:block flex-shrink-0">
                <AnimatedLedDigits 
                  value={`${song.play_count}×`} 
                  variant="cyan"
                />
              </div>
            )}

            {/* Duration */}
            <div className="flex items-center gap-1 text-xs flex-shrink-0">
              <Clock className="w-3 h-3 hidden sm:block" />
              <AnimatedLedDigits value={formatDuration(song.duration)} variant="time" />
            </div>

            {/* Like Button */}
            <button
              onClick={async (e) => {
                e.stopPropagation();
                if (likingIds.has(song.id)) return;
                console.log('👆 Like button clicked for song:', song.id, song.title);
                setLikingIds(prev => new Set(prev).add(song.id));
                try {
                  await toggleSongLike(song.id);
                } catch (err) {
                  console.error('Failed to toggle like:', err);
                } finally {
                  setLikingIds(prev => {
                    const next = new Set(prev);
                    next.delete(song.id);
                    return next;
                  });
                }
              }}
              disabled={likingIds.has(song.id)}
              className={`p-1 transition-all flex-shrink-0 ${
                likingIds.has(song.id)
                  ? 'text-gray-400 cursor-wait'
                  : song.is_liked
                    ? 'text-red-500 hover:text-red-400'
                    : 'text-gray-500 hover:text-red-500 sm:opacity-0 group-hover:opacity-100'
              }`}
              title={song.is_liked ? 'Quitar de Me gusta' : 'Añadir a Me gusta'}
            >
              <Heart 
                className={`w-4 h-4 ${likingIds.has(song.id) ? 'animate-pulse' : ''}`}
                fill={song.is_liked ? 'currentColor' : 'none'} 
              />
            </button>

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
                  {/* Hide cover search for special playlists (Me gusta / Sugerencias) */}
                  {!playlist.is_favorites && !playlist.is_suggestions && (
                    <button
                      onClick={async (e) => {
                        e.stopPropagation();
                        if (fetchingCoverIds.has(song.id)) return;
                        setFetchingCoverIds(prev => new Set(prev).add(song.id));
                        try {
                          await fetchSongCover(song.id);
                        } finally {
                          setFetchingCoverIds(prev => {
                            const next = new Set(prev);
                            next.delete(song.id);
                            return next;
                          });
                          setMenuOpen(null);
                        }
                      }}
                      disabled={fetchingCoverIds.has(song.id)}
                      className="w-full px-3 py-2 text-left text-sm text-gray-300 hover:bg-gray-700 flex items-center gap-2 disabled:opacity-50"
                    >
                      <ImageIcon className={`w-4 h-4 ${fetchingCoverIds.has(song.id) ? 'animate-pulse' : ''}`} />
                      {fetchingCoverIds.has(song.id) ? 'Buscando...' : 'Buscar carátula'}
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
    </div>
  );
};

export default SongList;
