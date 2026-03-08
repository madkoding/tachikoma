# Tachikoma Calendar Service

Microservicio de calendario con eventos y recordatorios para TACHIKOMA-OS.

## Puerto

- **3009** (configurable via `PORT`)

## Características

- ✅ Eventos de calendario con fechas/horas
- ✅ Eventos de día completo
- ✅ Tipos de evento: evento, tarea, recordatorio, cumpleaños, feriado
- ✅ Recurrencia: diario, semanal, mensual, anual
- ✅ Recordatorios con notificaciones
- ✅ Colores personalizados
- ✅ Ubicación de eventos

## API Endpoints

### Eventos

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/calendar/events` | Listar eventos (filtros: start, end, event_type) |
| GET | `/api/calendar/events/today` | Eventos de hoy |
| POST | `/api/calendar/events` | Crear evento |
| GET | `/api/calendar/events/:id` | Obtener evento |
| PUT | `/api/calendar/events/:id` | Actualizar evento |
| DELETE | `/api/calendar/events/:id` | Eliminar evento |

### Recordatorios

| Método | Endpoint | Descripción |
|--------|----------|-------------|
| GET | `/api/calendar/reminders` | Listar recordatorios pendientes |
| POST | `/api/calendar/reminders/:id/dismiss` | Descartar recordatorio |

## Modelos

### CalendarEvent

```json
{
  "id": "uuid",
  "title": "string",
  "description": "string | null",
  "start_time": "2024-01-15T10:00:00Z",
  "end_time": "2024-01-15T11:00:00Z | null",
  "all_day": false,
  "event_type": "event | task | reminder | birthday | holiday",
  "color": "#3b82f6 | null",
  "location": "string | null",
  "recurrence": "none | daily | weekly | monthly | yearly",
  "reminder_minutes": 15,
  "completed": false,
  "created_at": "2024-01-15T10:00:00Z",
  "updated_at": "2024-01-15T10:00:00Z"
}
```

### Ejemplo de Crear Evento

```bash
curl -X POST http://localhost:3009/api/calendar/events \
  -H "Content-Type: application/json" \
  -d '{
    "title": "Meeting with team",
    "start_time": "2024-01-15T14:00:00Z",
    "end_time": "2024-01-15T15:00:00Z",
    "event_type": "event",
    "color": "#3b82f6",
    "reminder_minutes": 15
  }'
```

## Variables de Entorno

| Variable | Descripción | Default |
|----------|-------------|---------|
| PORT | Puerto del servicio | 3009 |
| BACKEND_URL | URL del backend | http://localhost:3000 |
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
docker build -f Dockerfile.dev -t tachikoma-calendar:dev .

# Build de producción
docker build -t tachikoma-calendar .

# Ejecutar
docker run -p 3009:3009 tachikoma-calendar
```
