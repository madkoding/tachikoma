#!/bin/bash
# =============================================================================
# Quick Fix Script - Apply all remote access fixes
# =============================================================================

set -e

CYAN='\033[0;36m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}       ${YELLOW}NEURO-OS Remote Access Fix${NC}                        ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"

echo -e "${GREEN}▶${NC} Stopping all services..."
./stop.sh

echo -e "${GREEN}▶${NC} Waiting for ports to be free..."
sleep 3

echo -e "${GREEN}▶${NC} Making diagnose.sh executable..."
chmod +x diagnose.sh

echo -e "${GREEN}▶${NC} Starting services..."
./start.sh

echo -e "\n${GREEN}▶${NC} Running diagnostics..."
sleep 5
./diagnose.sh

echo -e "\n${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}                    ${GREEN}Fix Applied!${NC}                            ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"

echo -e "${YELLOW}Changes applied:${NC}"
echo -e "  ✓ Vite proxy configuration fixed (neuro-ui & neuro-admin)"
echo -e "  ✓ Stop script improved to kill processes properly"
echo -e "  ✓ Start script validates ports before starting"
echo -e "  ✓ Environment files created"
echo -e "  ✓ Diagnostic script added"

echo -e "\n${YELLOW}Test remote access:${NC}"
echo -e "  From another machine, open:"
IP=$(ip -4 addr show | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v 127.0.0.1 | head -1)
HOSTNAME=$(hostname)
echo -e "  • http://${IP}:5173 (User UI)"
echo -e "  • http://${IP}:5174 (Admin UI)"
if [ -n "$HOSTNAME" ]; then
    echo -e "  • http://${HOSTNAME}:5173 (User UI)"
    echo -e "  • http://${HOSTNAME}:5174 (Admin UI)"
fi

echo ""
