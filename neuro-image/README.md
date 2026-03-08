# Tachikoma Image Service

Microservicio de galería y generación de imágenes con IA para TACHIKOMA-OS.

## Puerto

- **3011** (configurable via `PORT`)

## Características

- ✅ Galería de imágenes
- ✅ Álbumes para organizar imágenes
- ✅ Imágenes favoritas
- ✅ Etiquetas (tags)
- ✅ Soporte para imágenes generadas por IA
- ✅ Soporte para imágenes subidas
- ✅ Estilos predefinidos para generación
- ✅ Metadatos de generación (prompt, seed, steps, etc.)

## API Endpoints

### Imágenes

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/images` | Listar imágenes (filtros: album_id, source, tag, favorite) |
| POST | `/api/images` | Subir imagen |
| POST | `/api/images/generate` | Generar imagen con IA |
| GET | `/api/images/styles` | Listar estilos disponibles |
| GET | `/api/images/:id` | Obtener imagen |
| PUT | `/api/images/:id` | Actualizar imagen |
| DELETE | `/api/images/:id` | Eliminar imagen |

### Álbumes

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/images/albums` | Listar álbumes |
| POST | `/api/images/albums` | Crear álbum |
| GET | `/api/images/albums/:id` | Obtener álbum |
| PUT | `/api/images/albums/:id` | Actualizar álbum |
| DELETE | `/api/images/albums/:id` | Eliminar álbum |

## Modelos

### Image

```json
{
  "id": "uuid",
  "name": "string",
  "prompt": "string | null",
  "negative_prompt": "string | null",
  "source": "generated | uploaded | external",
  "url": "string | null",
  "thumbnail_url": "string | null",
  "base64_data": "string | null",
  "width": 512,
  "height": 512,
  "format": "png",
  "size_bytes": 123456,
  "album_id": "uuid | null",
  "tags": ["tag1", "tag2"],
  "favorite": false,
  "model": "stable-diffusion",
  "seed": 12345,
  "steps": 20,
  "cfg_scale": 7.0,
  "created_at": "2024-01-15T10:00:00Z"
}
```

### Album

```json
{
  "id": "uuid",
  "name": "string",
  "description": "string | null",
  "cover_image_id": "uuid | null",
  "color": "#6366f1 | null",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

### Ejemplo de Generar Imagen

```bash
curl -X POST http://localhost:3011/api/images/generate \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "A beautiful sunset over mountains",
    "negative_prompt": "blurry, low quality",
    "width": 512,
    "height": 512,
    "steps": 20,
    "cfg_scale": 7.0
  }'
```

## Estilos Predefinidos

| Estilo | Descripción |
|--------|-------------|
| realistic | Fotorrealista, 8k, detallado |
| anime | Estilo anime, colores vibrantes |
| oil_painting | Pintura al óleo, clásico |
| watercolor | Acuarela, suave |
| digital_art | Arte digital, concept art |
| sketch | Boceto a lápiz, blanco y negro |
| cyberpunk | Cyberpunk, neón, futurista |
| fantasy | Arte fantástico, mágico |

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| PORT | Puerto del servicio | 3011 |
| BACKEND_URL | URL del backend | http://localhost:3000 |
| OLLAMA_URL | URL de Ollama | http://localhost:11434 |
| RUST_LOG | Nivel de logging | info |

## Desarrollo

```bash
# Ejecutar localmente
cargo run

# Con hot-reload
cargo watch -x run

# Build para producción
cargo build --release
```

## Docker

```bash
# Build de desarrollo (rápido)
docker build -f Dockerfile.dev -t tachikoma-image:dev .

# Build de producción
docker build -t tachikoma-image .

# Ejecutar
docker run -p 3011:3011 tachikoma-image
```
