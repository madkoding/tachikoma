# NEURO-OS 🧠

**NEURO-OS** is a modular AI ecosystem that combines a memory graph (GraphRAG), intelligent agents with tool capabilities, and automatic model selection based on available VRAM.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│                           NEURO-OS                                   │
├─────────────────────────────────────────────────────────────────────┤
│                                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐ │
│  │  User UI    │  │  Admin UI   │  │   Z-Brain   │  │ External    │ │
│  │  (React)    │  │  (React)    │  │   (CLI)     │  │ Clients     │ │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘ │
│         │                │                │                │        │
│         └────────────────┴────────────────┴────────────────┘        │
│                                   │                                  │
│                          ┌────────┴────────┐                        │
│                          │   REST API      │                        │
│                          │   (Axum)        │                        │
│                          └────────┬────────┘                        │
│                                   │                                  │
│  ┌────────────────────────────────┴────────────────────────────────┐│
│  │                    Application Layer                            ││
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────────────────┐││
│  │  │ ChatService  │ │MemoryService│ │   AgentOrchestrator      │││
│  │  └──────────────┘ └──────────────┘ │  ┌─────────┐ ┌────────┐ │││
│  │                                     │  │ Tools   │ │ModelMgr│ │││
│  │                                     │  └─────────┘ └────────┘ │││
│  │                                     └──────────────────────────┘││
│  └─────────────────────────────────────────────────────────────────┘│
│                                   │                                  │
│  ┌────────────────────────────────┴────────────────────────────────┐│
│  │                   Infrastructure Layer                          ││
│  │  ┌──────────────┐ ┌──────────────┐ ┌──────────────┐            ││
│  │  │SurrealDB     │ │ Ollama       │ │ Searxng      │            ││
│  │  │(Graph+Vector)│ │ (LLM)        │ │ (Search)     │            ││
│  │  └──────────────┘ └──────────────┘ └──────────────┘            ││
│  └─────────────────────────────────────────────────────────────────┘│
│                                                                      │
└─────────────────────────────────────────────────────────────────────┘
```

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
  - Dark/Light mode
  - i18n support (English/Spanish)
  - Conversation history with grouping
  - Typing indicators and markdown rendering

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
├── .env.example                # Environment template
├── config/
│   └── searxng/
│       └── settings.yml        # Searxng configuration
├── neuro-backend/              # Rust backend
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── domain/             # Entities, Value Objects, Ports
│       ├── application/        # Services (ChatService, MemoryService, etc.)
│       └── infrastructure/     # Adapters (DB, Ollama, Searxng, API)
├── neuro-ui/                   # User chat interface
│   ├── package.json
│   └── src/
│       ├── components/         # React components
│       ├── pages/              # Page components
│       ├── stores/             # Zustand stores
│       └── api/                # API client
├── neuro-admin/                # Admin dashboard
│   ├── package.json
│   └── src/
│       ├── components/         # Layout, navigation
│       ├── pages/              # Dashboard, Graph, Memories
│       └── api/                # API client
└── zbrain/                     # CLI shell
    ├── Cargo.toml
    └── src/
        ├── main.rs
        ├── api.rs              # NEURO-OS client
        ├── config.rs           # Configuration
        └── shell.rs            # Interactive shell
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

### 2. Start Infrastructure

```bash
docker-compose up -d surrealdb searxng ollama
```

### 3. Pull Required Models

```bash
# Fast model (required)
docker exec -it ollama ollama pull ministral:3b

# Balanced model (recommended)
docker exec -it ollama ollama pull qwen2.5:7b

# Complex/Coding model (optional)
docker exec -it ollama ollama pull qwen2.5-coder:14b

# Embedding model (required)
docker exec -it ollama ollama pull nomic-embed-text
```

### 4. Run Backend

```bash
cd neuro-backend
cargo run --release
```

### 5. Run User Interface

```bash
cd neuro-ui
npm install
npm run dev
```

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
