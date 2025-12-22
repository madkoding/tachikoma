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

# Kill backend first
echo -e "${YELLOW}▶${NC} Stopping backend..."
pkill -f "neuro-backend" 2>/dev/null || true

# Kill any running Node/Vite processes
echo -e "${YELLOW}▶${NC} Stopping frontend dev servers..."
pkill -f "vite" 2>/dev/null || true
pkill -f "esbuild" 2>/dev/null || true

# Stop Docker services (don't use sudo - user should be in docker group)
echo -e "${YELLOW}▶${NC} Stopping Docker containers..."
docker compose down 2>/dev/null || true

echo -e "${GREEN}✓ All NEURO-OS services stopped.${NC}"
