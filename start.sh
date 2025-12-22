#!/bin/bash
# =============================================================================
# NEURO-OS Start Script
# =============================================================================
# Starts all NEURO-OS services in the correct order.
# Run with: ./start.sh
# =============================================================================

set -e

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
run_docker compose up -d surrealdb searxng ollama

# Build and start Voice Service container
print_step "Starting Voice Service..."
if ! run_docker images | grep -q "neuro-voice"; then
    echo -e "  ${YELLOW}Building Voice Service image (first time only)...${NC}"
    run_docker build -t neuro-voice ./voice-service
fi
run_docker stop neuro-voice 2>/dev/null || true
run_docker rm neuro-voice 2>/dev/null || true
run_docker run -d --gpus all --name neuro-voice --network neuro-os-network -p 8100:8100 neuro-voice
echo -e "  ${GREEN}Voice Service started with GPU support!${NC}"

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

# Start backend in background
print_step "Starting backend..."
cd neuro-backend

# Voice service URL (Docker container)
export VOICE_SERVICE_URL="http://127.0.0.1:8100"

DATABASE_URL="ws://127.0.0.1:8000" \
DATABASE_USER="root" \
DATABASE_PASS="neuroos_secret_2024" \
OLLAMA_URL="http://127.0.0.1:11434" \
OLLAMA_DEFAULT_MODEL="qwen2.5:3b" \
SEARXNG_URL="http://127.0.0.1:8080" \
RUST_LOG=info \
./target/release/neuro-backend &
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
