#!/bin/bash
set -euo pipefail

# Dev helper for Tachikoma-OS
# Usage: ./dev.sh [rebuild|clean|up|down|watch]

case "${1:-up}" in
  up)
    docker compose up -d --build
    ;;
  rebuild)
    docker compose up -d --build "$2"
    ;;
  clean)
    docker compose down -v
    ;;
  down)
    docker compose down
    ;;
  watch)
    docker compose watch "$2"
    ;;
  *)
    echo "Usage: $0 [up|rebuild <service>|clean|down|watch <service>]"
    exit 1
    ;;
esac