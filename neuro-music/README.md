# Tachikoma-Music Microservice

Servicio de streaming de música con integración de YouTube, ecualizador de 16 bandas y gestión de playlists.

## Características

- 🎵 Streaming de audio desde YouTube
- 📋 Gestión de playlists y canciones
- 🎛️ Ecualizador gráfico de 16 bandas
- 📊 Analizador de espectro
- 🖼️ Búsqueda de carátulas (MusicBrainz/Cover Art Archive)
- 🔊 Mejora de audio (normalización, compresión)
- 📈 Historial y estadísticas de reproducción

## Dependencias del Sistema

```bash
# Ubuntu/Debian
apt install ffmpeg

# yt-dlp
pip install yt-dlp
```

## Variables de Entorno

```bash
PORT=3002
DATABASE_URL=ws://127.0.0.1:8000
DATABASE_USER=root
DATABASE_PASS=root
DATABASE_NAMESPACE=tachikoma
DATABASE_NAME=music
YTDLP_PATH=yt-dlp
FFMPEG_PATH=ffmpeg
```

## API Endpoints

### Playlists
- `GET /api/music/playlists` - Listar playlists
- `POST /api/music/playlists` - Crear playlist
- `GET /api/music/playlists/:id` - Obtener playlist con canciones
- `PATCH /api/music/playlists/:id` - Actualizar playlist
- `DELETE /api/music/playlists/:id` - Eliminar playlist

### Canciones
- `POST /api/music/playlists/:id/songs` - Agregar canción (por URL de YouTube)
- `PATCH /api/music/playlists/:playlist_id/songs/:song_id` - Actualizar canción
- `DELETE /api/music/playlists/:playlist_id/songs/:song_id` - Eliminar canción
- `POST /api/music/playlists/:id/reorder` - Reordenar canciones

### Streaming
- `GET /api/music/stream/:song_id` - Stream de audio (OGG/Opus)
- `GET /api/music/stream/:song_id/info` - Info del stream

### YouTube
- `GET /api/music/youtube/search?q=query` - Buscar en YouTube
- `GET /api/music/youtube/metadata?url=...` - Obtener metadatos

### Ecualizador
- `GET /api/music/equalizer` - Obtener configuración
- `PUT /api/music/equalizer` - Guardar configuración
- `GET /api/music/equalizer/preset?name=rock` - Obtener preset

### Carátulas
- `GET /api/music/covers/search?title=...&artist=...` - Buscar carátula

### Historial
- `GET /api/music/history` - Historial de reproducción
- `GET /api/music/stats/most-played` - Canciones más reproducidas

## Presets de Ecualizador

- `flat` - Respuesta plana
- `bass_boost` - Realce de bajos
- `treble_boost` - Realce de agudos
- `vocal` - Voces destacadas
- `rock` - Sonido rock
- `electronic` - Música electrónica
- `acoustic` - Acústico

## Desarrollo

```bash
cd tachikoma-music
cargo watch -x run
```
