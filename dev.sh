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
#   ./dev.sh voice        - Rebuild and restart voice service
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
    all|"")
        start_all
        ;;
    *)
        echo -e "${CYAN}Usage:${NC} ./dev.sh [command]"
        echo -e ""
        echo -e "${CYAN}Commands:${NC}"
        echo -e "  ${YELLOW}(none)${NC}    Start all services with hot reload (uses tmux)"
        echo -e "  ${YELLOW}backend${NC}   Start backend with cargo watch"
        echo -e "  ${YELLOW}ui${NC}        Start user UI with Vite HMR"
        echo -e "  ${YELLOW}admin${NC}     Start admin UI with Vite HMR"
        echo -e "  ${YELLOW}voice${NC}     Rebuild and restart voice service"
        exit 1
        ;;
esac
