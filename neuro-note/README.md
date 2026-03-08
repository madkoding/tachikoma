# Tachikoma Note Service

Microservicio de notas para TACHIKOMA-OS con soporte para carpetas, etiquetas y búsqueda.

## Puerto

- **3007** (configurable via `PORT`)

## Características

- ✅ Notas con título y contenido markdown
- ✅ Organización en carpetas
- ✅ Etiquetas (tags)
- ✅ Colores personalizados
- ✅ Fijar notas importantes (pin)
- ✅ Archivar notas
- ✅ Búsqueda de texto completo

## API Endpoints

### Notas

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/notes` | Listar notas (filtros: folder_id, tag, archived, pinned) |
| POST | `/api/notes` | Crear nota |
| GET | `/api/notes/search?q=texto` | Buscar notas |
| GET | `/api/notes/:id` | Obtener nota |
| PUT | `/api/notes/:id` | Actualizar nota |
| DELETE | `/api/notes/:id` | Eliminar nota |

### Carpetas

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/notes/folders` | Listar carpetas |
| POST | `/api/notes/folders` | Crear carpeta |
| GET | `/api/notes/folders/:id` | Obtener carpeta con notas |
| PUT | `/api/notes/folders/:id` | Actualizar carpeta |
| DELETE | `/api/notes/folders/:id` | Eliminar carpeta |

## Modelos

### Note

```json
{
  "id": "uuid",
  "title": "string",
  "content": "markdown string",
  "folder_id": "uuid | null",
  "tags": ["tag1", "tag2"],
  "color": "#fef08a | null",
  "pinned": false,
  "archived": false,
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

### Folder

```json
{
  "id": "uuid",
  "name": "string",
  "parent_id": "uuid | null",
  "color": "#6366f1 | null",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

### Ejemplo de Crear Nota

```bash
curl -X POST http://localhost:3007/api/notes \
  -H "Content-Type: application/json" \
  -d '{
    "title": "My First Note",
    "content": "# Hello World\n\nThis is my first note.",
    "tags": ["personal", "important"],
    "color": "#fef08a"
  }'
```

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| PORT | Puerto del servicio | 3007 |
| BACKEND_URL | URL del backend | http://localhost:3000 |
| VOICE_SERVICE_URL | URL del servicio de voz | http://localhost:8100 |
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
docker build -f Dockerfile.dev -t tachikoma-note:dev .

# Build de producción
docker build -t tachikoma-note .

# Ejecutar
docker run -p 3007:3007 tachikoma-note
```
