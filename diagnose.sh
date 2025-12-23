#!/bin/bash
# =============================================================================
# NEURO-OS Diagnostic Script
# =============================================================================
# Checks the status of all services and network connectivity
# =============================================================================

# Don't exit on errors - we want to see all checks even if some fail
set +e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║${NC}              ${YELLOW}NEURO-OS Diagnostics${NC}                          ${CYAN}║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}\n"

# Get hostname and IP
HOSTNAME=$(hostname)
LOCAL_IP=$(ip -4 addr show | grep -oP '(?<=inet\s)\d+(\.\d+){3}' | grep -v 127.0.0.1 | head -1)

echo -e "${CYAN}Server Information:${NC}"
echo -e "  Hostname: ${YELLOW}${HOSTNAME}${NC}"
echo -e "  Local IP: ${YELLOW}${LOCAL_IP}${NC}\n"

# Function to check port
check_port() {
    local port=$1
    local name=$2
    local pid=$(lsof -ti :$port 2>/dev/null)
    
    if [ -n "$pid" ]; then
        local process=$(ps -p $pid -o comm= 2>/dev/null)
        echo -e "  ${GREEN}✓${NC} Port ${YELLOW}$port${NC} - $name (PID: $pid, Process: $process)"
        return 0
    else
        echo -e "  ${RED}✗${NC} Port ${YELLOW}$port${NC} - $name ${RED}NOT RUNNING${NC}"
        return 1
    fi
}

# Function to check HTTP endpoint
check_http() {
    local url=$1
    local name=$2
    
    if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -q "200\|404"; then
        echo -e "  ${GREEN}✓${NC} $name - ${GREEN}RESPONDING${NC}"
        return 0
    else
        echo -e "  ${RED}✗${NC} $name - ${RED}NOT RESPONDING${NC}"
        return 1
    fi
}
echo -e "${CYAN}Port Status:${NC}"
check_port 3000 "Backend API" || true
check_port 5173 "User UI (Vite)" || true
check_port 5174 "Admin UI (Vite)" || true
check_port 8000 "SurrealDB" || true
check_port 11434 "Ollama" || true
check_port 8080 "Searxng" || true
check_port 8100 "Voice Service" || true
check_port 8100 "Voice Service"
echo -e "\n${CYAN}HTTP Endpoints:${NC}"
check_http "http://localhost:3000/api/health" "Backend Health" || true
check_http "http://localhost:5173" "User UI" || true
check_http "http://localhost:5174" "Admin UI" || true
check_http "http://localhost:8000/health" "SurrealDB" || true
check_http "http://localhost:11434/api/tags" "Ollama" || true
check_http "http://localhost:8100/health" "Voice Service" || true
check_http "http://localhost:8100/health" "Voice Service"

echo -e "\n${CYAN}Docker Containers:${NC}"
if command -v docker &> /dev/null; then
    docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -E "neuro-|NAME" || echo -e "  ${YELLOW}No NEURO-OS containers running${NC}"
else
    echo -e "  ${RED}Docker not available${NC}"
fi

echo -e "\n${CYAN}Network Access Information:${NC}"
echo -e "${YELLOW}Local Access:${NC}"
echo -e "  • User UI:    http://localhost:5173"
echo -e "  • Admin UI:   http://localhost:5174"
echo -e "  • Backend:    http://localhost:3000/api/health"

if [ -n "$LOCAL_IP" ]; then
    echo -e "\n${YELLOW}Remote Access (from other machines):${NC}"
    echo -e "  • User UI:    http://${LOCAL_IP}:5173"
    echo -e "  • Admin UI:   http://${LOCAL_IP}:5174"
    echo -e "  • Backend:    http://${LOCAL_IP}:3000/api/health"
    
    if [ -n "$HOSTNAME" ]; then
        echo -e "\n${YELLOW}Remote Access (using hostname):${NC}"
        echo -e "  • User UI:    http://${HOSTNAME}:5173"
        echo -e "  • Admin UI:   http://${HOSTNAME}:5174"
        echo -e "  • Backend:    http://${HOSTNAME}:3000/api/health"
    fi
fi

echo -e "\n${CYAN}Proxy Configuration:${NC}"
echo -e "  User UI and Admin UI use Vite proxy:"
echo -e "  • Frontend requests to ${YELLOW}/api/*${NC} are proxied to ${YELLOW}http://localhost:3000${NC}"
echo -e "  • Frontend requests to ${YELLOW}/voice/*${NC} are proxied to ${YELLOW}http://localhost:8100${NC}"

echo -e "\n${CYAN}Troubleshooting:${NC}"
echo -e "  ${YELLOW}If Ollama is not running:${NC}"
echo -e "    ${YELLOW}./fix-ollama.sh${NC}"
echo -e ""
echo -e "  ${YELLOW}If remote access doesn't work:${NC}"
echo -e "    1. Check firewall: ${YELLOW}sudo ufw status${NC}"
echo -e "    2. Allow ports: ${YELLOW}sudo ufw allow 3000,5173,5174/tcp${NC}"
echo -e "    3. Restart services: ${YELLOW}./stop.sh && ./start.sh${NC}"
echo -e "    4. Check logs in service terminals"
echo -e ""
echo -e "  ${YELLOW}If services won't start (ports busy):${NC}"
echo -e "    ${YELLOW}./stop.sh${NC}    # Stop everything"
echo -e "    ${YELLOW}sleep 2${NC}       # Wait"
echo -e "    ${YELLOW}./start.sh${NC}    # Start again"
echo -e ""
echo -e "  ${YELLOW}Quick fixes:${NC}"
echo -e "    • Ollama:   ${YELLOW}./fix-ollama.sh${NC}"
echo -e "    • All:      ${YELLOW}./fix-remote-access.sh${NC}"

echo -e "\n${CYAN}Current Processes:${NC}"
echo -e "${YELLOW}Backend:${NC}"
ps aux | grep -E "neuro-backend|cargo.*run" | grep -v grep | awk '{print "  PID: "$2" | CMD: "$11" "$12" "$13}' || echo -e "  ${RED}Not running${NC}"

echo -e "${YELLOW}Frontend (Vite):${NC}"
ps aux | grep -E "vite.*neuro-(ui|admin)" | grep -v grep | awk '{print "  PID: "$2" | "$11" "$12}' || echo -e "  ${RED}Not running${NC}"

echo ""
