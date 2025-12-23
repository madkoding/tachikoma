#!/bin/bash
# =============================================================================
# NEURO-OS Stop Script
# =============================================================================
# Stops all NEURO-OS services.
# Run with: ./stop.sh
# =============================================================================

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}Stopping NEURO-OS services...${NC}"

# Kill backend first (más agresivo)
echo -e "${YELLOW}▶${NC} Stopping backend..."
pkill -9 -f "neuro-backend" 2>/dev/null || true
pkill -9 -f "cargo.*run.*neuro-backend" 2>/dev/null || true

# Kill any running Node/Vite processes (más agresivo)
echo -e "${YELLOW}▶${NC} Stopping frontend dev servers..."
pkill -9 -f "vite.*neuro-ui" 2>/dev/null || true
pkill -9 -f "vite.*neuro-admin" 2>/dev/null || true
pkill -9 -f "node.*vite" 2>/dev/null || true
pkill -9 -f "esbuild" 2>/dev/null || true

# Liberar puertos específicos
echo -e "${YELLOW}▶${NC} Freeing ports..."
fuser -k 3000/tcp 2>/dev/null || true
fuser -k 5173/tcp 2>/dev/null || true
fuser -k 5174/tcp 2>/dev/null || true
sleep 1

# Stop Voice Service container
echo -e "${YELLOW}▶${NC} Stopping Voice Service..."
docker stop neuro-voice 2>/dev/null || true
docker rm neuro-voice 2>/dev/null || true

# Stop Docker services (don't use sudo - user should be in docker group)
echo -e "${YELLOW}▶${NC} Stopping Docker containers..."
docker compose down 2>/dev/null || true

# Clean up network if empty
echo -e "${YELLOW}▶${NC} Cleaning up networks..."
docker network prune -f 2>/dev/null || true

echo -e "${GREEN}✓ All NEURO-OS services stopped.${NC}"
