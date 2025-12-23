#!/bin/bash
# =============================================================================
# Fix Ollama Service
# =============================================================================
# Restarts Ollama if it's not running
# =============================================================================

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}              ${YELLOW}Fixing Ollama Service${NC}                         ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"

# Function to run docker commands (handles group membership issue)
run_docker() {
    if docker ps >/dev/null 2>&1; then
        docker "$@"
    elif sg docker -c "docker ps" >/dev/null 2>&1; then
        sg docker -c "docker $*"
    else
        echo -e "${RED}Error: Cannot access Docker. Please ensure you're in the docker group.${NC}"
        exit 1
    fi
}

echo -e "${YELLOW}▶${NC} Checking Ollama status..."
if run_docker ps | grep -q neuro-ollama; then
    echo -e "  ${GREEN}Ollama container is running${NC}"
    
    # Check if it's responding
    if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
        echo -e "  ${GREEN}Ollama API is responding${NC}"
        echo -e "\n${GREEN}✓ Ollama is working correctly${NC}"
        exit 0
    else
        echo -e "  ${YELLOW}Ollama container running but not responding${NC}"
        echo -e "  ${YELLOW}Restarting container...${NC}"
        run_docker restart neuro-ollama
    fi
else
    echo -e "  ${RED}Ollama container is not running${NC}"
    
    # Check if container exists but stopped
    if run_docker ps -a | grep -q neuro-ollama; then
        echo -e "${YELLOW}▶${NC} Starting existing Ollama container..."
        run_docker start neuro-ollama
    else
        echo -e "${YELLOW}▶${NC} Starting Ollama with docker-compose..."
        run_docker compose up -d ollama
    fi
fi

echo -e "\n${YELLOW}▶${NC} Waiting for Ollama to be ready..."
MAX_RETRIES=30
RETRY_COUNT=0
while ! curl -s http://localhost:11434/api/tags >/dev/null 2>&1; do
    RETRY_COUNT=$((RETRY_COUNT + 1))
    if [ $RETRY_COUNT -ge $MAX_RETRIES ]; then
        echo -e "${RED}✗ Ollama failed to start after $MAX_RETRIES attempts${NC}"
        echo -e "\n${YELLOW}Checking logs:${NC}"
        run_docker logs neuro-ollama --tail 20
        exit 1
    fi
    echo -e "  Waiting... (attempt $RETRY_COUNT/$MAX_RETRIES)"
    sleep 2
done

echo -e "${GREEN}✓ Ollama is ready!${NC}"

# Test with a simple request
echo -e "\n${YELLOW}▶${NC} Testing Ollama..."
if curl -s http://localhost:11434/api/tags | grep -q "models"; then
    echo -e "${GREEN}✓ Ollama API is working correctly${NC}"
    
    # Show available models
    echo -e "\n${CYAN}Available models:${NC}"
    curl -s http://localhost:11434/api/tags | grep -o '"name":"[^"]*"' | cut -d'"' -f4 | while read model; do
        echo -e "  • ${YELLOW}$model${NC}"
    done
else
    echo -e "${YELLOW}⚠ Ollama is running but may not have models installed${NC}"
    echo -e "\n${CYAN}To install required models, run:${NC}"
    echo -e "  docker exec -it neuro-ollama ollama pull ministral-3:3b"
    echo -e "  docker exec -it neuro-ollama ollama pull qwen2.5:7b"
    echo -e "  docker exec -it neuro-ollama ollama pull nomic-embed-text"
fi

echo -e "\n${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${GREEN}║${NC}                  ${CYAN}Ollama is ready!${NC}                          ${GREEN}║${NC}"
echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
