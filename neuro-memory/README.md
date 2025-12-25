# NEURO-OS Memory Service

Microservicio para gestión de memorias y grafo de conocimiento.

## Puerto

- **3004** (por defecto)

## Endpoints

### Health
- `GET /api/health` - Estado del servicio

### Memories
- `GET /api/memories` - Listar memorias
- `POST /api/memories` - Crear memoria
- `GET /api/memories/:id` - Obtener memoria
- `PATCH /api/memories/:id` - Actualizar memoria
- `DELETE /api/memories/:id` - Eliminar memoria
- `POST /api/memories/search` - Búsqueda semántica

### Relations
- `GET /api/memories/:id/relations` - Relaciones de una memoria
- `GET /api/memories/:id/related` - Memorias relacionadas
- `POST /api/memories/relations` - Crear relación
- `DELETE /api/memories/:from_id/relations/:to_id` - Eliminar relación

### Graph Admin
- `GET /api/admin/graph/stats` - Estadísticas del grafo
- `GET /api/admin/graph/export` - Exportar grafo completo
- `GET /api/admin/graph/events` - SSE para eventos del grafo

## Variables de Entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| HOST | 0.0.0.0 | Host de escucha |
| PORT | 3004 | Puerto |
| DATABASE_URL | 127.0.0.1:8000 | URL de SurrealDB |
| DATABASE_USER | root | Usuario de DB |
| DATABASE_PASS | root | Contraseña de DB |
| DATABASE_NS | neuro | Namespace |
| DATABASE_DB | memories | Database |
| OLLAMA_URL | http://localhost:11434 | URL de Ollama para embeddings |

## Desarrollo

```bash
# Ejecutar localmente
cargo run

# Con variables de entorno
DATABASE_URL=127.0.0.1:8000 cargo run
```

## Docker

```bash
# Build production
docker build -t neuro-memory .

# Build development (fast)
docker build -f Dockerfile.dev -t neuro-memory:dev .
```
