# =============================================================================
# Dockerfile for neuro-memory (Production)
# =============================================================================

# Stage 1: Chef planner
FROM rust:1.85-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# Stage 2: Prepare recipe
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Build dependencies
FROM chef AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Stage 4: Runtime
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/neuro-memory /app/neuro-memory
EXPOSE 3004
CMD ["/app/neuro-memory"]
