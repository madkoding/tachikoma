# NEURO-OS Chat Service

Microservicio para chat con LLM, streaming SSE y gestión de conversaciones.

## Puerto

- **3003** (por defecto)

## Endpoints

### Health & System
- `GET /api/health` - Estado del servicio
- `GET /api/models` - Listar modelos disponibles

### Chat
- `POST /api/chat` - Enviar mensaje (respuesta completa)
- `POST /api/chat/stream` - Enviar mensaje (streaming SSE)

### Conversations
- `GET /api/chat/conversations` - Listar conversaciones
- `GET /api/chat/conversations/:id` - Obtener conversación con mensajes
- `DELETE /api/chat/conversations/:id` - Eliminar conversación

## Variables de Entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| HOST | 0.0.0.0 | Host de escucha |
| PORT | 3003 | Puerto |
| DATABASE_URL | 127.0.0.1:8000 | URL de SurrealDB |
| DATABASE_USER | root | Usuario de DB |
| DATABASE_PASS | root | Contraseña de DB |
| DATABASE_NS | neuro | Namespace |
| DATABASE_DB | chat | Database |
| OLLAMA_URL | http://localhost:11434 | URL de Ollama |
| MEMORY_SERVICE_URL | http://localhost:3004 | URL del servicio de memoria |
| DEFAULT_MODEL | qwen2.5-coder:7b | Modelo por defecto |
| FAST_MODEL | ministral-3:3b | Modelo rápido |

## Streaming SSE

El endpoint `/api/chat/stream` devuelve eventos SSE:

```
event: message
data: {"type": "start", "conversation_id": "uuid", "model": "model"}

event: message
data: {"type": "chunk", "content": "texto..."}

event: message
data: {"type": "done", "tokens_prompt": 100, "tokens_completion": 50}
```

## Desarrollo

```bash
cargo run
```

## Docker

```bash
docker build -t neuro-chat .
docker build -f Dockerfile.dev -t neuro-chat:dev .
```
