# Tachikoma-Pomodoro 🍅

Microservicio de timer Pomodoro para productividad en TACHIKOMA-OS.

## Características

- ⏱️ Timer Pomodoro configurable (trabajo, descanso corto, descanso largo)
- 📊 Estadísticas diarias y históricas
- ⚙️ Configuración personalizable
- 🔔 Notificaciones de finalización
- 💾 Persistencia vía tachikoma-backend

## API Endpoints

### Timer State
- `GET /api/pomodoro/state` - Estado actual del timer + estadísticas del día

### Sessions
- `POST /api/pomodoro/sessions` - Iniciar nueva sesión
- `GET /api/pomodoro/sessions` - Listar sesiones de hoy
- `PATCH /api/pomodoro/sessions/:id` - Actualizar sesión (tiempo transcurrido)
- `POST /api/pomodoro/sessions/:id/complete` - Marcar como completada
- `POST /api/pomodoro/sessions/:id/cancel` - Cancelar sesión
- `POST /api/pomodoro/sessions/:id/pause` - Pausar sesión
- `POST /api/pomodoro/sessions/:id/resume` - Reanudar sesión

### Settings
- `GET /api/pomodoro/settings` - Obtener configuración
- `POST /api/pomodoro/settings` - Guardar configuración

### Stats
- `GET /api/pomodoro/stats?start=YYYY-MM-DD&end=YYYY-MM-DD` - Estadísticas por rango

## Configuración por Defecto

| Setting | Valor |
|---------|-------|
| Trabajo | 25 min |
| Descanso corto | 5 min |
| Descanso largo | 15 min |
| Pomodoros antes de descanso largo | 4 |

## Variables de Entorno

| Variable | Default | Descripción |
|----------|---------|-------------|
| PORT | 3010 | Puerto del servicio |
| BACKEND_URL | http://localhost:3000 | URL del backend |
| DEFAULT_WORK_MINUTES | 25 | Duración trabajo |
| DEFAULT_SHORT_BREAK_MINUTES | 5 | Duración descanso corto |
| DEFAULT_LONG_BREAK_MINUTES | 15 | Duración descanso largo |
| POMODOROS_BEFORE_LONG_BREAK | 4 | Pomodoros antes de descanso largo |

## Desarrollo

```bash
# Desarrollo local
cargo run

# Con hot reload
cargo watch -x run

# Build de producción
cargo build --release
```

## Docker

```bash
# Build
docker build -t tachikoma-pomodoro .

# Run
docker run -p 3010:3010 -e BACKEND_URL=http://host.docker.internal:3000 tachikoma-pomodoro
```
