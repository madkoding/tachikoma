# =============================================================================
# Voice Service Rust - Dockerfile
# =============================================================================
# Multi-stage build for the Piper TTS voice service in Rust.
# Includes NVIDIA GPU support for accelerated ONNX inference.
# =============================================================================

# -----------------------------------------------------------------------------
# Stage 1: Build the Rust binary
# -----------------------------------------------------------------------------
FROM rust:1.83-slim AS builder

# Install build dependencies (including libopus for audio encoding)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    libssl-dev \
    libopus-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for dependency caching
COPY Cargo.toml Cargo.lock* ./

# Create dummy source to cache dependencies
RUN mkdir -p src && \
    echo "fn main() {}" > src/main.rs && \
    cargo build --release && \
    rm -rf src

# Copy actual source code
COPY src ./src

# Build the actual binary
RUN touch src/main.rs && cargo build --release

# -----------------------------------------------------------------------------
# Stage 2: Runtime image with NVIDIA GPU support
# -----------------------------------------------------------------------------
FROM nvidia/cuda:12.2.0-runtime-ubuntu22.04

# Install runtime dependencies (including libopus for Opus audio codec)
RUN apt-get update && apt-get install -y --no-install-recommends \
    wget \
    ca-certificates \
    libssl3 \
    libgomp1 \
    libopus0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Download Piper TTS with CUDA support
# Using the GPU-enabled version for NVIDIA acceleration
RUN wget -q https://github.com/rhasspy/piper/releases/download/2023.11.14-2/piper_linux_x86_64.tar.gz \
    && tar -xzf piper_linux_x86_64.tar.gz \
    && rm piper_linux_x86_64.tar.gz \
    && chmod +x /app/piper/piper

# Create models directory and download Spanish voice
RUN mkdir -p /app/models \
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
