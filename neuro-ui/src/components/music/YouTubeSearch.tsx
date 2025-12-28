import React, { useState } from 'react';
import { Search, Plus, Loader2, Music, ExternalLink, Sparkles, Database } from 'lucide-react';
import { useSearchState, formatDuration } from '../../stores/musicStore';
import { EnrichedSearchResultDto, CreateSongRequest } from '../../api/client';
import { AnimatedLedDigits } from '../common';

interface YouTubeSearchProps {
  playlistId: string;
  onSongAdded?: () => void;
}

export const YouTubeSearch: React.FC<YouTubeSearchProps> = ({ playlistId, onSongAdded }) => {
  // Use optimized selector for search state
  const { searchResults, isSearching, searchYouTube, addSong, clearSearch } = useSearchState();
  const [query, setQuery] = useState('');
  const [addingIds, setAddingIds] = useState<Set<string>>(new Set());

  const handleSearch = async (e: React.FormEvent) => {
    e.preventDefault();
    if (query.trim()) {
      await searchYouTube(query);
    }
  };

  const handleAddSong = async (result: EnrichedSearchResultDto) => {
    if (addingIds.has(result.video_id)) return;
    
    setAddingIds(prev => new Set(prev).add(result.video_id));
    
    try {
      // Use enriched metadata
      const request: CreateSongRequest = {
        youtube_url: `https://www.youtube.com/watch?v=${result.video_id}`,
        title: result.title,
        artist: result.artist || result.channel || undefined,
        album: result.album || undefined,
      };
      
      await addSong(playlistId, request);
      onSongAdded?.();
    } finally {
      setAddingIds(prev => {
        const next = new Set(prev);
        next.delete(result.video_id);
        return next;
      });
    }
  };

  const handleAddByUrl = async () => {
    // Check if query is a YouTube URL
    const urlPattern = /(?:youtube\.com\/watch\?v=|youtu\.be\/|youtube\.com\/shorts\/)([a-zA-Z0-9_-]{11})/;
    const match = query.match(urlPattern);
    
    if (match) {
      const videoId = match[1];
      setAddingIds(prev => new Set(prev).add(videoId));
      
      try {
        await addSong(playlistId, { youtube_url: query });
        setQuery('');
        clearSearch();
        onSongAdded?.();
      } finally {
        setAddingIds(prev => {
          const next = new Set(prev);
          next.delete(videoId);
          return next;
        });
      }
    }
  };

  const isYouTubeUrl = /(?:youtube\.com|youtu\.be)/.test(query);

  return (
    <div className="space-y-4">
      {/* Search form */}
      <form onSubmit={handleSearch} className="relative">
        <div className="flex gap-2">
          <div className="relative flex-1">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-5 h-5 text-gray-500" />
            <input
              type="text"
              value={query}
              onChange={(e) => setQuery(e.target.value)}
              placeholder="Buscar en YouTube o pegar URL..."
              className="w-full pl-10 pr-4 py-2 bg-gray-800 border border-gray-700 text-white placeholder-gray-500 focus:border-cyan-500 focus:ring-1 focus:ring-cyan-500 transition-all"
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

      {/* Results */}
      {searchResults.length > 0 && (
        <div className="space-y-2 max-h-96 overflow-y-auto">
          <div className="text-sm text-gray-500 mb-2">
            {searchResults.length} resultados
          </div>
          
          {searchResults.map((result) => {
            const isAdding = addingIds.has(result.video_id);
            const isEnriched = result.source === 'music_brainz' || result.source === 'llm_inference';
            
            return (
              <div
                key={result.video_id}
                className="flex items-center gap-3 p-3 bg-gray-800/50 hover:bg-gray-800 transition-all group"
              >
                {/* Thumbnail */}
                <div className="w-20 h-12 bg-gray-700 overflow-hidden flex-shrink-0 relative">
                  <img
                    src={result.thumbnail}
                    alt={result.title}
                    className="w-full h-full object-cover"
                  />
                  <div className="absolute bottom-1 right-1 px-1 bg-black/80 text-[10px]">
                    <AnimatedLedDigits value={formatDuration(result.duration)} variant="time" />
                  </div>
                </div>

                {/* Info - Show enriched metadata */}
                <div className="flex-1 min-w-0">
                  <div className="font-medium text-white truncate text-sm flex items-center gap-1.5">
                    {result.title}
                    {isEnriched && (
                      <span title={result.source === 'music_brainz' ? 'Metadata de MusicBrainz' : 'Metadata generada por IA'}>
                        {result.source === 'music_brainz' ? (
                          <Database className="w-3 h-3 text-cyan-400" />
                        ) : (
                          <Sparkles className="w-3 h-3 text-purple-400" />
                        )}
                      </span>
                    )}
                  </div>
                  <div className="text-xs text-gray-400 truncate flex items-center gap-2">
                    {result.artist ? (
                      <span className="text-cyan-400">{result.artist}</span>
                    ) : (
                      <span>{result.channel}</span>
                    )}
                    {result.album && (
                      <span className="text-gray-500">• {result.album}</span>
                    )}
                    {result.view_count && (
                      <span className="text-gray-600">
                        • {(result.view_count / 1000000).toFixed(1)}M vistas
                      </span>
                    )}
                  </div>
                </div>

                {/* Actions */}
                <div className="flex items-center gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
                  <a
                    href={`https://www.youtube.com/watch?v=${result.video_id}`}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="p-2 text-gray-400 hover:text-white transition-colors"
                    onClick={(e) => e.stopPropagation()}
                  >
                    <ExternalLink className="w-4 h-4" />
                  </a>
                  <button
                    onClick={() => handleAddSong(result)}
                    disabled={isAdding}
                    className="p-2 bg-cyan-500/20 text-cyan-400 hover:bg-cyan-500/30 transition-all disabled:opacity-50"
                  >
                    {isAdding ? (
                      <Loader2 className="w-4 h-4 animate-spin" />
                    ) : (
                      <Plus className="w-4 h-4" />
                    )}
                  </button>
                </div>
              </div>
            );
          })}
        </div>
      )}

      {/* Empty state after search */}
      {!isSearching && searchResults.length === 0 && query && !isYouTubeUrl && (
        <div className="flex flex-col items-center justify-center py-8 text-gray-500">
          <Music className="w-12 h-12 mb-3 opacity-30" />
          <p className="text-sm">No se encontraron resultados</p>
        </div>
      )}
    </div>
  );
};

export default YouTubeSearch;
