# =============================================================================
# Dockerfile for tachikoma-memory (Production)
# Build context: repository root (crate inherits workspace.dependencies)
# =============================================================================

# Stage 1: Chef planner
FROM rust:1.98-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Prepare recipe
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin tachikoma-memory

# Stage 3: Build dependencies
FROM chef AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin tachikoma-memory

# Build application
COPY . .
RUN cargo build --release --bin tachikoma-memory

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/tachikoma-memory /app/tachikoma-memory
EXPOSE 3004
CMD ["/app/tachikoma-memory"]
