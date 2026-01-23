# NEURO-OS 🧠

**NEURO-OS** is a modular AI ecosystem that combines a memory graph (GraphRAG), intelligent agents with tool capabilities, and automatic model selection based on available VRAM.

Available as:
- 🌐 **Web Application** (React/Vite)
- 🖥️ **Desktop Application** (Windows, Linux, macOS via Tauri)

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           NEURO-OS                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │  User UI    │  │  Admin UI   │  │   Z-Brain   │                  │
│  │  (React)    │  │  (React)    │  │   (CLI)     │                  │
│  │  :5173      │  │  :5174      │  │             │                  │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘                  │
│         │                │                │                          │
│         └────────────────┴────────────────┘                          │
│                          │                                           │
│              ┌───────────┴───────────┐                              │
│              │   API Gateway (Axum)  │                              │
│              │       :3000           │                              │
│              └───────────┬───────────┘                              │
│                          │                                           │
│  ┌───────────────────────┴───────────────────────┐                  │
│  │              Microservices Layer              │                  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐         │                  │
│  │  │  Chat   │ │ Memory  │ │  Agent  │         │                  │
│  │  │  :3003  │ │  :3004  │ │  :3005  │         │                  │
│  │  └─────────┘ └─────────┘ └─────────┘         │                  │
│  │  ┌─────────┐ ┌─────────┐ ┌─────────┐         │                  │
│  │  │Checklst │ │  Music  │ │  Voice  │         │                  │
│  │  │  :3001  │ │  :3002  │ │  :8100  │         │                  │
│  │  └─────────┘ └─────────┘ └─────────┘         │                  │
│  └───────────────────────────────────────────────┘                  │
│                          │                                           │
│  ┌───────────────────────┴───────────────────────┐                  │
│  │              Infrastructure Layer              │                  │
│  │  ┌──────────────┐ ┌──────────────┐ ┌────────┐ │                  │
│  │  │  SurrealDB   │ │   Ollama     │ │Searxng │ │                  │
│  │  │  :8000       │ │   :11434     │ │ :8080  │ │                  │
│  │  └──────────────┘ └──────────────┘ └────────┘ │                  │
│  └───────────────────────────────────────────────┘                  │
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

## Microservices

| Service | Port | Description |
|---------|------|-------------|
| neuro-backend | 3000 | Central API Gateway + LLM Gateway |
| neuro-checklists | 3001 | Checklist management |
| neuro-music | 3002 | YouTube music streaming |
| neuro-chat | 3003 | LLM conversations |
| neuro-memory | 3004 | GraphRAG semantic memory |
| neuro-agent | 3005 | AI agent tools |
| neuro-voice | 8100 | Piper TTS synthesis |

### External Services (neuro-ollama)

Ollama runs independently in the [neuro-ollama](https://github.com/madkoding/neuro-ollama) project:

| Service | Port | Description |
|---------|------|-------------|
| Ollama | 11434 | LLM inference server |

**Important**: All LLM operations (chat, embeddings, speculative decoding) go through neuro-backend's `/api/llm/*` endpoints. Microservices should NOT connect directly to Ollama.

### Planned Microservices

| Service | Port | Description |
|---------|------|-------------|
| neuro-kanban | 3006 | Kanban boards |
| neuro-note | 3007 | Notes + voice transcription |
| neuro-docs | 3008 | AI document generation (DOCX, XLSX, PPTX) |
| neuro-calendar | 3009 | Calendar + reminders |
| neuro-pomodoro | 3010 | Pomodoro timer |
| neuro-image | 3011 | AI image gallery |

## Features

### 🧠 GraphRAG Memory Engine
- **Graph + Vector Storage**: Uses SurrealDB for both relationship graphs and vector embeddings
- **11 Relation Types**: RelatedTo, Causes, PartOf, HasProperty, UsedFor, CapableOf, AtLocation, CreatedBy, DerivedFrom, SimilarTo, ContradictsWith
- **Semantic Search**: Find relevant memories using embedding similarity
- **Automatic Memory Extraction**: Extracts facts, preferences, and entities from conversations

### 🤖 Intelligent Agent System
- **Automatic Model Selection**: Chooses the best model based on available VRAM
  - `ministral-3b` (Fast) - Quick responses, <4GB VRAM
  - `qwen2.5:7b` (Balanced) - Good quality, 4-8GB VRAM
  - `qwen2.5-coder:14b` (Complex) - Best for coding, >8GB VRAM
- **Built-in Tools**:
  - `search_web`: Privacy-respecting web search via Searxng
  - `execute_command`: Safe local command execution (whitelisted)
  - `remember`: Store facts in long-term memory

### 🌐 REST API
- Complete Axum-based API
- Endpoints: `/chat`, `/memories`, `/admin/graph`, `/agent`, `/system`
- CORS support for frontend applications

### 💻 User Interfaces
- **User UI**: React + TypeScript + Tailwind chat interface
  - 🌐 **Web**: Runs in browser (localhost:5173)
  - 🖥️ **Desktop**: Native app for Windows/Linux/macOS via Tauri
  - Dark/Light mode
  - i18n support (English/Spanish)
  - Conversation history with grouping
  - Typing indicators and markdown rendering
  - **Desktop build**: See [NEURO_DESKTOP_SETUP.md](NEURO_DESKTOP_SETUP.md)

- **Admin UI**: Memory graph management dashboard
  - Force-directed graph visualization (react-force-graph)
  - Statistics dashboard with charts
  - Memory CRUD operations
  - System health monitoring

### 🐚 Z-Brain CLI
- Interactive shell for terminal-based interaction
- Command history with persistence
- Special commands: `/help`, `/new`, `/search`, `/models`
- Quick query mode: `zbrain "your question"`

## Project Structure

```
kibo/
├── docker-compose.yml          # Container orchestration
├── docker-compose.dev.yml      # Development overrides
├── dev.sh                      # Development helper script
├── config/
│   └── searxng/
│       └── settings.yml        # Searxng configuration
├── neuro-backend/              # API Gateway (Rust/Axum)
│   └── src/
│       ├── domain/             # Entities, Value Objects
│       ├── application/        # Business logic
│       └── infrastructure/     # API, DB, Adapters
├── neuro-checklists/           # Checklist microservice
├── neuro-music/                # Music streaming microservice
├── neuro-chat/                 # LLM chat microservice
├── neuro-memory/               # GraphRAG memory microservice
├── neuro-agent/                # Agent tools microservice
├── neuro-voice/                # TTS microservice
├── neuro-ui/                   # User interface (React)
├── neuro-admin/                # Admin dashboard (React)
└── zbrain/                     # CLI shell
```

## Quick Start

### Prerequisites
- Docker & Docker Compose
- Node.js 18+
- Rust 1.75+
- NVIDIA GPU with CUDA (optional, for GPU acceleration)

### 1. Clone and Configure

```bash
cd kibo
cp .env.example .env
# Edit .env with your settings
```

### 2. Start Ollama (External)

First, clone and start neuro-ollama in a separate directory:

```bash
# In a separate project directory (not in kibo)
git clone https://github.com/madkoding/neuro-ollama.git
cd neuro-ollama
./setup.sh  # Downloads models and starts Ollama
```

### 3. Start Infrastructure

```bash
# In the kibo directory
docker-compose up -d surrealdb searxng
```

### 4. Run Backend

```bash
cd neuro-backend
cargo run --release
```

### 5. Run User Interface

**Web version:**
```bash
cd neuro-ui
npm install
npm run dev
```

**Desktop version:**
```bash
cd neuro-ui
npm install
npm run tauri:dev  # Development with hot-reload
# Or for production build:
npm run tauri:build  # Generates native executable
```

See [NEURO_DESKTOP_SETUP.md](NEURO_DESKTOP_SETUP.md) for complete desktop build guide.

### 6. Run Admin Interface (Optional)

```bash
cd neuro-admin
npm install
npm run dev
```

### 7. Build Z-Brain CLI (Optional)

```bash
cd zbrain
cargo build --release
# Binary at target/release/zbrain
./target/release/zbrain
```

## API Endpoints

### Chat
- `POST /api/chat` - Send a message and get AI response

### Memories
- `GET /api/memories` - List all memories
- `POST /api/memories` - Create a memory
- `GET /api/memories/search?query=...` - Search memories
- `GET /api/memories/:id` - Get memory by ID
- `DELETE /api/memories/:id` - Delete memory
- `GET /api/memories/:id/related` - Get related memories

### Admin Graph
- `GET /api/admin/graph` - Get full memory graph
- `GET /api/admin/graph/stats` - Get graph statistics

### System
- `GET /api/system/health` - Health check
- `GET /api/system/models` - List available models
- `GET /api/system/vram` - Get VRAM information

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NEURO_API_PORT` | Backend port | 3000 |
| `SURREALDB_URL` | SurrealDB connection | ws://localhost:8000 |
| `SURREALDB_USER` | Database user | root |
| `SURREALDB_PASS` | Database password | root |
| `OLLAMA_URL` | Ollama API URL | http://localhost:11434 |
| `SEARXNG_URL` | Searxng URL | http://localhost:8080 |
| `FAST_MODEL` | Quick response model | ministral:3b |
| `BALANCED_MODEL` | Balanced model | qwen2.5:7b |
| `COMPLEX_MODEL` | Complex task model | qwen2.5-coder:14b |
| `EMBED_MODEL` | Embedding model | nomic-embed-text |

## Development

### Backend Development
```bash
cd neuro-backend
cargo watch -x run  # Auto-reload on changes
```

### Frontend Development
```bash
cd neuro-ui
npm run dev  # Vite dev server with HMR
```

### Running Tests
```bash
# Backend tests
cd neuro-backend
cargo test

# Z-Brain tests
cd zbrain
cargo test
```

## License

MIT License - See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please read our contributing guidelines before submitting PRs.

---

Built with ❤️ using Rust, React, and AI
