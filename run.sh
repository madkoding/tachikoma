#!/bin/bash
set -euo pipefail

# Tachikoma-OS runner
# Usage: ./run.sh [command] [args]
#
# Commands:
#   up              Build and start all services (default)
#   down            Stop all services
#   restart         Restart all services
#   rebuild [svc]   Rebuild specific service (or all)
#   logs [svc]      Tail logs (all or specific service)
#   status          Show service status
#   health          Check health of all services
#   clean           Stop and remove volumes (DESTRUCTIVE)
#   watch [svc]     Watch mode for hot-reload
#   shell <svc>     Open shell in running service
#   help            Show this help

COMPOSE_FILES="-f docker-compose.yml -f docker-compose.dev.yml"
PROJECT="tachikoma-os"

cmd="${1:-up}"
shift || true

case "$cmd" in
  up)
    docker compose $COMPOSE_FILES -p "$PROJECT" up -d --build
    echo ""
    echo "Services started. Run './run.sh status' to check or './run.sh logs' to tail."
    ;;
  down)
    docker compose $COMPOSE_FILES -p "$PROJECT" down
    ;;
  restart)
    docker compose $COMPOSE_FILES -p "$PROJECT" restart "${@:-}"
    ;;
  rebuild)
    if [ $# -gt 0 ]; then
      docker compose $COMPOSE_FILES -p "$PROJECT" up -d --build "$1"
    else
      docker compose $COMPOSE_FILES -p "$PROJECT" up -d --build
    fi
    ;;
  logs)
    docker compose $COMPOSE_FILES -p "$PROJECT" logs -f "${@:-}"
    ;;
  status)
    docker compose $COMPOSE_FILES -p "$PROJECT" ps
    ;;
  health)
    echo "Checking health of all services..."
    docker compose $COMPOSE_FILES -p "$PROJECT" ps --format json 2>/dev/null \
      | python3 -c "
import json, sys
healthy = unhealthy = 0
for line in sys.stdin:
    line = line.strip()
    if not line: continue
    try:
        svc = json.loads(line)
        name = svc.get('Service', '?')
        state = svc.get('Health', 'no healthcheck')
        if state == 'healthy':
            print(f'  OK  {name}')
            healthy += 1
        else:
            print(f'  --  {name} ({state})')
            unhealthy += 1
    except: pass
print(f'\n{healthy} healthy, {unhealthy} not healthy')
" 2>/dev/null || docker compose $COMPOSE_FILES -p "$PROJECT" ps
    ;;
  clean)
    echo "This will remove all volumes (database, downloads, etc)."
    read -p "Continue? [y/N] " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
      docker compose $COMPOSE_FILES -p "$PROJECT" down -v
    else
      echo "Aborted."
    fi
    ;;
  watch)
    docker compose $COMPOSE_FILES -p "$PROJECT" watch "${@:-}"
    ;;
  shell)
    if [ $# -lt 1 ]; then
      echo "Usage: ./run.sh shell <service>"
      exit 1
    fi
    docker compose $COMPOSE_FILES -p "$PROJECT" exec "$1" /bin/sh || \
      docker compose $COMPOSE_FILES -p "$PROJECT" exec "$1" /bin/bash
    ;;
  help|--help|-h)
    sed -n '2,18p' "$0"
    ;;
  *)
    echo "Unknown command: $cmd"
    echo "Run './run.sh help' for usage."
    exit 1
    ;;
esac
