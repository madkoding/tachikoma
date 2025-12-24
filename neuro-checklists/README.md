# Neuro-Checklists

Microservicio independiente para gestión de checklists.

## Arquitectura

- **Puerto**: 3001
- **Base de datos**: SurrealDB (namespace: `neuro`, database: `checklists`)
- **Framework**: Axum (Rust)

## Endpoints

### Health
- `GET /health` - Estado del servicio

### Checklists
- `GET /api/checklists` - Listar checklists
- `POST /api/checklists` - Crear checklist
- `GET /api/checklists/:id` - Obtener checklist con items
- `PATCH /api/checklists/:id` - Actualizar checklist
- `DELETE /api/checklists/:id` - Eliminar checklist
- `POST /api/checklists/import` - Importar desde markdown

### Items
- `POST /api/checklists/:id/items` - Agregar item
- `PATCH /api/checklists/:checklist_id/items/:item_id` - Actualizar item
- `DELETE /api/checklists/:checklist_id/items/:item_id` - Eliminar item
- `POST /api/checklists/:checklist_id/items/:item_id/toggle` - Toggle completado

## Variables de entorno

```env
PORT=3001
DATABASE_URL=ws://127.0.0.1:8000
DATABASE_USER=root
DATABASE_PASS=root
DATABASE_NAMESPACE=neuro
DATABASE_NAME=checklists
```

## Desarrollo

```bash
cargo run
```

## Docker

```bash
docker build -t neuro-checklists .
docker run -p 3001:3001 neuro-checklists
```
