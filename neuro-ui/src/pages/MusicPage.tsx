import { useEffect, useState } from 'react';
import { 
  Music, 
  Settings, 
  Clock, 
  Shuffle, 
  Repeat, 
  Plus,
  Activity,
  RefreshCw
} from 'lucide-react';
import TypewriterText from '../components/common/TypewriterText';
import { AnimatedLedDigits } from '../components/common';
import { useMusicStore, usePlayerState, useCurrentPlaylistDetail, formatDurationLong } from '../stores/musicStore';
import { PlaylistDto } from '../api/client';
import {
  MusicPlayer,
  SpectrumAnalyzer,
  Equalizer,
  PlaylistList,
  SongList,
  CreatePlaylistModal,
  AddSongsModal,
} from '../components/music';

type TabType = 'songs' | 'equalizer';

export default function MusicPage() {
  // Use optimized selectors
  const { currentPlaylistDetail, fetchPlaylistDetail } = useCurrentPlaylistDetail();
  const player = usePlayerState();
  
  // Get other actions from store directly
  const {
    fetchPlaylists,
    fetchEqualizer,
    refreshSuggestions,
    startWatchingPlaylist,
    stopWatchingPlaylist,
  } = useMusicStore();

  const [selectedPlaylistId, setSelectedPlaylistId] = useState<string | null>(null);
  const [activeTab, setActiveTab] = useState<TabType>('songs');
  const [showCreateModal, setShowCreateModal] = useState(false);
  const [showAddSongsModal, setShowAddSongsModal] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [isRefreshingSuggestions, setIsRefreshingSuggestions] = useState(false);

  // Load initial data
  useEffect(() => {
    fetchPlaylists();
    fetchEqualizer();
  }, [fetchPlaylists, fetchEqualizer]);

  // Load playlist detail and start watching for updates when selected
  useEffect(() => {
    if (selectedPlaylistId) {
      fetchPlaylistDetail(selectedPlaylistId);
      // Start SSE watching for real-time updates
      startWatchingPlaylist(selectedPlaylistId);
    }
    
    // Cleanup: stop watching when component unmounts or playlist changes
    return () => {
      stopWatchingPlaylist();
    };
  }, [selectedPlaylistId, fetchPlaylistDetail, startWatchingPlaylist, stopWatchingPlaylist]);

  const handleSelectPlaylist = (playlist: PlaylistDto) => {
    setSelectedPlaylistId(playlist.id);
    setActiveTab('songs');
  };

  const handlePlaylistCreated = (playlistId: string) => {
    setSelectedPlaylistId(playlistId);
    setActiveTab('songs');
  };

  return (
    <div className="h-full flex flex-col bg-gradient-to-br from-gray-950 via-gray-900 to-gray-950 overflow-hidden">
      {/* Cyberpunk grid background */}
      <div 
        className="fixed inset-0 pointer-events-none opacity-5"
        style={{
          backgroundImage: `
            linear-gradient(to right, cyan 1px, transparent 1px),
            linear-gradient(to bottom, cyan 1px, transparent 1px)
          `,
          backgroundSize: '40px 40px',
        }}
      />
      
      {/* Gradient overlays */}
      <div className="fixed top-0 left-0 w-1/3 h-1/3 bg-gradient-to-br from-cyan-500/10 to-transparent pointer-events-none" />
      <div className="fixed bottom-0 right-0 w-1/3 h-1/3 bg-gradient-to-tl from-purple-500/10 to-transparent pointer-events-none" />

      {/* Main content */}
      <div className="flex-1 flex overflow-hidden relative">
        {/* Left sidebar - Playlists (hidden on mobile when playlist selected) */}
        <aside className={`${selectedPlaylistId ? 'hidden md:flex' : 'flex'} w-full md:w-64 lg:w-72 border-r border-cyan-500/20 bg-gray-900/50 flex-col absolute md:relative inset-0 z-10 md:z-auto`}>
          <PlaylistList
            onSelectPlaylist={handleSelectPlaylist}
            selectedPlaylistId={selectedPlaylistId || undefined}
            onCreatePlaylist={() => setShowCreateModal(true)}
          />
        </aside>

        {/* Main area */}
        <main className={`${selectedPlaylistId ? 'flex' : 'hidden md:flex'} flex-1 flex-col overflow-hidden`}>
          {selectedPlaylistId && currentPlaylistDetail ? (
            <>
              {/* Playlist header */}
              <header className="p-3 sm:p-4 lg:p-6 border-b border-cyan-500/20 bg-gradient-to-r from-gray-900/80 to-gray-900/50 backdrop-blur">
                <div className="flex items-start gap-3 sm:gap-4 lg:gap-6">
                  {/* Back button for mobile */}
                  <button
                    onClick={() => setSelectedPlaylistId(null)}
                    className="md:hidden p-2 text-cyan-400 hover:bg-cyan-500/20 rounded-lg flex-shrink-0"
                  >
                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
                    </svg>
                  </button>
                  {/* Cover */}
                  <div className="w-16 h-16 sm:w-24 sm:h-24 lg:w-32 lg:h-32 rounded-xl bg-gray-800 overflow-hidden shadow-2xl relative flex-shrink-0">
                    {currentPlaylistDetail.cover_url ? (
                      <img
                        src={currentPlaylistDetail.cover_url}
                        alt={currentPlaylistDetail.name}
                        className="w-full h-full object-cover"
                      />
                    ) : (
                      <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyan-500/20 to-purple-500/20">
                        <Music className="w-6 h-6 sm:w-8 sm:h-8 lg:w-12 lg:h-12 text-gray-600" />
                      </div>
                    )}
                    
                    {/* Glow effect */}
                    <div className="absolute inset-0 bg-gradient-to-t from-cyan-500/20 to-transparent" />
                  </div>

                  {/* Info */}
                  <div className="flex-1 min-w-0">
                    <div className="text-xs text-cyber-cyan uppercase tracking-wider mb-1 font-mono hidden sm:block">
                      Playlist
                    </div>
                    <h1 className="text-lg sm:text-xl lg:text-3xl font-cyber font-bold text-cyber-cyan mb-1 sm:mb-2 truncate">
                      <TypewriterText text={currentPlaylistDetail.name} speed={20} />
                    </h1>
                    {currentPlaylistDetail.description && (
                      <p className="text-gray-400 text-xs sm:text-sm mb-2 sm:mb-3 line-clamp-1 sm:line-clamp-2 hidden sm:block">
                        {currentPlaylistDetail.description}
                      </p>
                    )}
                    <div className="flex flex-wrap items-center gap-2 sm:gap-4 text-xs sm:text-sm text-gray-500 font-mono">
                      <span className="flex items-center gap-1">
                        <Music className="w-3 h-3 sm:w-4 sm:h-4" />
                        <AnimatedLedDigits value={`${currentPlaylistDetail.song_count} canciones`} variant="time" />
                      </span>
                      <span className="flex items-center gap-1">
                        <Clock className="w-3 h-3 sm:w-4 sm:h-4" />
                        <AnimatedLedDigits value={formatDurationLong(currentPlaylistDetail.total_duration)} variant="time" />
                      </span>
                      {currentPlaylistDetail.shuffle && (
                        <span className="hidden sm:flex items-center gap-1 text-cyan-400">
                          <Shuffle className="w-4 h-4" />
                          Aleatorio
                        </span>
                      )}
                      {currentPlaylistDetail.repeat_mode !== 'off' && (
                        <span className="hidden sm:flex items-center gap-1 text-cyan-400">
                          <Repeat className="w-4 h-4" />
                          {currentPlaylistDetail.repeat_mode === 'one' ? 'Una' : 'Todas'}
                        </span>
                      )}
                    </div>
                  </div>

                  {/* Settings button */}
                  <button
                    onClick={() => setShowSettings(!showSettings)}
                    className={`p-2 rounded-lg transition-all hidden sm:block ${
                      showSettings 
                        ? 'bg-cyan-500 text-black' 
                        : 'text-gray-400 hover:text-white hover:bg-gray-800'
                    }`}
                  >
                    <Settings className="w-5 h-5" />
                  </button>
                </div>

                {/* Tabs */}
                <div className="flex flex-wrap gap-1 mt-3 sm:mt-6">
                  {[
                    { id: 'songs' as TabType, label: 'Canciones', icon: Music },
                    { id: 'equalizer' as TabType, label: 'Ecualizador', icon: Activity },
                  ].map((tab) => (
                    <button
                      key={tab.id}
                      onClick={() => setActiveTab(tab.id)}
                      className={`flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-1.5 sm:py-2 rounded-lg text-xs sm:text-sm font-medium transition-all font-mono ${
                        activeTab === tab.id
                          ? 'bg-cyan-500/20 text-cyan-400 border border-cyan-500/50'
                          : 'text-gray-400 hover:text-white hover:bg-gray-800'
                      }`}
                    >
                      <tab.icon className="w-3.5 h-3.5 sm:w-4 sm:h-4" />
                      <span className="hidden sm:inline">{tab.label}</span>
                    </button>
                  ))}
                  
                  {/* Add Songs button or Refresh Suggestions button */}
                  {currentPlaylistDetail.is_suggestions ? (
                    <button
                      onClick={async () => {
                        setIsRefreshingSuggestions(true);
                        try {
                          await refreshSuggestions();
                        } finally {
                          setIsRefreshingSuggestions(false);
                        }
                      }}
                      disabled={isRefreshingSuggestions}
                      className="flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-1.5 sm:py-2 rounded-lg text-xs sm:text-sm font-medium transition-all ml-auto bg-purple-500 text-white hover:bg-purple-400 font-mono disabled:opacity-50 disabled:cursor-wait"
                    >
                      <RefreshCw className={`w-3.5 h-3.5 sm:w-4 sm:h-4 ${isRefreshingSuggestions ? 'animate-spin' : ''}`} />
                      <span className="hidden sm:inline">{isRefreshingSuggestions ? 'Actualizando...' : 'Actualizar Sugerencias'}</span>
                      <span className="sm:hidden">{isRefreshingSuggestions ? '...' : 'Actualizar'}</span>
                    </button>
                  ) : (
                    <button
                      onClick={() => setShowAddSongsModal(true)}
                      className="flex items-center gap-1.5 sm:gap-2 px-2.5 sm:px-4 py-1.5 sm:py-2 rounded-lg text-xs sm:text-sm font-medium transition-all ml-auto bg-cyan-500 text-black hover:bg-cyan-400 font-mono"
                    >
                      <Plus className="w-3.5 h-3.5 sm:w-4 sm:h-4" />
                      <span className="hidden sm:inline">Agregar Canciones</span>
                      <span className="sm:hidden">Agregar</span>
                    </button>
                  )}
                </div>
              </header>

              {/* Tab content */}
              <div className="flex-1 overflow-y-auto">
                {activeTab === 'songs' && (
                  <SongList playlist={currentPlaylistDetail} />
                )}
                
                {activeTab === 'equalizer' && (
                  <div className="p-2 sm:p-4">
                    <Equalizer />
                  </div>
                )}
              </div>
            </>
          ) : (
            /* Empty state */
            <div className="flex-1 flex flex-col items-center justify-center text-gray-500 p-4">
              <div className="relative">
                <Music className="w-16 h-16 sm:w-24 sm:h-24 opacity-20" />
                <div className="absolute inset-0 bg-gradient-to-t from-cyan-500/10 to-transparent rounded-full blur-xl" />
              </div>
              <h2 className="text-lg sm:text-xl font-bold text-white mt-4 sm:mt-6 mb-2 font-cyber text-center">
                <TypewriterText text="Selecciona una playlist" speed={30} />
              </h2>
              <p className="text-xs sm:text-sm text-gray-500 mb-4 sm:mb-6 font-mono text-center">
                O crea una nueva para comenzar
              </p>
              <button
                onClick={() => setShowCreateModal(true)}
                className="flex items-center gap-2 px-4 py-2 bg-cyan-500 text-black font-medium rounded-lg hover:bg-cyan-400 transition-all font-mono text-sm"
              >
                <Plus className="w-4 h-4" />
                Nueva Playlist
              </button>
            </div>
          )}
        </main>

        {/* Right sidebar - Now Playing (hidden on mobile) */}
        {player.currentSong && (
          <aside className="hidden lg:flex w-80 border-l border-cyan-500/20 bg-gray-900/50 flex-col">
            {/* Now playing header */}
            <div className="p-4 border-b border-cyber-cyan/20">
              <h3 className="text-cyber-cyan font-cyber font-bold text-sm tracking-wider uppercase">
                <TypewriterText text="Reproduciendo" speed={40} />
              </h3>
            </div>

            {/* Album art */}
            <div className="p-4">
              <div className="aspect-square rounded-xl bg-gray-800 overflow-hidden shadow-2xl relative">
                {player.currentSong.cover_url || player.currentSong.thumbnail_url ? (
                  <img
                    src={player.currentSong.cover_url || player.currentSong.thumbnail_url}
                    alt={player.currentSong.title}
                    className="w-full h-full object-cover"
                  />
                ) : (
                  <div className="w-full h-full flex items-center justify-center bg-gradient-to-br from-cyan-500/20 to-purple-500/20">
                    <Music className="w-16 h-16 text-gray-600" />
                  </div>
                )}
                
                {/* Animated ring when playing */}
                {player.isPlaying && (
                  <div className="absolute inset-0 border-2 border-cyan-500/50 rounded-xl animate-pulse" />
                )}
              </div>
            </div>

            {/* Song info */}
            <div className="px-4 text-center">
              <h4 className="font-cyber font-bold text-cyber-cyan text-lg truncate">
                <TypewriterText text={player.currentSong.title} speed={25} />
              </h4>
              <p className="text-gray-400 text-sm truncate font-mono">
                {player.currentSong.artist || 'Artista desconocido'}
              </p>
            </div>

            {/* Mini spectrum */}
            <div className="flex-1 p-4">
              <SpectrumAnalyzer className="h-full" barCount={16} showReflection={false} />
            </div>
          </aside>
        )}
      </div>

      {/* Bottom player bar */}
      <MusicPlayer />

      {/* Create playlist modal */}
      <CreatePlaylistModal
        isOpen={showCreateModal}
        onClose={() => setShowCreateModal(false)}
        onCreated={handlePlaylistCreated}
      />
      
      {/* Add songs modal */}
      {currentPlaylistDetail && (
        <AddSongsModal
          playlistId={currentPlaylistDetail.id}
          isOpen={showAddSongsModal}
          onClose={() => setShowAddSongsModal(false)}
          onSongsAdded={() => fetchPlaylistDetail(currentPlaylistDetail.id)}
        />
      )}
    </div>
  );
}
