#!/bin/bash
# =============================================================================
# NEURO-OS Development Script with Hot Reload
# =============================================================================
# Starts all services with automatic reload on code changes.
# 
# Requirements:
#   - cargo-watch: cargo install cargo-watch
#   - Docker services running: ./start.sh --docker
#
# Usage:
#   ./dev.sh              - Start all dev services with hot reload
#   ./dev.sh backend      - Start only backend with hot reload
#   ./dev.sh ui           - Start only user UI
#   ./dev.sh admin        - Start only admin UI
#   ./dev.sh voice        - Rebuild and restart voice service (release)
#
# FAST DEV MODE (Docker, 3-5x faster builds):
#   ./dev.sh docker-dev   - Start Docker services with dev optimizations
#   ./dev.sh rebuild-voice    - Rebuild voice service (debug + mold)
#   ./dev.sh rebuild-music    - Rebuild music service (debug + mold)
#   ./dev.sh rebuild-checklists - Rebuild checklists service (debug + mold)
#   ./dev.sh clean-cache  - Clear persistent cargo cache
#
# Fast Dev Mode uses:
#   - Debug builds (no --release) - ~30% faster compilation
#   - mold linker - 2-3x faster linking
#   - Persistent cargo cache - dependencies compile ONCE
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m'

PROJECT_ROOT="$(cd "$(dirname "$0")" && pwd)"
cd "$PROJECT_ROOT"

print_header() {
    echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC} ${BLUE}$1${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    if ! command -v cargo-watch &> /dev/null; then
        print_warning "cargo-watch not found. Installing..."
        cargo install cargo-watch
    fi
    
    if [ ! -f .env ]; then
        echo -e "${RED}Error: .env file not found. Run ./setup.sh first.${NC}"
        exit 1
    fi
}

# Check if Docker services are running
check_docker_services() {
    if ! docker ps 2>/dev/null | grep -q "surrealdb"; then
        print_warning "Docker services not running. Starting them first..."
        ./start.sh --docker
        sleep 3
    fi
}

# Export common environment variables
export_env() {
    export DATABASE_URL="ws://127.0.0.1:8000"
    export DATABASE_USER="root"
    export DATABASE_PASS="neuroos_secret_2024"
    export OLLAMA_URL="http://127.0.0.1:11434"
    export SEARXNG_URL="http://127.0.0.1:8080"
    export VOICE_SERVICE_URL="http://127.0.0.1:8100"
    export RUST_LOG=debug
}

# Start backend with hot reload
start_backend() {
    print_step "Starting backend with hot reload..."
    echo -e "  ${CYAN}Watching: neuro-backend/src/**/*.rs${NC}"
    echo -e "  ${YELLOW}Press Ctrl+C to stop${NC}\n"
    
    cd neuro-backend
    export_env
    cargo watch -x run -w src
}

# Start user UI (Vite already has HMR)
start_ui() {
    print_step "Starting User UI with HMR..."
    echo -e "  ${CYAN}URL: http://localhost:5173${NC}"
    echo -e "  ${YELLOW}Press Ctrl+C to stop${NC}\n"
    
    cd neuro-ui
    npm run dev
}

# Start admin UI (Vite already has HMR)
start_admin() {
    print_step "Starting Admin UI with HMR..."
    echo -e "  ${CYAN}URL: http://localhost:5174${NC}"
    echo -e "  ${YELLOW}Press Ctrl+C to stop${NC}\n"
    
    cd neuro-admin
    npm run dev
}

# Rebuild and restart voice service
restart_voice() {
    print_step "Rebuilding and restarting Voice Service..."
    
    # Stop existing container
    docker stop neuro-voice 2>/dev/null || true
    docker rm neuro-voice 2>/dev/null || true
    
    # Rebuild image
    echo -e "  ${YELLOW}Building image...${NC}"
    docker build -t neuro-voice ./neuro-voice
    
    # Get network name
    NETWORK_NAME=$(docker network ls --filter name=kibo --format '{{.Name}}' | grep neuro-network | head -1)
    if [ -z "$NETWORK_NAME" ]; then
        NETWORK_NAME="kibo_neuro-network"
    fi
    
    # Start container
    docker run -d --gpus all \
        --name neuro-voice \
        --network $NETWORK_NAME \
        -p 8100:8100 \
        -e RUST_LOG=info,voice_service=debug \
        -e HOST=0.0.0.0 \
        -e PORT=8100 \
        -e PIPER_BIN=/app/piper/piper \
        -e MODELS_DIR=/app/models \
        -e DEFAULT_VOICE=es_MX-claude-high \
        neuro-voice
    
    echo -e "  ${GREEN}Voice Service restarted!${NC}"
}

# =============================================================================
# FAST DEV MODE - Uses debug builds + persistent cache
# =============================================================================
# Builds are ~3-5x faster because:
# 1. Uses debug mode (no optimizations)
# 2. Uses mold linker (2-3x faster linking)
# 3. Persistent cargo cache (dependencies compile once)
# =============================================================================

# Start Docker services in fast dev mode
start_docker_dev() {
    print_header "Starting Docker services in FAST DEV mode"
    
    echo -e "${CYAN}Using development optimizations:${NC}"
    echo -e "  • ${GREEN}Debug builds${NC} (no --release, ~30% faster)"
    echo -e "  • ${GREEN}mold linker${NC} (2-3x faster linking)"
    echo -e "  • ${GREEN}Persistent cache${NC} (dependencies compile once)"
    echo -e ""
    
    # Always rebuild to catch source changes
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d --build
    
    echo -e "\n${GREEN}✓ Dev services started!${NC}"
    echo -e "  Voice:      ${YELLOW}http://localhost:8100${NC}"
    echo -e "  Music:      ${YELLOW}http://localhost:3002${NC}"
    echo -e "  Checklists: ${YELLOW}http://localhost:3001${NC}"
    echo -e "  Chat:       ${YELLOW}http://localhost:3003${NC}"
    echo -e "  Memory:     ${YELLOW}http://localhost:3004${NC}"
    echo -e "  Agent:      ${YELLOW}http://localhost:3005${NC}"
    echo -e "  Kanban:     ${YELLOW}http://localhost:3006${NC}"
    echo -e "  Note:       ${YELLOW}http://localhost:3007${NC}"
    echo -e "  Docs:       ${YELLOW}http://localhost:3008${NC}"
    echo -e "  Calendar:   ${YELLOW}http://localhost:3009${NC}"
    echo -e "  Pomodoro:   ${YELLOW}http://localhost:3010${NC}"
    echo -e "  Image:      ${YELLOW}http://localhost:3011${NC}"
    echo -e ""
    echo -e "${CYAN}Tip:${NC} Use ${YELLOW}./dev.sh watch${NC} for auto-rebuild on changes"
}

# Start Docker services in watch mode (auto-rebuild on changes)
start_docker_watch() {
    print_header "Starting Docker services in WATCH mode (auto-rebuild)"
    
    echo -e "${CYAN}Watch mode features:${NC}"
    echo -e "  • ${GREEN}Auto-rebuild${NC} when source files change"
    echo -e "  • ${GREEN}Solo reconstruye el servicio modificado${NC} (los demás siguen corriendo)"
    echo -e "  • ${GREEN}Debug builds${NC} (no --release, ~30% faster)"
    echo -e "  • ${GREEN}mold linker${NC} (2-3x faster linking)"
    echo -e "  • ${GREEN}Persistent cache${NC} (dependencies compile once)"
    echo -e "  • ${GREEN}Rebuild típico: ~30s${NC} (vs 3-5min en release)"
    echo -e ""
    echo -e "${YELLOW}Watching for changes in:${NC}"
    echo -e "  • neuro-voice/src/**/*.rs       → Reconstruye solo neuro-voice"
    echo -e "  • neuro-music/src/**/*.rs       → Reconstruye solo neuro-music"
    echo -e "  • neuro-checklists/src/**/*.rs  → Reconstruye solo neuro-checklists"
    echo -e "  • neuro-chat/src/**/*.rs        → Reconstruye solo neuro-chat"
    echo -e "  • neuro-memory/src/**/*.rs      → Reconstruye solo neuro-memory"
    echo -e "  • neuro-agent/src/**/*.rs       → Reconstruye solo neuro-agent"
    echo -e "  • neuro-*/Cargo.toml"
    echo -e ""
    echo -e "${CYAN}Press Ctrl+C to stop${NC}"
    echo -e ""
    
    # Use docker compose watch for auto-rebuild
    docker compose -f docker-compose.yml -f docker-compose.dev.yml -f docker-compose.watch.yml watch
}

# Rebuild a specific service in dev mode (fast)
rebuild_service_dev() {
    local service=$1
    print_step "Rebuilding $service in FAST DEV mode..."
    
    echo -e "  ${CYAN}Using debug build + mold linker${NC}"
    
    docker compose -f docker-compose.yml -f docker-compose.dev.yml build --no-cache $service
    docker compose -f docker-compose.yml -f docker-compose.dev.yml up -d $service
    
    echo -e "  ${GREEN}$service rebuilt and restarted!${NC}"
}

# Clean cargo cache (if needed)
clean_cargo_cache() {
    print_step "Cleaning persistent cargo cache..."
    
    docker volume rm neuro-cargo-cache-voice neuro-cargo-git-voice neuro-voice-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-music neuro-cargo-git-music neuro-music-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-checklists neuro-cargo-git-checklists neuro-checklists-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-chat neuro-cargo-git-chat neuro-chat-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-memory neuro-cargo-git-memory neuro-memory-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-agent neuro-cargo-git-agent neuro-agent-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-pomodoro neuro-cargo-git-pomodoro neuro-pomodoro-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-kanban neuro-cargo-git-kanban neuro-kanban-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-note neuro-cargo-git-note neuro-note-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-docs neuro-cargo-git-docs neuro-docs-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-calendar neuro-cargo-git-calendar neuro-calendar-target 2>/dev/null || true
    docker volume rm neuro-cargo-cache-image neuro-cargo-git-image neuro-image-target 2>/dev/null || true
    
    echo -e "  ${GREEN}Cache cleaned! Next build will be slower but fresh.${NC}"
}

# Start all services in tmux (if available) or parallel
start_all() {
    print_header "NEURO-OS Development Mode"
    
    check_docker_services
    
    # Check if tmux is available
    if command -v tmux &> /dev/null; then
        echo -e "${CYAN}Using tmux for multiple terminals${NC}\n"
        
        # Kill existing session if any
        tmux kill-session -t neuro-dev 2>/dev/null || true
        
        # Create new session with backend
        tmux new-session -d -s neuro-dev -n backend
        tmux send-keys -t neuro-dev:backend "cd $PROJECT_ROOT && ./dev.sh backend" Enter
        
        # Create window for UI
        tmux new-window -t neuro-dev -n ui
        tmux send-keys -t neuro-dev:ui "cd $PROJECT_ROOT && ./dev.sh ui" Enter
        
        # Create window for admin
        tmux new-window -t neuro-dev -n admin
        tmux send-keys -t neuro-dev:admin "cd $PROJECT_ROOT && ./dev.sh admin" Enter
        
        echo -e "
${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}
${GREEN}║${NC}         ${CYAN}NEURO-OS Dev Mode (tmux session)${NC}                     ${GREEN}║${NC}
${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}

${CYAN}Services with Hot Reload:${NC}
  • ${MAGENTA}backend${NC}  - Rust backend (cargo watch)
  • ${MAGENTA}ui${NC}       - User interface (Vite HMR)
  • ${MAGENTA}admin${NC}    - Admin interface (Vite HMR)

${CYAN}URLs:${NC}
  • User UI:    ${YELLOW}http://localhost:5173${NC}
  • Admin UI:   ${YELLOW}http://localhost:5174${NC}
  • Backend:    ${YELLOW}http://localhost:3000${NC}
  • Voice API:  ${YELLOW}http://localhost:8100${NC}

${CYAN}tmux Commands:${NC}
  • Attach:     ${YELLOW}tmux attach -t neuro-dev${NC}
  • Switch:     ${YELLOW}Ctrl+B then 0/1/2${NC} (backend/ui/admin)
  • Detach:     ${YELLOW}Ctrl+B then D${NC}
  • Kill:       ${YELLOW}tmux kill-session -t neuro-dev${NC}

${CYAN}To stop:${NC} ${YELLOW}./stop.sh${NC} or ${YELLOW}tmux kill-session -t neuro-dev${NC}
"
        # Attach to session
        tmux attach -t neuro-dev
    else
        # No tmux - run in foreground with instructions
        echo -e "
${YELLOW}tmux not found. Run these in separate terminals:${NC}

  ${CYAN}Terminal 1 (Backend):${NC}
    ./dev.sh backend

  ${CYAN}Terminal 2 (User UI):${NC}
    ./dev.sh ui

  ${CYAN}Terminal 3 (Admin UI):${NC}
    ./dev.sh admin
"
    fi
}

# Main
check_prerequisites

case "${1:-all}" in
    backend|be|b)
        check_docker_services
        start_backend
        ;;
    ui|user|u)
        start_ui
        ;;
    admin|a)
        start_admin
        ;;
    voice|v)
        restart_voice
        ;;
    docker-dev|dd)
        start_docker_dev
        ;;
    watch|w)
        start_docker_watch
        ;;
    rebuild-voice|rv)
        rebuild_service_dev neuro-voice
        ;;
    rebuild-music|rm)
        rebuild_service_dev neuro-music
        ;;
    rebuild-checklists|rc)
        rebuild_service_dev neuro-checklists
        ;;
    rebuild-chat|rch)
        rebuild_service_dev neuro-chat
        ;;
    rebuild-memory|rmem)
        rebuild_service_dev neuro-memory
        ;;
    rebuild-agent|ra)
        rebuild_service_dev neuro-agent
        ;;
    rebuild-pomodoro|rp)
        rebuild_service_dev neuro-pomodoro
        ;;
    rebuild-kanban|rk)
        rebuild_service_dev neuro-kanban
        ;;
    rebuild-note|rn)
        rebuild_service_dev neuro-note
        ;;
    rebuild-docs|rd)
        rebuild_service_dev neuro-docs
        ;;
    rebuild-calendar|rcal)
        rebuild_service_dev neuro-calendar
        ;;
    rebuild-image|ri)
        rebuild_service_dev neuro-image
        ;;
    clean-cache|cc)
        clean_cargo_cache
        ;;
    all|"")
        start_all
        ;;
    *)
        echo -e "${CYAN}Usage:${NC} ./dev.sh [command]"
        echo -e ""
        echo -e "${CYAN}Local Development:${NC}"
        echo -e "  ${YELLOW}(none)${NC}         Start all services with hot reload (uses tmux)"
        echo -e "  ${YELLOW}backend${NC}        Start backend with cargo watch"
        echo -e "  ${YELLOW}ui${NC}             Start user UI with Vite HMR"
        echo -e "  ${YELLOW}admin${NC}          Start admin UI with Vite HMR"
        echo -e ""
        echo -e "${CYAN}Docker Fast Dev Mode (3-5x faster builds):${NC}"
        echo -e "  ${YELLOW}docker-dev${NC}     Start Docker services with dev optimizations"
        echo -e "  ${YELLOW}watch${NC}          ${GREEN}Auto-rebuild on changes${NC} (solo reconstruye servicio modificado)"
        echo -e "  ${YELLOW}rebuild-voice${NC}  Rebuild voice service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-music${NC}  Rebuild music service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-checklists${NC}  Rebuild checklists service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-chat${NC}   Rebuild chat service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-memory${NC} Rebuild memory service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-agent${NC}  Rebuild agent service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-pomodoro${NC} Rebuild pomodoro service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-kanban${NC} Rebuild kanban service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-note${NC}   Rebuild note service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-docs${NC}   Rebuild docs service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-calendar${NC} Rebuild calendar service (fast dev mode)"
        echo -e "  ${YELLOW}rebuild-image${NC}  Rebuild image service (fast dev mode)"
        echo -e "  ${YELLOW}clean-cache${NC}    Clean persistent cargo cache"
        echo -e ""
        echo -e "${CYAN}Legacy (slower):${NC}"
        echo -e "  ${YELLOW}voice${NC}          Rebuild voice service (release mode)"
        exit 1
        ;;
esac
