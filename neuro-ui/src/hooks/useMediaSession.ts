import { useEffect, useCallback } from 'react';
import { useMusicStore } from '../stores/musicStore';

/**
 * Hook para integrar la Media Session API
 * Permite que el audio se reproduzca en background en móviles
 * y muestra controles en las notificaciones del sistema
 */
export const useMediaSession = () => {
  const {
    player,
    togglePlay,
    nextSong,
    previousSong,
    seek,
  } = useMusicStore();

  // Actualizar metadata cuando cambia la canción
  const updateMediaMetadata = useCallback(() => {
    if (!('mediaSession' in navigator) || !player.currentSong) return;

    const { title, artist, album, cover_url, thumbnail_url } = player.currentSong;
    
    // Construir artwork con las imágenes disponibles
    const artwork: MediaImage[] = [];
    const imageUrl = cover_url || thumbnail_url;
    
    if (imageUrl) {
      artwork.push(
        { src: imageUrl, sizes: '96x96', type: 'image/jpeg' },
        { src: imageUrl, sizes: '128x128', type: 'image/jpeg' },
        { src: imageUrl, sizes: '192x192', type: 'image/jpeg' },
        { src: imageUrl, sizes: '256x256', type: 'image/jpeg' },
        { src: imageUrl, sizes: '384x384', type: 'image/jpeg' },
        { src: imageUrl, sizes: '512x512', type: 'image/jpeg' }
      );
    }

    navigator.mediaSession.metadata = new MediaMetadata({
      title: title || 'Sin título',
      artist: artist || 'Artista desconocido',
      album: album || '',
      artwork,
    });

    console.log('🎵 Media Session metadata updated:', title);
  }, [player.currentSong]);

  // Actualizar estado de reproducción
  const updatePlaybackState = useCallback(() => {
    if (!('mediaSession' in navigator)) return;

    navigator.mediaSession.playbackState = player.isPlaying ? 'playing' : 'paused';
  }, [player.isPlaying]);

  // Actualizar posición de reproducción
  const updatePositionState = useCallback(() => {
    if (!('mediaSession' in navigator) || !player.currentSong) return;

    try {
      // Solo actualizar si tenemos valores válidos
      if (player.duration > 0 && Number.isFinite(player.duration)) {
        navigator.mediaSession.setPositionState({
          duration: player.duration,
          playbackRate: 1,
          position: Math.min(player.currentTime, player.duration),
        });
      }
    } catch (error) {
      // Algunos navegadores pueden no soportar setPositionState
      console.log('🎵 Position state not supported:', error);
    }
  }, [player.currentTime, player.duration, player.currentSong]);

  // Configurar action handlers
  useEffect(() => {
    if (!('mediaSession' in navigator)) {
      console.log('🎵 Media Session API not supported');
      return;
    }

    console.log('🎵 Setting up Media Session handlers');

    // Play
    navigator.mediaSession.setActionHandler('play', () => {
      console.log('🎵 Media Session: play');
      if (!player.isPlaying) {
        togglePlay();
      }
    });

    // Pause
    navigator.mediaSession.setActionHandler('pause', () => {
      console.log('🎵 Media Session: pause');
      if (player.isPlaying) {
        togglePlay();
      }
    });

    // Previous track
    navigator.mediaSession.setActionHandler('previoustrack', () => {
      console.log('🎵 Media Session: previous');
      previousSong();
    });

    // Next track
    navigator.mediaSession.setActionHandler('nexttrack', () => {
      console.log('🎵 Media Session: next');
      nextSong();
    });

    // Seek backward (10 seconds)
    navigator.mediaSession.setActionHandler('seekbackward', (details) => {
      const skipTime = details.seekOffset || 10;
      const newTime = Math.max(player.currentTime - skipTime, 0);
      console.log('🎵 Media Session: seek backward', skipTime);
      seek(newTime);
    });

    // Seek forward (10 seconds)
    navigator.mediaSession.setActionHandler('seekforward', (details) => {
      const skipTime = details.seekOffset || 10;
      const newTime = Math.min(player.currentTime + skipTime, player.duration);
      console.log('🎵 Media Session: seek forward', skipTime);
      seek(newTime);
    });

    // Seek to specific position
    navigator.mediaSession.setActionHandler('seekto', (details) => {
      if (details.seekTime !== undefined && details.seekTime !== null) {
        console.log('🎵 Media Session: seek to', details.seekTime);
        seek(details.seekTime);
      }
    });

    // Stop (not always supported, but good to have)
    try {
      navigator.mediaSession.setActionHandler('stop', () => {
        console.log('🎵 Media Session: stop');
        if (player.isPlaying) {
          togglePlay();
        }
      });
    } catch {
      // Some browsers don't support stop action
    }

    // Cleanup
    return () => {
      try {
        navigator.mediaSession.setActionHandler('play', null);
        navigator.mediaSession.setActionHandler('pause', null);
        navigator.mediaSession.setActionHandler('previoustrack', null);
        navigator.mediaSession.setActionHandler('nexttrack', null);
        navigator.mediaSession.setActionHandler('seekbackward', null);
        navigator.mediaSession.setActionHandler('seekforward', null);
        navigator.mediaSession.setActionHandler('seekto', null);
        navigator.mediaSession.setActionHandler('stop', null);
      } catch {
        // Ignore cleanup errors
      }
    };
  }, [togglePlay, nextSong, previousSong, seek, player.isPlaying, player.currentTime, player.duration]);

  // Actualizar metadata cuando cambia la canción
  useEffect(() => {
    updateMediaMetadata();
  }, [updateMediaMetadata]);

  // Actualizar estado de reproducción
  useEffect(() => {
    updatePlaybackState();
  }, [updatePlaybackState]);

  // Actualizar posición periódicamente (cada 1 segundo cuando está reproduciendo)
  useEffect(() => {
    if (!player.isPlaying) return;

    // Actualizar inmediatamente
    updatePositionState();

    // Y luego cada segundo
    const interval = setInterval(updatePositionState, 1000);

    return () => clearInterval(interval);
  }, [player.isPlaying, updatePositionState]);

  return null;
};

export default useMediaSession;
