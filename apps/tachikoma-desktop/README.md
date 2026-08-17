# TACHIKOMA-OS Desktop

Tauri v2 shell. Bundles the Rust backend sidecars (SurrealDB + API gateway)
and the React frontend into a native app.

## Layout

```
tachikoma-desktop/
├── package.json            # npm scripts (tauri:dev/build)
├── src-tauri/
│   ├── src/lib.rs          # Tauri entry + SidecarManager (process supervision)
│   ├── src/hardware.rs     # Hardware detection for onboarding
│   └── binaries/           # bundled sidecar binaries (gitignored)
└── scripts/
    └── build-sidecars.sh   # builds sidecars for a target triple
```

## Build

```bash
# 1. Provide the vendored SurrealDB binary (required, not compiled from source)
mkdir -p vendor
curl -L -o /tmp/surreal.tgz https://github.com/surrealdb/surrealdb/releases/download/v3.2.0/surreal-v3.2.0.linux-amd64.tgz
tar xzf /tmp/surreal.tgz -C /tmp
cp /tmp/surreal vendor/surrealdb-x86_64-unknown-linux-gnu
chmod +x vendor/surrealdb-x86_64-unknown-linux-gnu

# 2. Build the sidecars (SurrealDB + backend + 5 microservices)
./scripts/build-sidecars.sh

# 3. Build the desktop app
npm install
npm run tauri:build
```

The `.deb`/`.rpm` bundles the main executable plus all sidecars.

## Sidecars

| Sidecar | Purpose | Port |
|---------|---------|------|
| surrealdb | Graph+vector DB | 8000 |
| tachikoma-backend | API gateway | 3000 |
| tachikoma-checklists | Checklist management | 3001 |
| tachikoma-music | YouTube music streaming | 3002 |
| tachikoma-chat | LLM conversations | 3003 |
| tachikoma-memory | GraphRAG semantic memory | 3004 |
| tachikoma-agent | AI agent tools | 3005 |

Sidecars are auto-restarted with backoff on crash. The `surrealdb` binary is
vendored (not compiled from source); see `build-sidecars.sh`.

**Excluded**: `tachikoma-voice` (needs the external Piper binary + voice models)
and Ollama (runs separately, see the `tachikoma-ollama` project). The backend
embeds a `VoiceEngine` for basic synthesis.
