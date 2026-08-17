#!/usr/bin/env bash
# =============================================================================
# Build Tauri sidecar binaries for the current host triple.
#
# Sidecars are plain Rust binaries from the workspace. Tauri requires them to
# be named `<bin>-<target-triple>` and placed in `src-tauri/binaries/`.
#
# Usage:
#   ./build-sidecars.sh [x86_64-unknown-linux-gnu]   # default = host triple
# =============================================================================
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
TAURI_DIR="$ROOT/apps/tachikoma-desktop/src-tauri"
BIN_DIR="$TAURI_DIR/binaries"

# Workspace crates to bundle as sidecars. Order matters: DB first, then backend,
# then the microservices. Voice is excluded (needs external Piper binary + models).
SIDECARS=(surrealdb tachikoma-backend tachikoma-checklists tachikoma-music tachikoma-chat tachikoma-memory tachikoma-agent)

TARGET="${1:-$(rustc -vV | sed -n 's/host: //p')}"

echo "Building sidecars for target: $TARGET"
mkdir -p "$BIN_DIR"

for name in "${SIDECARS[@]}"; do
  case "$name" in
    surrealdb)
      echo "  surrealdb: vendored binary required (see README). Copying if present..."
      if [ -f "$ROOT/vendor/surrealdb-$TARGET" ]; then
        cp "$ROOT/vendor/surrealdb-$TARGET" "$BIN_DIR/surrealdb-$TARGET"
      else
        echo "  WARN: no vendored surrealdb for $TARGET. Download from https://github.com/surrealdb/surrealdb/releases"
      fi
      ;;
    *)
      crate="${name#tachikoma-}"
      echo "  building workspace crate: $name (from neuro-$crate)"
      # The backend crate is named `tachikoma-backend` but lives in neuro-backend.
      if [ "$crate" = "backend" ]; then crate_dir="neuro-backend"; else crate_dir="neuro-$crate"; fi
      (cd "$ROOT/$crate_dir" && cargo build --release --target "$TARGET" --bin "$name")
      cp "$ROOT/target/$TARGET/release/$name" "$BIN_DIR/$name-$TARGET"
      ;;
  esac
done

echo "Done. Sidecars in $BIN_DIR:"
ls -la "$BIN_DIR"
