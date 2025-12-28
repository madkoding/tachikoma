# NEURO-OS Copilot Instructions

## Arquitectura del Proyecto

NEURO-OS es un ecosistema de IA modular que consiste en:

## ⚠️ CONVENCIÓN DE NOMBRES ESTANDARIZADA

**Todos los nombres deben seguir el patrón `neuro-{servicio}`** para mantener consistencia:

| Elemento | Patrón | Ejemplo |
|----------|--------|---------|
| Carpeta | `neuro-{servicio}` | `neuro-voice/` |
| Cargo.toml `name` | `neuro-{servicio}` | `name = "neuro-voice"` |
| Servicio docker-compose | `neuro-{servicio}` | `neuro-voice:` |
| `container_name` | `neuro-{servicio}` | `container_name: neuro-voice` |
| Variables de entorno | `{SERVICIO}_SERVICE_URL` | `VOICE_SERVICE_URL` |

**Ejemplo correcto de servicio en docker-compose.yml:**
```yaml
neuro-voice:                    # ✅ Nombre del servicio = neuro-voice
  build:
    context: ./neuro-voice      # ✅ Carpeta = neuro-voice
    dockerfile: Dockerfile
  container_name: neuro-voice   # ✅ container_name = neuro-voice
```

### Servicios Core
- **neuro-backend**: API Gateway central + LLM Gateway en Rust/Axum (puerto 3000)
- **neuro-ui**: Interfaz de usuario en React/Vite (puerto 5173)
- **neuro-admin**: Panel de administración en React/Vite (puerto 5174)

### Microservicios Existentes
| Servicio | Puerto | Descripción |
|----------|--------|-------------|
| neuro-voice | 8100 | Síntesis de voz con Piper TTS |
| neuro-checklists | 3001 | Gestión de checklists |
| neuro-music | 3002 | Streaming de música YouTube |
| neuro-chat | 3003 | Conversaciones con LLM (via backend) |
| neuro-memory | 3004 | Memoria semántica GraphRAG (embeddings via backend) |
| neuro-agent | 3005 | Herramientas de agente IA |

### Microservicios Planeados
| Servicio | Puerto | Descripción |
|----------|--------|-------------|
| neuro-kanban | 3006 | Tableros Kanban |
| neuro-note | 3007 | Notas + transcripción de voz con IA |
| neuro-docs | 3008 | Documentos con IA (DOCX, XLSX, PPTX, embeddings via backend) |
| neuro-calendar | 3009 | Calendario + recordatorios |
| neuro-pomodoro | 3010 | Timer Pomodoro |
| neuro-image | 3011 | Galería de imágenes generadas por IA (via backend) |

### Servicios de Infraestructura (Docker)
| Servicio | Puerto | Descripción |
|----------|--------|-------------|
| SurrealDB | 8000 | Base de datos Graph + Vector |
| Searxng | 8080 | Motor de búsqueda privado |

### Servicios Externos (neuro-ollama)
| Servicio | Puerto | Descripción |
|----------|--------|-------------|
| Ollama | 11434 | Inferencia LLM local (proyecto independiente) |

## ⚠️ IMPORTANTE: LLM Gateway Pattern

**Todas las operaciones LLM deben pasar por neuro-backend.** Los microservicios NO deben conectarse directamente a Ollama.

### Endpoints LLM en backend (`/api/llm/*`)
| Endpoint | Método | Descripción |
|----------|--------|-------------|
| `/api/llm/health` | GET | Estado de Ollama y modelos disponibles |
| `/api/llm/embed` | POST | Generar embedding para un texto |
| `/api/llm/embed/batch` | POST | Generar embeddings para múltiples textos |
| `/api/llm/chat` | POST | Chat completo (no streaming) |
| `/api/llm/chat/stream` | POST | Chat con streaming SSE |
| `/api/llm/speculative/stream` | POST | Speculative decoding con streaming SSE |
| `/api/llm/generate` | POST | Generación de tokens raw |

### Model Tiers (configurados en backend)
| Tier | Modelo | Uso |
|------|--------|-----|
| Light | ministral-3:3b | Draft model para speculative decoding, respuestas rápidas |
| Standard | ministral-3:8b | Target model, uso general |
| Heavy | ministral-3:8b | Mismo que Standard (no hay modelo más pesado por ahora) |
| Embedding | nomic-embed-text | Embeddings vectoriales |

### Ejemplo: Microservicio usando backend como LLM gateway

```rust
// En el microservicio (ej: neuro-chat)
pub struct BackendLlmClient {
    client: reqwest::Client,
    base_url: String,  // BACKEND_URL, no OLLAMA_URL
}

impl BackendLlmClient {
    pub async fn chat(&self, messages: Vec<ChatMessage>, model: Option<&str>) -> Result<ChatResponse, String> {
        let url = format!("{}/api/llm/chat", self.base_url);
        let body = json!({ "messages": messages, "model": model });
        // ...
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>, String> {
        let url = format!("{}/api/llm/embed", self.base_url);
        // ...
    }
}
```

## Patrón de Proxy para Microservicios

### ⚠️ IMPORTANTE: El router de Axum con `.nest()` modifica el path

Cuando usas `.nest("/api", api_routes)` en Axum, el prefijo `/api` es **removido** del path antes de llegar al handler.

**Ejemplo:**
- Request entrante: `GET /api/music/playlists`
- Path en el handler: `/music/playlists` (sin `/api`)

### Cómo implementar un proxy a microservicio:

```rust
// En routes.rs - registrar las rutas del proxy
.route("/music", any(handlers::proxy_music))
.route("/music/*path", any(handlers::proxy_music))

// En proxy.rs - construir la URL correctamente
let path = request.uri().path();  // Esto será "/music/playlists"
// IMPORTANTE: Agregar /api de vuelta para el microservicio
let target_url = format!("{}/api{}{}", service_url, path, query);
// Resultado: "http://localhost:3002/api/music/playlists"
```

### Checklist para agregar un nuevo microservicio:

1. **Crear estructura del microservicio:**
   ```
   neuro-newservice/
   ├── Cargo.toml
   ├── Dockerfile
   ├── Dockerfile.dev
   ├── README.md
   └── src/
       ├── main.rs
       ├── config.rs
       ├── routes.rs
       ├── handlers.rs
       ├── models.rs
       └── backend_client.rs  # Si necesita acceso a datos
   ```

2. **Agregar configuración en `config.rs` del backend:**
   ```rust
   pub struct MicroservicesConfig {
       pub new_service_url: String,
   }
   ```

3. **Agregar variable de entorno:**
   - En `tasks.json`: `NEW_SERVICE_URL=http://127.0.0.1:PORT`
   - En `dev.sh`: agregar rebuild-newservice
   - En `docker-compose.yml`: definir el servicio
   - En `docker-compose.dev.yml`: agregar volumes de cache

4. **Crear handler en `handlers/proxy.rs`:**
   ```rust
   pub async fn proxy_new_service(
       State(state): State<Arc<AppState>>,
       request: Request,
   ) -> Result<Response, StatusCode> {
       debug!("Proxying new_service request: {}", request.uri().path());
       proxy_to_service(
           &state.microservices_config.new_service_url,
           "new_service",
           request,
       ).await
   }
   ```

4. **Registrar rutas en `routes.rs`:**
   ```rust
   .route("/new_service", any(handlers::proxy_new_service))
   .route("/new_service/*path", any(handlers::proxy_new_service))
   ```

5. **El microservicio debe escuchar en `/api/...`:**
   - Las rutas del microservicio deben incluir el prefijo `/api`
   - Ejemplo: `/api/new_service/endpoint`

## Debugging del Backend

### Verificar qué binario está corriendo:

```bash
# Ver fecha del binario
ls -la neuro-backend/target/debug/neuro-backend

# Ver proceso y hora de inicio
ps aux | grep neuro-backend

# Verificar build info via health endpoint
curl http://localhost:3000/api/health | jq '.build_info'
```

### El health endpoint incluye:
- `git_hash`: Hash corto del commit
- `build_time`: Timestamp de compilación
- `rust_version`: Versión de Rust usada

Esto ayuda a identificar si el binario en ejecución corresponde al código actual.

### Forzar recompilación:

```bash
# Limpiar fingerprint y recompilar
rm -rf target/debug/.fingerprint/neuro-backend*
cargo build

# O usar la tarea de VS Code
# "🔨 Rebuild Backend (Clean)"
```

## Estructura de Tareas de VS Code

- **🐳 Docker Services**: Levanta SurrealDB, Ollama, Searxng, Voice, Music
- **🦀 Backend (Rust)**: Ejecuta neuro-backend con cargo watch
- **⚛️ User UI (Vite)**: Inicia neuro-ui
- **🔧 Admin UI (Vite)**: Inicia neuro-admin
- **🔨 Rebuild Backend (Clean)**: Limpia cache y recompila desde cero
- **🎵 Rebuild Music Service**: Reconstruye el contenedor de música

### ⚠️ IMPORTANTE: NO iniciar servicios manualmente por terminal

**NUNCA** ejecutes comandos como:
```bash
# ❌ INCORRECTO - Esto interfiere con los tasks de VS Code
cargo run
./target/debug/neuro-backend
cargo watch -x run
```

**SIEMPRE** usa los tasks de VS Code:
- Si el task "🦀 Backend (after Docker)" ya está corriendo, cargo watch detectará los cambios automáticamente
- Si necesitas reiniciar, usa el task "🔨 Rebuild Backend (Clean)"
- Para ver logs, usa la terminal del task en VS Code

Si ejecutas comandos manuales:
1. Pisas el proceso del task
2. Los logs se pierden en múltiples terminales
3. El cargo watch deja de funcionar
4. Puedes tener múltiples instancias compitiendo por el mismo puerto

## Desarrollo Rápido de Microservicios Docker

### ⚡ FAST DEV MODE (3-5x más rápido)

Para desarrollo, usa `docker-compose.dev.yml` que proporciona:

| Optimización | Beneficio |
|--------------|-----------|
| Debug builds (sin `--release`) | ~30% más rápido |
| mold linker | 2-3x más rápido en linking |
| Cargo cache persistente | Dependencias compilan UNA vez |

### Comandos de desarrollo rápido:

```bash
# Iniciar todos los servicios Docker en modo dev
./dev.sh docker-dev

# Reconstruir un servicio específico (rápido)
./dev.sh rebuild-voice
./dev.sh rebuild-music
./dev.sh rebuild-checklists

# Limpiar cache si hay problemas
./dev.sh clean-cache
```

### Diferencia release vs debug:

| Modo | Velocidad Build | Rendimiento Runtime | Uso |
|------|-----------------|--------------------|----|
| `debug` | Rápido (~30s-1min) | Más lento | **Desarrollo** |
| `--release` | Lento (~3-5min) | Óptimo | **Producción** |

Para producción, usar los Dockerfile normales (sin `.dev`).

## Patrones de Código

### Servicios Rust/Axum (microservicios):

```rust
// Rutas siempre bajo /api
let app = Router::new()
    .route("/api/service/endpoint", get(handler))
    .with_state(state);
```

### Frontend (React/Vite):

```typescript
// El proxy de Vite redirige /api/* a localhost:3000
const response = await fetch('/api/music/playlists');
```

## Variables de Entorno Requeridas

### neuro-backend:
- `DATABASE_URL`: WebSocket URL de SurrealDB
- `DATABASE_USER`: Usuario de SurrealDB
- `DATABASE_PASS`: Contraseña de SurrealDB
- `OLLAMA_URL`: URL del servidor Ollama
- `SEARXNG_URL`: URL del servidor Searxng
- `VOICE_SERVICE_URL`: URL del servicio de voz
- `MUSIC_SERVICE_URL`: URL del servicio de música
- `RUST_LOG`: Nivel de logging (debug, info, warn, error)

## Optimización de Builds con cargo-chef

Todos los microservicios Rust usan **cargo-chef** para acelerar builds sucesivos en Docker.

### Patrón de Dockerfile con cargo-chef:

```dockerfile
# Stage 1: Chef planner
FROM rust:1.83-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Prepare recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies (CACHED!)
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build app (solo recompila código que cambió)
COPY src ./src
RUN cargo build --release
```

### Beneficios:
- **Cache de dependencias**: Las dependencias se compilan una vez y se cachean
- **Builds incrementales**: Solo recompila tu código, no las dependencias
- **Compatible**: Funciona con docker compose antiguo (sin BuildKit)

### Al agregar un nuevo microservicio Rust:
1. Usar el patrón de 3 stages: chef → planner → builder
2. Instalar cargo-chef en el stage `chef`
3. Usar `cargo chef prepare` para generar recipe.json
4. Usar `cargo chef cook` para compilar dependencias

## SurrealDB v1.5.6: Configuración y Compatibilidad

### ⚠️ IMPORTANTE: Usar SurrealDB versión 1.5.6

El proyecto usa **SurrealDB v1.5.6** tanto en el servidor como en los clientes Rust. NO usar versiones 2.x ya que hay incompatibilidades de protocolo y sintaxis.

### 📋 Resumen de Consideraciones Críticas

| Problema | Solución |
|----------|----------|
| **FIELD id en SCHEMAFULL** | **NO definir** - SurrealDB maneja IDs automáticamente |
| Conexión WebSocket | Quitar prefijo `ws://` de la URL |
| Sintaxis `IF NOT EXISTS` | NO usar, no soportado en 1.5.x |
| `TYPE RELATION` | Usar tabla normal con campos `in/out` |
| Record IDs | Usar `Thing` para deserializar, convertir a `Uuid` |
| Valores opcionales vacíos | Usar `NONE`, NO `null` |
| Timestamps | Usar `time::now()` en SQL, NO pasar datetime de Rust |
| `.create().content()` con ID | NO incluir `id` en el struct content |
| `DELETE` no devuelve registro | Verificar existencia antes de eliminar |
| API builder vs SQL | Preferir SQL queries directas |

### Versiones requeridas:

| Componente | Versión | Notas |
|------------|---------|-------|
| SurrealDB Server (Docker) | `surrealdb/surrealdb:v1.5.6` | En docker-compose.yml |
| Cliente Rust | `surrealdb = "1.5"` | En Cargo.toml |
| Storage Engine | `file:/data/neuro.db` | NO usar `surrealkv` |

### Configuración en docker-compose.yml:

```yaml
surrealdb:
  image: surrealdb/surrealdb:v1.5.6  # ⚠️ Fijar versión, NO usar :latest
  command: start --user root --pass secret --log info file:/data/neuro.db
```

### Configuración en Cargo.toml:

```toml
[dependencies]
surrealdb = { version = "1.5", features = ["kv-mem"] }
```

### Conexión al WebSocket - Quitar prefijo ws://

El cliente SurrealDB 1.5.x espera solo `host:port`, sin el esquema `ws://`:

```rust
pub async fn connect(config: &Config) -> Result<Self, Box<dyn std::error::Error>> {
    // ⚠️ IMPORTANTE: Quitar el prefijo ws:// de la URL
    let db_url = config.database_url
        .replace("ws://", "")
        .replace("wss://", "");
    
    tracing::info!("Connecting to database at: {}", db_url);
    
    let client = Surreal::new::<Ws>(&db_url).await?;
    // ...
}
```

### ⚠️ CRÍTICO: NO definir FIELD id en tablas SCHEMAFULL

SurrealDB maneja automáticamente el `id` de los records. Si defines `DEFINE FIELD id ON table TYPE string`, causarás un conflicto cuando intentes hacer UPDATE o CREATE:

```rust
// ❌ INCORRECTO - Esto causa "expected a string" error en UPDATE
let schema = r#"
    DEFINE TABLE conversation SCHEMAFULL;
    DEFINE FIELD id ON conversation TYPE string;  -- ❌ NO HACER ESTO
    DEFINE FIELD title ON conversation TYPE string;
"#;

// ✅ CORRECTO - Dejar que SurrealDB maneje el id automáticamente
let schema = r#"
    DEFINE TABLE conversation SCHEMAFULL;
    -- NO definir FIELD id - SurrealDB lo maneja automáticamente
    DEFINE FIELD title ON conversation TYPE option<string>;
    DEFINE FIELD created_at ON conversation TYPE datetime;
"#;
```

Si ya tienes este problema, elimina el campo con:
```sql
REMOVE FIELD id ON conversation;
REMOVE INDEX conversation_id_idx ON conversation;  -- Si existe
```

### Sintaxis de Schema - NO usar IF NOT EXISTS ni TYPE RELATION

SurrealDB 1.5.x NO soporta:
- `DEFINE TABLE IF NOT EXISTS` 
- `DEFINE FIELD IF NOT EXISTS`
- `DEFINE INDEX IF NOT EXISTS`
- `TYPE RELATION FROM table TO table`

```rust
// ❌ INCORRECTO - Sintaxis de SurrealDB 2.x
let schema = r#"
    DEFINE TABLE IF NOT EXISTS memory SCHEMAFULL;
    DEFINE TABLE related_to SCHEMAFULL TYPE RELATION FROM memory TO memory;
"#;

// ✅ CORRECTO - Sintaxis de SurrealDB 1.5.x
let schema = r#"
    DEFINE TABLE memory SCHEMAFULL;
    
    -- Para relaciones, usar tabla normal con campos in/out
    DEFINE TABLE related_to SCHEMAFULL;
    DEFINE FIELD in ON related_to TYPE record(memory);
    DEFINE FIELD out ON related_to TYPE record(memory);
"#;
```

### Manejo de Record IDs con Thing

SurrealDB devuelve los IDs como objetos `Thing`, no como strings o UUIDs:

```rust
use surrealdb::sql::Thing;

// Struct para deserializar desde DB
#[derive(Deserialize)]
struct PlaylistRecord {
    id: Thing,  // ⚠️ Usar Thing, no Uuid
    name: String,
}

// Struct para uso en la aplicación
struct Playlist {
    id: Uuid,
    name: String,
}

// Helper para convertir Thing a UUID
fn thing_to_uuid(thing: &Thing) -> Option<Uuid> {
    match &thing.id {
        surrealdb::sql::Id::String(s) => Uuid::parse_str(s).ok(),
        _ => None,
    }
}

impl From<PlaylistRecord> for Playlist {
    fn from(record: PlaylistRecord) -> Self {
        Playlist {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            name: record.name,
        }
    }
}
```

### Queries con Record IDs:

```rust
// ✅ CORRECTO - Usar sintaxis directa de record ID
let query = format!("SELECT * FROM playlist:`{}`", id);

// Para crear relaciones (graph edges):
let query = format!(
    "RELATE memory:`{}`->related_to->memory:`{}`",
    from_id, to_id
);
```

### ⚠️ Usar NONE en lugar de null para campos Option

SurrealDB 1.5.x NO acepta `null` para campos `option<T>`. Usar `NONE`:

```rust
// ❌ INCORRECTO - Causa error "expected option<datetime>"
let query = format!(r#"
    CREATE checklist:`{}` SET
        title = $title,
        last_reminded = null
"#, id);

// ✅ CORRECTO - Usar NONE para valores opcionales vacíos
let query = format!(r#"
    CREATE checklist:`{}` SET
        title = $title,
        last_reminded = NONE
"#, id);
```

### ⚠️ Usar time::now() para timestamps en SQL

NO pasar `chrono::Utc::now()` como parámetro. SurrealDB tiene problemas deserializando datetime de Rust. Usar `time::now()` en el SQL:

```rust
// ❌ INCORRECTO - Error de deserialización de datetime
let now = chrono::Utc::now();
let query = "CREATE record SET created_at = $now";
client.query(query).bind(("now", now)).await?;

// ✅ CORRECTO - Usar time::now() en SQL
let query = format!(r#"
    CREATE checklist:`{}` SET
        title = $title,
        created_at = time::now(),
        updated_at = time::now()
"#, id);
```

### ⚠️ NO pasar ID en content al usar .create()

Cuando usas `.create(("table", id)).content(data)`, NO incluyas el `id` en el struct de data:

```rust
// ❌ INCORRECTO - ID duplicado causa error
let checklist = Checklist { id, title, ... };
client.create(("checklist", id.to_string()))
    .content(&checklist)  // ❌ checklist tiene id
    .await?;

// ✅ CORRECTO - Usar SQL query sin ID en el contenido
let query = format!(r#"
    CREATE checklist:`{}` SET
        title = $title,
        priority = $priority
"#, id);
client.query(&query)
    .bind(("title", data.title))
    .bind(("priority", data.priority))
    .await?;
```

### ⚠️ DELETE no devuelve el registro eliminado

En SurrealDB 1.5.x, `DELETE` devuelve array vacío, no el registro eliminado. Verificar existencia antes:

```rust
// ❌ INCORRECTO - deleted siempre será None
let query = format!("DELETE checklist:`{}`", id);
let mut result = client.query(&query).await?;
let deleted: Option<ChecklistRecord> = result.take(0)?;
Ok(deleted.is_some())  // Siempre false!

// ✅ CORRECTO - Verificar existencia primero
pub async fn delete_checklist(&self, id: Uuid) -> Result<bool, Error> {
    // Primero verificar si existe
    let exists = self.get_checklist(id).await?;
    if exists.is_none() {
        return Ok(false);
    }

    // Luego eliminar
    let query = format!("DELETE checklist:`{}`", id);
    self.client.query(&query).await?;
    
    Ok(true)
}
```

### Patrón recomendado: SQL queries vs API builder

Preferir SQL queries directas sobre el API builder para evitar problemas de serialización:

```rust
// ❌ EVITAR - API builder tiene problemas con tipos complejos
let created: Option<Checklist> = client
    .create(("checklist", id.to_string()))
    .content(&data)
    .await?;

// ✅ PREFERIR - SQL queries con bindings
let query = format!(r#"
    CREATE checklist:`{}` SET
        title = $title,
        description = $description,
        priority = $priority,
        created_at = time::now(),
        updated_at = time::now()
"#, id);

let mut result = client.query(&query)
    .bind(("title", data.title))
    .bind(("description", data.description))
    .bind(("priority", data.priority.unwrap_or(3)))
    .await?;

let record: Option<ChecklistRecord> = result.take(0)?;
```

### Patrón completo: Structs Record para deserialización

Usar structs separados para DB (con Thing) y aplicación (con Uuid):

```rust
use surrealdb::sql::Thing;
use uuid::Uuid;

// ============================================
// Struct para deserializar desde SurrealDB
// ============================================
#[derive(Debug, Clone, Deserialize)]
pub struct ChecklistRecord {
    pub id: Thing,  // SurrealDB devuelve Thing
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// Struct para uso en la aplicación
// ============================================
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checklist {
    pub id: Uuid,  // Aplicación usa Uuid
    pub title: String,
    pub description: Option<String>,
    pub priority: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// ============================================
// Conversión Thing -> Uuid
// ============================================
fn thing_to_uuid(thing: &Thing) -> Option<Uuid> {
    match &thing.id {
        surrealdb::sql::Id::String(s) => Uuid::parse_str(s).ok(),
        _ => None,
    }
}

impl From<ChecklistRecord> for Checklist {
    fn from(record: ChecklistRecord) -> Self {
        Checklist {
            id: thing_to_uuid(&record.id).unwrap_or_default(),
            title: record.title,
            description: record.description,
            priority: record.priority,
            created_at: record.created_at,
            updated_at: record.updated_at,
        }
    }
}

// ============================================
// Uso en queries
// ============================================
pub async fn get_all(&self) -> Result<Vec<Checklist>, Error> {
    let query = "SELECT * FROM checklist ORDER BY created_at DESC";
    let mut result = self.client.query(query).await?;
    let records: Vec<ChecklistRecord> = result.take(0)?;
    Ok(records.into_iter().map(Checklist::from).collect())
}
```
