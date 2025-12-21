#!/bin/bash
# =============================================================================
# NEURO-OS Setup Script
# =============================================================================
# This script automates the complete setup of NEURO-OS ecosystem.
# Run with: ./setup.sh
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Print functions
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

print_error() {
    echo -e "${RED}✗${NC} $1"
}

print_success() {
    echo -e "${GREEN}✓${NC} $1"
}

# Check if command exists
check_command() {
    if ! command -v $1 &> /dev/null; then
        print_error "$1 is not installed. Please install it first."
        exit 1
    fi
}

# =============================================================================
# Pre-flight checks
# =============================================================================
print_header "NEURO-OS Setup - Pre-flight Checks"

print_step "Checking required tools..."
check_command docker
check_command docker-compose
check_command node
check_command npm
check_command cargo

print_success "All required tools are installed"

# =============================================================================
# Environment Setup
# =============================================================================
print_header "Environment Configuration"

if [ ! -f .env ]; then
    print_step "Creating .env from .env.example..."
    cp .env.example .env
    print_success ".env file created"
else
    print_warning ".env file already exists, skipping..."
fi

# =============================================================================
# Docker Services
# =============================================================================
print_header "Starting Docker Services"

print_step "Starting SurrealDB, Searxng, and Ollama..."
docker-compose up -d surrealdb searxng ollama

print_step "Waiting for services to be ready..."
sleep 10

# Check if services are running
if docker-compose ps | grep -q "surrealdb.*Up"; then
    print_success "SurrealDB is running"
else
    print_error "SurrealDB failed to start"
fi

if docker-compose ps | grep -q "searxng.*Up"; then
    print_success "Searxng is running"
else
    print_error "Searxng failed to start"
fi

if docker-compose ps | grep -q "ollama.*Up"; then
    print_success "Ollama is running"
else
    print_error "Ollama failed to start"
fi

# =============================================================================
# Ollama Models
# =============================================================================
print_header "Installing Ollama Models"

print_step "Pulling ministral:3b (fast model)..."
docker exec ollama ollama pull ministral:3b || print_warning "Failed to pull ministral:3b"

print_step "Pulling qwen2.5:7b (balanced model)..."
docker exec ollama ollama pull qwen2.5:7b || print_warning "Failed to pull qwen2.5:7b"

print_step "Pulling nomic-embed-text (embeddings)..."
docker exec ollama ollama pull nomic-embed-text || print_warning "Failed to pull nomic-embed-text"

print_success "Essential models installed"

print_step "Pulling qwen2.5-coder:14b (complex model - this may take a while)..."
docker exec ollama ollama pull qwen2.5-coder:14b || print_warning "Failed to pull qwen2.5-coder:14b (optional)"

# =============================================================================
# Frontend Dependencies
# =============================================================================
print_header "Installing Frontend Dependencies"

print_step "Installing User UI dependencies..."
cd neuro-ui
npm install
cd ..
print_success "User UI dependencies installed"

print_step "Installing Admin UI dependencies..."
cd neuro-admin
npm install
cd ..
print_success "Admin UI dependencies installed"

# =============================================================================
# Backend Build
# =============================================================================
print_header "Building Backend"

print_step "Building Rust backend (this may take a while on first run)..."
cd neuro-backend
cargo build --release
cd ..
print_success "Backend built successfully"

# =============================================================================
# Z-Brain CLI Build
# =============================================================================
print_header "Building Z-Brain CLI"

print_step "Building Z-Brain CLI..."
cd zbrain
cargo build --release
cd ..
print_success "Z-Brain CLI built successfully"

# =============================================================================
# Final Summary
# =============================================================================
print_header "Setup Complete! 🎉"

echo -e "
${GREEN}NEURO-OS has been set up successfully!${NC}

${CYAN}To start the system:${NC}

  ${YELLOW}1. Start the backend:${NC}
     cd neuro-backend && cargo run --release

  ${YELLOW}2. Start the User UI (in another terminal):${NC}
     cd neuro-ui && npm run dev
     → Open http://localhost:5173

  ${YELLOW}3. Start the Admin UI (in another terminal):${NC}
     cd neuro-admin && npm run dev
     → Open http://localhost:5174

  ${YELLOW}4. Use Z-Brain CLI:${NC}
     ./zbrain/target/release/zbrain

${CYAN}Or use the start script:${NC}
     ./start.sh

${CYAN}Service URLs:${NC}
  • User UI:    http://localhost:5173
  • Admin UI:   http://localhost:5174
  • Backend:    http://localhost:3000
  • SurrealDB:  http://localhost:8000
  • Searxng:    http://localhost:8080
  • Ollama:     http://localhost:11434

${CYAN}Useful commands:${NC}
  • Stop all:   docker-compose down
  • Logs:       docker-compose logs -f
  • Status:     docker-compose ps
"
