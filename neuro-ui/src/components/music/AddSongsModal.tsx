import React, { useState, useRef, useCallback, useEffect } from 'react';
import { 
  Search, 
  Plus, 
  Loader2, 
  Music, 
  X, 
  ExternalLink,
  CheckSquare,
  Square,
  MinusSquare
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
  const { addSong } = useMusicStore();
  
  // Search state
  const [query, setQuery] = useState('');
  const [results, setResults] = useState<YouTubeSearchResultDto[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [hasMore, setHasMore] = useState(false);
  const [currentLimit, setCurrentLimit] = useState(PAGE_SIZE);
  const [lastQuery, setLastQuery] = useState('');
  
  // Selection state
  const [selectedIds, setSelectedIds] = useState<Set<string>>(new Set());
  
  // Adding state
  const [isAddingMultiple, setIsAddingMultiple] = useState(false);
  const [addingIds, setAddingIds] = useState<Set<string>>(new Set());
  const [addedIds, setAddedIds] = useState<Set<string>>(new Set());
  const [addProgress, setAddProgress] = useState({ current: 0, total: 0 });
  
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
      setSelectedIds(new Set());
      setAddedIds(new Set());
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
    setSelectedIds(new Set());
    setAddedIds(new Set());
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
  
  // Selection handlers
  const toggleSelection = (videoId: string) => {
    setSelectedIds(prev => {
      const next = new Set(prev);
      if (next.has(videoId)) {
        next.delete(videoId);
      } else {
        next.add(videoId);
      }
      return next;
    });
  };
  
  const selectAll = () => {
    const selectableIds = results
      .filter(r => !addedIds.has(r.video_id))
      .map(r => r.video_id);
    setSelectedIds(new Set(selectableIds));
  };
  
  const deselectAll = () => {
    setSelectedIds(new Set());
  };
  
  const isAllSelected = results.length > 0 && 
    results.filter(r => !addedIds.has(r.video_id)).every(r => selectedIds.has(r.video_id));
  const isSomeSelected = selectedIds.size > 0 && !isAllSelected;
  
  // Add single song
  const handleAddSingle = async (result: YouTubeSearchResultDto) => {
    if (addingIds.has(result.video_id) || addedIds.has(result.video_id)) return;
    
    setAddingIds(prev => new Set(prev).add(result.video_id));
    
    try {
      const request: CreateSongRequest = {
        youtube_url: `https://www.youtube.com/watch?v=${result.video_id}`,
        title: result.title,
        artist: result.channel,
      };
      
      await addSong(playlistId, request);
      setAddedIds(prev => new Set(prev).add(result.video_id));
      setSelectedIds(prev => {
        const next = new Set(prev);
        next.delete(result.video_id);
        return next;
      });
      onSongsAdded?.();
    } finally {
      setAddingIds(prev => {
        const next = new Set(prev);
        next.delete(result.video_id);
        return next;
      });
    }
  };
  
  // Add selected songs
  const handleAddSelected = async () => {
    if (selectedIds.size === 0 || isAddingMultiple) return;
    
    setIsAddingMultiple(true);
    const selectedResults = results.filter(r => selectedIds.has(r.video_id));
    setAddProgress({ current: 0, total: selectedResults.length });
    
    try {
      for (let i = 0; i < selectedResults.length; i++) {
        const result = selectedResults[i];
        setAddingIds(prev => new Set(prev).add(result.video_id));
        
        try {
          const request: CreateSongRequest = {
            youtube_url: `https://www.youtube.com/watch?v=${result.video_id}`,
            title: result.title,
            artist: result.channel,
          };
          
          await addSong(playlistId, request);
          setAddedIds(prev => new Set(prev).add(result.video_id));
        } catch (error) {
          console.error(`Failed to add ${result.title}:`, error);
        }
        
        setAddingIds(prev => {
          const next = new Set(prev);
          next.delete(result.video_id);
          return next;
        });
        setAddProgress({ current: i + 1, total: selectedResults.length });
      }
      
      setSelectedIds(new Set());
      onSongsAdded?.();
    } finally {
      setIsAddingMultiple(false);
      setAddProgress({ current: 0, total: 0 });
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
      setAddingIds(prev => new Set(prev).add(videoId));
      
      try {
        await addSong(playlistId, { youtube_url: query });
        setQuery('');
        setAddedIds(prev => new Set(prev).add(videoId));
        onSongsAdded?.();
      } finally {
        setAddingIds(prev => {
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
                disabled={addingIds.size > 0}
                className="px-4 py-2 bg-cyan-500 text-black font-medium hover:bg-cyan-400 transition-all flex items-center gap-2 disabled:opacity-50"
              >
                {addingIds.size > 0 ? (
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
        
        {/* Selection toolbar */}
        {results.length > 0 && (
          <div className="flex items-center justify-between px-4 py-2 bg-gray-800/50 border-b border-gray-800">
            <div className="flex items-center gap-3">
              <button
                onClick={isAllSelected ? deselectAll : selectAll}
                className="flex items-center gap-2 text-sm text-gray-400 hover:text-white transition-colors"
              >
                {isAllSelected ? (
                  <CheckSquare className="w-4 h-4 text-cyan-400" />
                ) : isSomeSelected ? (
                  <MinusSquare className="w-4 h-4 text-cyan-400" />
                ) : (
                  <Square className="w-4 h-4" />
                )}
                {isAllSelected ? 'Deseleccionar todo' : 'Seleccionar todo'}
              </button>
              
              {selectedIds.size > 0 && (
                <span className="text-sm text-cyan-400">
                  {selectedIds.size} seleccionado{selectedIds.size !== 1 ? 's' : ''}
                </span>
              )}
            </div>
            
            {selectedIds.size > 0 && (
              <button
                onClick={handleAddSelected}
                disabled={isAddingMultiple}
                className="flex items-center gap-2 px-3 py-1.5 bg-cyan-500 text-black font-medium text-sm hover:bg-cyan-400 transition-all disabled:opacity-50"
              >
                {isAddingMultiple ? (
                  <>
                    <Loader2 className="w-4 h-4 animate-spin" />
                    Agregando {addProgress.current}/{addProgress.total}...
                  </>
                ) : (
                  <>
                    <Plus className="w-4 h-4" />
                    Agregar {selectedIds.size}
                  </>
                )}
              </button>
            )}
          </div>
        )}
        
        {/* Results */}
        <div 
          ref={scrollContainerRef}
          onScroll={handleScroll}
          className="flex-1 overflow-y-auto p-2"
        >
          {results.length > 0 ? (
            <div className="space-y-1">
              {results.map((result) => {
                const isSelected = selectedIds.has(result.video_id);
                const isAdding = addingIds.has(result.video_id);
                const isAdded = addedIds.has(result.video_id);
                
                return (
                  <div
                    key={result.video_id}
                    onClick={() => !isAdded && !isAdding && toggleSelection(result.video_id)}
                    className={`flex items-center gap-3 p-3 rounded-lg cursor-pointer transition-all ${
                      isAdded 
                        ? 'bg-green-500/10 border border-green-500/30 cursor-default' 
                        : isSelected 
                          ? 'bg-cyan-500/20 border border-cyan-500/50' 
                          : 'bg-gray-800/50 border border-transparent hover:bg-gray-800 hover:border-gray-700'
                    }`}
                  >
                    {/* Checkbox */}
                    <div className="flex-shrink-0">
                      {isAdded ? (
                        <CheckSquare className="w-5 h-5 text-green-400" />
                      ) : isSelected ? (
                        <CheckSquare className="w-5 h-5 text-cyan-400" />
                      ) : (
                        <Square className="w-5 h-5 text-gray-500" />
                      )}
                    </div>
                    
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
                      <div className={`font-medium truncate text-sm ${isAdded ? 'text-green-400' : 'text-white'}`}>
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

                    {/* Actions */}
                    <div className="flex items-center gap-2 flex-shrink-0">
                      <a
                        href={`https://www.youtube.com/watch?v=${result.video_id}`}
                        target="_blank"
                        rel="noopener noreferrer"
                        className="p-2 text-gray-400 hover:text-white transition-colors"
                        onClick={(e) => e.stopPropagation()}
                      >
                        <ExternalLink className="w-4 h-4" />
                      </a>
                      
                      {isAdded ? (
                        <span className="px-2 py-1 text-xs text-green-400 bg-green-500/20 rounded">
                          Agregado
                        </span>
                      ) : (
                        <button
                          onClick={(e) => {
                            e.stopPropagation();
                            handleAddSingle(result);
                          }}
                          disabled={isAdding}
                          className="p-2 bg-cyan-500/20 text-cyan-400 rounded-lg hover:bg-cyan-500/30 transition-all disabled:opacity-50"
                        >
                          {isAdding ? (
                            <Loader2 className="w-4 h-4 animate-spin" />
                          ) : (
                            <Plus className="w-4 h-4" />
                          )}
                        </button>
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
        <div className="flex items-center justify-between p-4 border-t border-gray-800 bg-gray-900/80">
          <span className="text-sm text-gray-500">
            {addedIds.size > 0 && `${addedIds.size} canción${addedIds.size !== 1 ? 'es' : ''} agregada${addedIds.size !== 1 ? 's' : ''}`}
          </span>
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
