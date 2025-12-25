import React, { useState, useRef, useCallback, useEffect } from 'react';
import { 
  Search, 
  Plus, 
  Loader2, 
  Music, 
  X, 
  CheckSquare
} from 'lucide-react';
import { useMusicStore, formatDuration } from '../../stores/musicStore';
import { YouTubeSearchResultDto, CreateSongRequest, musicApi } from '../../api/client';

interface AddSongsModalProps {
  playlistId: string;
  isOpen: boolean;
  onClose: () => void;
  onSongsAdded?: () => void;
}

const PAGE_SIZE = 10;

export const AddSongsModal: React.FC<AddSongsModalProps> = ({ 
  playlistId, 
  isOpen, 
  onClose,
  onSongsAdded 
}) => {
  const { addSong, deleteSong, currentPlaylistDetail } = useMusicStore();
  
  // Search state
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<YouTubeSearchResultDto[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [currentLimit, setCurrentLimit] = useState(PAGE_SIZE);
  const [lastQuery, setLastQuery] = useState('');
  
  // Adding/removing state
  const [processingIds, setProcessingIds] = useState<Set<string>>(new Set());
  
  // Map of video_id -> song_id for songs in the playlist
  const songIdMap = React.useMemo(() => {
    const map = new Map<string, string>();
    if (currentPlaylistDetail?.id === playlistId) {
      currentPlaylistDetail.songs.forEach(song => {
        map.set(song.youtube_id, song.id);
      });
    }
    return map;
  }, [currentPlaylistDetail, playlistId]);
  
  // Refs
  const scrollContainerRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  
  // Focus input when modal opens
  useEffect(() => {
    if (isOpen && inputRef.current) {
      inputRef.current.focus();
    }
  }, [isOpen]);
  
  // Reset state when modal closes
  useEffect(() => {
    if (!isOpen) {
      setQuery('');
      setResults([]);
      setCurrentLimit(PAGE_SIZE);
      setLastQuery('');
      setHasMore(false);
    }
  }, [isOpen]);
  
  // Search handler
  const handleSearch = async (searchQuery: string, limit: number = PAGE_SIZE) => {
    if (!searchQuery.trim()) {
      setResults([]);
      setHasMore(false);
      return;
    }
    
    setIsSearching(true);
    try {
      const searchResults = await musicApi.searchYouTube(searchQuery, limit);
      setResults(searchResults);
      setLastQuery(searchQuery);
      setCurrentLimit(limit);
      // If we got the full amount requested, there might be more
      setHasMore(searchResults.length >= limit);
    } catch (error) {
      console.error('Search failed:', error);
    } finally {
      setIsSearching(false);
    }
  };
  
  // Form submit handler
  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    handleSearch(query, PAGE_SIZE);
  };
  
  // Load more (infinite scroll)
  const loadMore = useCallback(async () => {
    if (isSearching || !hasMore || !lastQuery) return;
    
    const newLimit = currentLimit + PAGE_SIZE;
    await handleSearch(lastQuery, newLimit);
  }, [isSearching, hasMore, lastQuery, currentLimit]);
  
  // Scroll handler for infinite scroll
  const handleScroll = useCallback(() => {
    const container = scrollContainerRef.current;
    if (!container) return;
    
    const { scrollTop, scrollHeight, clientHeight } = container;
    // Load more when user scrolls to bottom (with 100px threshold)
    if (scrollHeight - scrollTop - clientHeight < 100) {
      loadMore();
    }
  }, [loadMore]);
  
  // Toggle song (add or remove)
  const handleToggleSong = async (result: YouTubeSearchResultDto) => {
    if (processingIds.has(result.video_id)) return;
    
    setProcessingIds(prev => new Set(prev).add(result.video_id));
    
    try {
      const existingSongId = songIdMap.get(result.video_id);
      
      if (existingSongId) {
        // Song exists - remove it
        await deleteSong(playlistId, existingSongId);
      } else {
        // Song doesn't exist - add it
        const request: CreateSongRequest = {
          youtube_url: `https://www.youtube.com/watch?v=${result.video_id}`,
          title: result.title,
          artist: result.channel,
        };
        await addSong(playlistId, request);
      }
      onSongsAdded?.();
    } finally {
      setProcessingIds(prev => {
        const next = new Set(prev);
        next.delete(result.video_id);
        return next;
      });
    }
  };
  
  // Check if query is a YouTube URL
  const isYouTubeUrl = /(?:youtube\.com|youtu\.be)/.test(query);
  
  // Handle adding by URL
  const handleAddByUrl = async () => {
    const urlPattern = /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/shorts\/)([a-zA-Z0-9_-]{11})/;
    const match = query.match(urlPattern);
    
    if (match) {
      const videoId = match[1];
      setProcessingIds(prev => new Set(prev).add(videoId));
      
      try {
        await addSong(playlistId, { youtube_url: query });
        setQuery('');
        onSongsAdded?.();
      } finally {
        setProcessingIds(prev => {
          const next = new Set(prev);
          next.delete(videoId);
          return next;
        });
      }
    }
  };
  
  if (!isOpen) return null;
  
  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div 
        className="absolute inset-0 bg-black/80 backdrop-blur-sm"
        onClick={onClose}
      />
      
      {/* Modal */}
      <div className="relative w-full max-w-2xl max-h-[85vh] bg-gray-900 rounded-2xl border border-cyan-500/30 shadow-2xl shadow-cyan-500/20 flex flex-col overflow-hidden">
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-800">
          <h2 className="text-lg font-bold text-white flex items-center gap-2">
            <Plus className="w-5 h-5 text-cyan-400" />
            Agregar Canciones
          </h2>
          <button
            onClick={onClose}
            className="p-2 text-gray-400 hover:text-white hover:bg-gray-800 transition-all"
          >
            <X className="w-5 h-5" />
          </button>
        </div>
        
        {/* Search form */}
        <form onSubmit={handleSubmit} className="p-4 border-b border-gray-800">
          <div className="flex gap-2">
            <div className="relative flex-1">
              <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
              <input
                ref={inputRef}
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Buscar en YouTube o pegar URL..."
                className="w-full pl-10 pr-4 py-2.5 bg-gray-800 border border-gray-700 text-white placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition-all"
              />
            </div>
            
            {isYouTubeUrl ? (
              <button
                type="button"
                onClick={handleAddByUrl}
                disabled={processingIds.size > 0}
                className="px-4 py-2 bg-cyan-500 text-black font-medium hover:bg-cyan-400 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                {processingIds.size > 0 ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Plus className="w-4 h-4" />
                )}
                Agregar
              </button>
            ) : (
              <button
                type="submit"
                disabled={isSearching || !query.trim()}
                className="px-4 py-2 bg-gray-700 text-white hover:bg-gray-600 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                {isSearching ? (
                  <Loader2 className="w-4 h-4 animate-spin" />
                ) : (
                  <Search className="w-4 h-4" />
                )}
                Buscar
              </button>
            )}
          </div>
        </form>
        
        {/* Results */}
        <div 
          ref={scrollContainerRef}
          onScroll={handleScroll}
          className="flex-1 overflow-y-auto p-2"
        >
          {results.length > 0 ? (
            <div className="space-y-1">
              {results.map((result) => {
                const isProcessing = processingIds.has(result.video_id);
                const isInPlaylist = songIdMap.has(result.video_id);
                
                return (
                  <div
                    key={result.video_id}
                    onClick={() => !isProcessing && handleToggleSong(result)}
                    className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all ${
                      isProcessing
                        ? 'bg-cyan-500/20 border border-cyan-500/50 cursor-wait'
                        : isInPlaylist 
                          ? 'bg-green-500/10 border border-green-500/30 hover:bg-red-500/10 hover:border-red-500/30' 
                          : 'bg-gray-800/50 border border-transparent hover:bg-gray-800 hover:border-gray-700 active:bg-cyan-500/20'
                    }`}
                  >
                    {/* Thumbnail */}
                    <div className="w-20 h-12 rounded bg-gray-700 overflow-hidden flex-shrink-0 relative">
                      <img
                        src={result.thumbnail}
                        alt={result.title}
                        className="w-full h-full object-cover"
                      />
                      <div className="absolute bottom-1 right-1 px-1 bg-black/80 text-[10px] rounded led-time">
                        {formatDuration(result.duration)}
                      </div>
                    </div>

                    {/* Info */}
                    <div className="flex-1 min-w-0">
                      <div className={`font-medium truncate text-sm ${isInPlaylist ? 'text-green-400' : 'text-white'}`}>
                        {result.title}
                      </div>
                      <div className="text-xs text-gray-400 truncate flex items-center gap-2">
                        <span>{result.channel}</span>
                        {result.view_count && (
                          <span className="text-gray-600">
                            {(result.view_count / 1000000).toFixed(1)}M vistas
                          </span>
                        )}
                      </div>
                    </div>

                    {/* Status indicator */}
                    <div className="flex items-center gap-2 flex-shrink-0">
                      {isProcessing ? (
                        <Loader2 className="w-4 h-4 text-cyan-400 animate-spin" />
                      ) : isInPlaylist ? (
                        <span className="px-2 py-1 text-xs text-green-400 bg-green-500/20 rounded flex items-center gap-1 group-hover:hidden">
                          <CheckSquare className="w-3 h-3" />
                          Agregado
                        </span>
                      ) : (
                        <Plus className="w-4 h-4 text-gray-500" />
                      )}
                    </div>
                  </div>
                );
              })}
              
              {/* Load more indicator */}
              {isSearching && (
                <div className="flex items-center justify-center py-4">
                  <Loader2 className="w-6 h-6 text-cyan-400 animate-spin" />
                </div>
              )}
              
              {hasMore && !isSearching && (
                <button
                  onClick={loadMore}
                  className="w-full py-3 text-sm text-gray-400 hover:text-cyan-400 transition-colors"
                >
                  Cargar más resultados...
                </button>
              )}
            </div>
          ) : !isSearching && query && !isYouTubeUrl ? (
            <div className="flex flex-col items-center justify-center py-12 text-gray-500">
              <Music className="w-12 h-12 mb-3 opacity-30" />
              <p className="text-sm">No se encontraron resultados</p>
            </div>
          ) : !query ? (
            <div className="flex flex-col items-center justify-center py-12 text-gray-500">
              <Search className="w-12 h-12 mb-3 opacity-30" />
              <p className="text-sm">Busca canciones en YouTube</p>
              <p className="text-xs text-gray-600 mt-1">
                O pega una URL de YouTube directamente
              </p>
            </div>
          ) : null}
        </div>
        
        {/* Footer */}
        <div className="flex items-center justify-end p-4 border-t border-gray-800 bg-gray-900/80">
          <button
            onClick={onClose}
            className="px-4 py-2 text-gray-400 hover:text-white transition-colors"
          >
            Cerrar
          </button>
        </div>
      </div>
    </div>
  );
};
