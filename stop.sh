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

# Stop Docker services
echo -e "${YELLOW}▶${NC} Stopping Docker containers..."
docker-compose down

# Kill any running Node processes for neuro-ui and neuro-admin
echo -e "${YELLOW}▶${NC} Stopping frontend dev servers..."
pkill -f "vite.*neuro-ui" 2>/dev/null || true
pkill -f "vite.*neuro-admin" 2>/dev/null || true

# Kill backend
echo -e "${YELLOW}▶${NC} Stopping backend..."
pkill -f "neuro-backend" 2>/dev/null || true
pkill -f "target/release/neuro-backend" 2>/dev/null || true

echo -e "${GREEN}✓ All NEURO-OS services stopped.${NC}"
