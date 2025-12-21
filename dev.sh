#!/bin/bash
# =============================================================================
# NEURO-OS Development Script
# =============================================================================
# Starts services in development mode with auto-reload.
# Run with: ./dev.sh [backend|ui|admin|all]
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

# Parse command line argument
MODE=${1:-all}

print_header "NEURO-OS Development Mode"

# Ensure Docker services are running
echo -e "${GREEN}▶${NC} Ensuring Docker services are running..."
docker-compose up -d surrealdb searxng ollama
sleep 3

case $MODE in
    backend)
        echo -e "${GREEN}▶${NC} Starting backend with cargo-watch..."
        echo -e "${YELLOW}Note: Install cargo-watch with: cargo install cargo-watch${NC}"
        cd neuro-backend
        if command -v cargo-watch &> /dev/null; then
            cargo watch -x run
        else
            cargo run
        fi
        ;;
    ui)
        echo -e "${GREEN}▶${NC} Starting User UI in dev mode..."
        cd neuro-ui
        npm run dev
        ;;
    admin)
        echo -e "${GREEN}▶${NC} Starting Admin UI in dev mode..."
        cd neuro-admin
        npm run dev
        ;;
    all)
        echo -e "${GREEN}▶${NC} Starting all services..."
        echo -e "${YELLOW}Tip: Use tmux or separate terminals for better experience${NC}"
        echo ""
        echo -e "${CYAN}Run these commands in separate terminals:${NC}"
        echo -e "  ${YELLOW}Terminal 1 (Backend):${NC}  cd neuro-backend && cargo run"
        echo -e "  ${YELLOW}Terminal 2 (User UI):${NC}  cd neuro-ui && npm run dev"
        echo -e "  ${YELLOW}Terminal 3 (Admin UI):${NC} cd neuro-admin && npm run dev"
        echo ""
        echo -e "${CYAN}Or use the start script:${NC} ./start.sh"
        ;;
    *)
        echo -e "${RED}Unknown mode: $MODE${NC}"
        echo "Usage: ./dev.sh [backend|ui|admin|all]"
        exit 1
        ;;
esac
