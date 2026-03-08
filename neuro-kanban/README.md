# tachikoma-kanban

Microservicio de tableros Kanban para TACHIKOMA-OS.

## Descripción

Servicio que proporciona funcionalidad completa de tableros Kanban con:
- **Boards**: Tableros que agrupan columnas
- **Columns**: Columnas con límites WIP opcionales
- **Cards**: Tarjetas con etiquetas, colores y fechas de vencimiento

## Puerto

- **3006** (configurable via `PORT`)

## API Endpoints

### Health
- `GET /api/health` - Estado del servicio

### Boards (Tableros)
- `GET /api/kanban/boards` - Listar tableros
- `POST /api/kanban/boards` - Crear tablero
- `GET /api/kanban/boards/:id` - Obtener tablero con columnas y tarjetas
- `PATCH /api/kanban/boards/:id` - Actualizar tablero
- `DELETE /api/kanban/boards/:id` - Eliminar tablero

### Columns (Columnas)
- `POST /api/kanban/boards/:board_id/columns` - Crear columna
- `PATCH /api/kanban/boards/:board_id/columns/:id` - Actualizar columna
- `DELETE /api/kanban/boards/:board_id/columns/:id` - Eliminar columna
- `PUT /api/kanban/boards/:board_id/columns/:id/reorder` - Reordenar columna

### Cards (Tarjetas)
- `POST /api/kanban/boards/:board_id/columns/:column_id/cards` - Crear tarjeta
- `PATCH /api/kanban/boards/:board_id/columns/:column_id/cards/:id` - Actualizar tarjeta
- `DELETE /api/kanban/boards/:board_id/columns/:column_id/cards/:id` - Eliminar tarjeta
- `PUT /api/kanban/boards/:board_id/columns/:column_id/cards/:id/move` - Mover tarjeta

## Modelos

### Board
```json
{
  "id": "uuid",
  "name": "Project Tasks",
  "description": "Main project board",
  "color": "#3b82f6",
  "is_archived": false,
  "columns": [...],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Column
```json
{
  "id": "uuid",
  "board_id": "uuid",
  "name": "In Progress",
  "color": "#f59e0b",
  "wip_limit": 5,
  "order": 1,
  "cards": [...],
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

### Card
```json
{
  "id": "uuid",
  "column_id": "uuid",
  "title": "Implement feature X",
  "description": "Details...",
  "color": "#ef4444",
  "labels": ["bug", "urgent"],
  "due_date": "2024-02-01T00:00:00Z",
  "order": 0,
  "created_at": "2024-01-01T00:00:00Z",
  "updated_at": "2024-01-01T00:00:00Z"
}
```

## Variables de Entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| `PORT` | 3006 | Puerto del servicio |
| `BACKEND_URL` | http://localhost:3000 | URL del backend principal |
| `RUST_LOG` | info | Nivel de logging |

## Desarrollo

```bash
# Ejecutar localmente
cd tachikoma-kanban
cargo run

# Con hot-reload
cargo watch -x run

# Build para producción
cargo build --release
```

## Docker

```bash
# Build producción
docker build -t tachikoma-kanban .

# Build desarrollo (más rápido)
docker build -f Dockerfile.dev -t tachikoma-kanban:dev .

# Ejecutar
docker run -p 3006:3006 -e BACKEND_URL=http://host.docker.internal:3000 tachikoma-kanban
```

## Características

- ✅ CRUD completo para boards, columns y cards
- ✅ Límites WIP (Work In Progress) por columna
- ✅ Drag & drop (mover tarjetas entre columnas)
- ✅ Etiquetas y colores personalizables
- ✅ Fechas de vencimiento
- ✅ Columnas por defecto (To Do, In Progress, Done)
- ✅ Estado en memoria (para demo, sin persistencia)
