# =============================================================================
# Music Service Rust - Dockerfile
# =============================================================================
# Multi-stage build optimized with cargo-chef for faster cached builds.
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Chef planner - analyze dependencies
# -----------------------------------------------------------------------------
FROM rust:1.86-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# -----------------------------------------------------------------------------
# Stage 2: Prepare recipe (dependency lockfile)
# -----------------------------------------------------------------------------
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# -----------------------------------------------------------------------------
# Stage 3: Build dependencies (cached layer)
# -----------------------------------------------------------------------------
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy recipe and build ONLY dependencies (this layer is cached!)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy source and build the actual app (only this recompiles on code changes)
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Runtime stage - use same base as builder to avoid glibc issues
FROM debian:trixie-slim

# Install runtime dependencies and yt-dlp
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3t64 \
    ffmpeg \
    python3 \
    python3-pip \
    curl \
    && pip3 install --break-system-packages yt-dlp \
    && rm -rf /var/lib/apt/lists/* \
    && useradd -ms /bin/bash neuro

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/neuro-music /app/neuro-music

# Set ownership
RUN chown -R neuro:neuro /app

USER neuro

# Environment variables with defaults
ENV MUSIC_PORT=3002 \
    DATABASE_URL=ws://surrealdb:8000 \
    DATABASE_USER=root \
    DATABASE_PASS=neuroos_secret_2024 \
    DATABASE_NAMESPACE=music \
    DATABASE_NAME=music \
    YTDLP_PATH=/usr/local/bin/yt-dlp \
    FFMPEG_PATH=/usr/bin/ffmpeg \
    RUST_LOG=info

EXPOSE 3002

CMD ["./neuro-music"]
