#!/bin/bash
# =============================================================================
# NEURO-OS Start Script
# =============================================================================
# Starts all NEURO-OS services in the correct order.
# Usage:
#   ./start.sh          - Start all services (production mode)
#   ./start.sh --dev    - Start services in development mode (with hot reload)
#   ./start.sh --docker - Start only Docker services (for manual development)
# =============================================================================

set -e

# Parse arguments
MODE="prod"
if [ "$1" = "--dev" ]; then
    MODE="dev"
elif [ "$1" = "--docker" ]; then
    MODE="docker"
fi

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

print_header() {
    echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC} ${BLUE}$1${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"
}

print_step() {
    echo -e "${GREEN}▶${NC} $1"
}

print_header "Starting NEURO-OS"

# Check if .env exists
if [ ! -f .env ]; then
    echo -e "${RED}Error: .env file not found. Run ./setup.sh first.${NC}"
    exit 1
fi

# Function to wait for port to be free
wait_for_port_free() {
    local port=$1
    local max_wait=10
    local waited=0
    
    while lsof -ti :$port >/dev/null 2>&1; do
        if [ $waited -ge $max_wait ]; then
            echo -e "${YELLOW}Warning: Port $port still in use after ${max_wait}s, killing process...${NC}"
            fuser -k $port/tcp 2>/dev/null || true
            sleep 1
            break
        fi
        sleep 1
        waited=$((waited + 1))
    done
}

# Ensure critical ports are free before starting
print_step "Checking if ports are available..."
if lsof -ti :3000 >/dev/null 2>&1 || lsof -ti :5173 >/dev/null 2>&1 || lsof -ti :5174 >/dev/null 2>&1; then
    echo -e "  ${YELLOW}Some ports are in use. Cleaning up...${NC}"
    fuser -k 3000/tcp 2>/dev/null || true
    fuser -k 5173/tcp 2>/dev/null || true
    fuser -k 5174/tcp 2>/dev/null || true
    wait_for_port_free 3000
    wait_for_port_free 5173
    wait_for_port_free 5174
    echo -e "  ${GREEN}Ports cleaned${NC}"
else
    echo -e "  ${GREEN}All ports available${NC}"
fi

# Function to run docker commands (handles group membership issue)
run_docker() {
    if docker ps >/dev/null 2>&1; then
        docker "$@"
    elif sg docker -c "docker ps" >/dev/null 2>&1; then
        sg docker -c "docker $*"
    else
        echo -e "${RED}Error: Cannot access Docker. Please ensure you're in the docker group and re-login.${NC}"
        exit 1
    fi
}

# Start Docker services (user must be in docker group)
print_step "Starting Docker services..."
run_docker compose up -d surrealdb searxng ollama music-service

# Build and start Voice Service container (Rust)
print_step "Starting Voice Service (Rust)..."
if ! run_docker images | grep -q "neuro-voice"; then
    echo -e "  ${YELLOW}Building Voice Service image (first time only)...${NC}"
    run_docker build -t neuro-voice ./neuro-voice
fi
run_docker stop neuro-voice 2>/dev/null || true
run_docker rm neuro-voice 2>/dev/null || true

# Get the network name from docker-compose
NETWORK_NAME=$(run_docker network ls --filter name=kibo --format '{{.Name}}' | grep neuro-network | head -1)
if [ -z "$NETWORK_NAME" ]; then
    NETWORK_NAME="kibo_neuro-network"
    # Create network if it doesn't exist
    run_docker network create $NETWORK_NAME 2>/dev/null || true
fi

run_docker run -d --gpus all \
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
echo -e "  ${GREEN}Voice Service (Rust) started with GPU support!${NC}"

# If docker-only mode, exit here with instructions
if [ "$MODE" = "docker" ]; then
    echo -e "
${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}
${GREEN}║${NC}           ${CYAN}Docker services are running!${NC}                        ${GREEN}║${NC}
${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}

${CYAN}Docker Services:${NC}
  • SurrealDB:  ${YELLOW}http://localhost:8000${NC}
  • Ollama:     ${YELLOW}http://localhost:11434${NC}
  • Searxng:    ${YELLOW}http://localhost:8080${NC}
  • Voice API:  ${YELLOW}http://localhost:8100${NC}
  • Music API:  ${YELLOW}http://localhost:3002${NC}

${CYAN}Now start the app services manually:${NC}
  ${YELLOW}Terminal 1:${NC} cd neuro-backend && cargo run
  ${YELLOW}Terminal 2:${NC} cd neuro-ui && npm run dev
  ${YELLOW}Terminal 3:${NC} cd neuro-admin && npm run dev

${CYAN}Stop with:${NC} ./stop.sh
"
    exit 0
fi

# Wait for SurrealDB to be ready
print_step "Waiting for SurrealDB to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0
while ! curl -s http://127.0.0.1:8000/health >/dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${RED}Error: SurrealDB failed to start after $MAX_RETRIES attempts${NC}"
        exit 1
    fi
    echo -e "  Waiting for SurrealDB... (attempt $RETRY_COUNT/$MAX_RETRIES)"
    sleep 2
done
echo -e "${GREEN}  SurrealDB is ready!${NC}"

# Wait for Ollama to be ready
print_step "Waiting for Ollama to be ready..."
RETRY_COUNT=0
while ! curl -s http://127.0.0.1:11434/api/tags >/dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${YELLOW}Warning: Ollama not responding, continuing anyway...${NC}"
        break
    fi
    echo -e "  Waiting for Ollama... (attempt $RETRY_COUNT/$MAX_RETRIES)"
    sleep 2
done
echo -e "${GREEN}  Ollama is ready!${NC}"

# Pre-load the Light model (ministral-3:3b) - most used for simple queries
print_step "Pre-loading Light model (this may take ~30s on first run)..."
OLLAMA_LIGHT_MODEL="ministral-3:3b"
echo -e "  ${YELLOW}Loading model: $OLLAMA_LIGHT_MODEL${NC}"
# Send a simple prompt to force model loading
WARMUP_RESPONSE=$(curl -s --max-time 120 http://127.0.0.1:11434/api/chat \
    -d "{\"model\": \"$OLLAMA_LIGHT_MODEL\", \"messages\": [{\"role\": \"user\", \"content\": \"hi\"}], \"stream\": false}" 2>/dev/null)
if echo "$WARMUP_RESPONSE" | grep -q "content"; then
    echo -e "${GREEN}  Light model loaded and ready!${NC}"
else
    echo -e "${YELLOW}  Warning: Model warm-up may have failed. First request might be slow.${NC}"
fi

# Start backend in background
print_step "Starting backend..."
cd neuro-backend

# Voice service URL (Docker container)
export VOICE_SERVICE_URL="http://127.0.0.1:8100"
# Music service URL (Docker container)
export MUSIC_SERVICE_URL="http://127.0.0.1:3002"

# Note: OLLAMA_DEFAULT_MODEL is not used - ModelManager selects model dynamically based on task
if [ "$MODE" = "dev" ]; then
    echo -e "  ${YELLOW}Development mode: using cargo run${NC}"
    DATABASE_URL="ws://127.0.0.1:8000" \
    DATABASE_USER="root" \
    DATABASE_PASS="neuroos_secret_2024" \
    OLLAMA_URL="http://127.0.0.1:11434" \
    SEARXNG_URL="http://127.0.0.1:8080" \
    VOICE_SERVICE_URL="http://127.0.0.1:8100" \
    MUSIC_SERVICE_URL="http://127.0.0.1:3002" \
    RUST_LOG=debug \
    cargo run &
else
    DATABASE_URL="ws://127.0.0.1:8000" \
    DATABASE_USER="root" \
    DATABASE_PASS="neuroos_secret_2024" \
    OLLAMA_URL="http://127.0.0.1:11434" \
    SEARXNG_URL="http://127.0.0.1:8080" \
    VOICE_SERVICE_URL="http://127.0.0.1:8100" \
    MUSIC_SERVICE_URL="http://127.0.0.1:3002" \
    RUST_LOG=info \
    ./target/release/neuro-backend &
fi
BACKEND_PID=$!
cd ..

# Wait for backend
sleep 3

# Start User UI in background
print_step "Starting User UI..."
cd neuro-ui
npm run dev &
UI_PID=$!
cd ..

# Start Admin UI in background
print_step "Starting Admin UI..."
cd neuro-admin
npm run dev &
ADMIN_PID=$!
cd ..

echo -e "
${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}
${GREEN}║${NC}              ${CYAN}NEURO-OS is running!${NC}                           ${GREEN}║${NC}
${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}

${CYAN}Services:${NC}
  • User UI:    ${YELLOW}http://localhost:5173${NC}
  • Admin UI:   ${YELLOW}http://localhost:5174${NC}
  • Backend:    ${YELLOW}http://localhost:3000${NC}
  • Voice API:  ${YELLOW}http://localhost:8100${NC}
  • Music API:  ${YELLOW}http://localhost:3002${NC}

${CYAN}Process IDs:${NC}
  • Backend:  $BACKEND_PID
  • User UI:  $UI_PID
  • Admin UI: $ADMIN_PID

${CYAN}Press Ctrl+C to stop all services${NC}
"

# Trap Ctrl+C to cleanup
cleanup() {
    echo -e "\n${YELLOW}Stopping services...${NC}"
    kill $BACKEND_PID 2>/dev/null || true
    kill $UI_PID 2>/dev/null || true
    kill $ADMIN_PID 2>/dev/null || true
    run_docker stop neuro-voice 2>/dev/null || true
    echo -e "${GREEN}All services stopped.${NC}"
    exit 0
}

trap cleanup SIGINT SIGTERM

# Wait for any process to exit
wait
