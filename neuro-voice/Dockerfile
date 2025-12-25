# =============================================================================
# Voice Service Rust - Dockerfile
# =============================================================================
# Multi-stage build for the Piper TTS voice service in Rust.
# Includes NVIDIA GPU support for accelerated ONNX inference.
# Optimized with sccache + mold linker and cargo-chef for fast cached builds.
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

# Install build dependencies + mold linker
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libopus-dev \
    mold \
    clang \
    && rm -rf /var/lib/apt/lists/*

# Configure cargo: mold linker + parallel jobs
RUN mkdir -p /root/.cargo && \
    echo '[build]' > /root/.cargo/config.toml && \
    echo 'jobs = -1' >> /root/.cargo/config.toml && \
    echo '' >> /root/.cargo/config.toml && \
    echo '[target.x86_64-unknown-linux-gnu]' >> /root/.cargo/config.toml && \
    echo 'linker = "clang"' >> /root/.cargo/config.toml && \
    echo 'rustflags = ["-C", "link-arg=-fuse-ld=mold"]' >> /root/.cargo/config.toml

WORKDIR /app

# Copy recipe and build ONLY dependencies (this layer is cached!)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy source and build the actual app (only this recompiles on code changes)
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# -----------------------------------------------------------------------------
# Stage 4: Runtime image with NVIDIA GPU support
# -----------------------------------------------------------------------------
FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04

# Install runtime dependencies (including libopus for Opus audio codec)
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    ca-certificates \
    libssl3 \
    libgomp1 \
    libopus0 \
    curl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Download Piper TTS with CUDA support and Spanish voice model
RUN wget -q https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz \
    && tar -xzf piper_linux_x86_64.tar.gz \
    && rm piper_linux_x86_64.tar.gz \
    && chmod +x /app/piper/piper \
    && mkdir -p /app/models \
    && cd /app/models \
    && wget -q https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_MX/claude/high/es_MX-claude-high.onnx \
    && wget -q https://huggingface.co/rhasspy/piper-voices/resolve/main/es/es_MX/claude/high/es_MX-claude-high.onnx.json

# Copy the built binary from builder stage
COPY --from=builder /app/target/release/voice-service /app/voice-service

# Environment variables
ENV PIPER_BIN=/app/piper/piper
ENV MODELS_DIR=/app/models
ENV DEFAULT_VOICE=es_MX-claude-high
ENV HOST=0.0.0.0
ENV PORT=8100
ENV RUST_LOG=info,voice_service=debug

# Expose port
EXPOSE 8100

# Run the service
CMD ["/app/voice-service"]
