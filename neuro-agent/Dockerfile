# ==============================================================================
# Stage 1: Chef planner - Prepare dependency recipe
# ==============================================================================
FROM rust:1.85-slim AS chef
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN cargo install cargo-chef --locked
WORKDIR /app

# ==============================================================================
# Stage 2: Planner - Generate recipe.json
# ==============================================================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ==============================================================================
# Stage 3: Builder - Build dependencies (cached) then app
# ==============================================================================
FROM chef AS builder

# Build dependencies (cached if recipe.json hasn't changed)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# ==============================================================================
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
COPY --from=builder /app/target/release/neuro-agent /app/neuro-agent

# Default environment
ENV HOST=0.0.0.0
ENV PORT=3005
ENV SEARXNG_URL=http://searxng:8080
ENV RUST_LOG=info

EXPOSE 3005

CMD ["./neuro-agent"]
