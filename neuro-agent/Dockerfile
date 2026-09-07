# ==============================================================================
# Dockerfile for tachikoma-agent (Production)
# Build context: repository root (crate inherits workspace.dependencies)
# ==============================================================================

# Stage 1: Chef planner - Prepare dependency recipe
# ==============================================================================
FROM rust:1.98-slim AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Planner - Generate recipe.json
# ==============================================================================
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin tachikoma-agent

# Stage 3: Builder - Build dependencies (cached) then app
# ==============================================================================
FROM chef AS builder

# Build dependencies (cached if recipe.json hasn't changed)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin tachikoma-agent

# Build application
COPY . .
RUN cargo build --release --bin tachikoma-agent

# Stage 4: Runtime - Minimal production image
# ==============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    coreutils \
    findutils \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/tachikoma-agent /app/tachikoma-agent

# Default environment
ENV HOST=0.0.0.0
ENV PORT=3005
ENV SEARXNG_URL=http://searxng:8080
ENV RUST_LOG=info

EXPOSE 3005

CMD ["./tachikoma-agent"]
