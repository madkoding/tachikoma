# Neuro Docs Service

Microservicio de gestión de documentos para NEURO-OS.

## Puerto

- **3008** (configurable via `PORT`)

## Características

- ✅ Documentos con diferentes tipos (text, markdown, code, spreadsheet, pdf, etc.)
- ✅ Organización en carpetas
- ✅ Etiquetas (tags)
- ✅ Documentos destacados (starred)
- ✅ Documentos compartidos
- ✅ Búsqueda de texto completo
- ✅ Estadísticas de almacenamiento
- ✅ Detección automática de tipo de archivo

## API Endpoints

### Documentos

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/docs` | Listar documentos (filtros: folder_id, doc_type, tag, starred) |
| POST | `/api/docs` | Crear documento |
| GET | `/api/docs/search?q=texto` | Buscar documentos |
| GET | `/api/docs/stats` | Estadísticas de almacenamiento |
| GET | `/api/docs/:id` | Obtener documento |
| PUT | `/api/docs/:id` | Actualizar documento |
| DELETE | `/api/docs/:id` | Eliminar documento |

### Carpetas

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/docs/folders` | Listar carpetas |
| POST | `/api/docs/folders` | Crear carpeta |
| GET | `/api/docs/folders/:id` | Obtener carpeta con contenidos |
| PUT | `/api/docs/folders/:id` | Actualizar carpeta |
| DELETE | `/api/docs/folders/:id` | Eliminar carpeta |

## Modelos

### Document

```json
{
  "id": "uuid",
  "name": "string",
  "doc_type": "text | markdown | code | spreadsheet | presentation | pdf | other",
  "content": "string",
  "folder_id": "uuid | null",
  "tags": ["tag1", "tag2"],
  "size_bytes": 1234,
  "mime_type": "text/plain | null",
  "starred": false,
  "shared": false,
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z",
  "last_accessed_at": "2024-01-15T10:00:00Z"
}
```

### DocFolder

```json
{
  "id": "uuid",
  "name": "string",
  "parent_id": "uuid | null",
  "color": "#3b82f6 | null",
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

### StorageStats

```json
{
  "total_documents": 42,
  "total_folders": 5,
  "total_size_bytes": 1234567,
  "by_type": {
    "text": 10,
    "markdown": 15,
    "code": 12,
    "pdf": 5
  }
}
```

### Ejemplo de Crear Documento

```bash
curl -X POST http://localhost:3008/api/docs \
  -H "Content-Type: application/json" \
  -d '{
    "name": "notes.md",
    "content": "# My Notes\n\nSome content here.",
    "tags": ["work", "important"]
  }'
```

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| PORT | Puerto del servicio | 3008 |
| BACKEND_URL | URL del backend | http://localhost:3000 |
| OLLAMA_URL | URL de Ollama (para IA) | http://localhost:11434 |
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
docker build -f Dockerfile.dev -t neuro-docs:dev .

# Build de producción
docker build -t neuro-docs .

# Ejecutar
docker run -p 3008:3008 neuro-docs
```
