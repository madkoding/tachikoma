# =============================================================================
# Dockerfile for tachikoma-chat (Production)
# Build context: repository root (crate inherits workspace.dependencies)
# =============================================================================

FROM rust:1.98-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json --bin tachikoma-chat

FROM chef AS builder
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json --bin tachikoma-chat

COPY . .
RUN cargo build --release --bin tachikoma-chat

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/tachikoma-chat /app/tachikoma-chat
EXPOSE 3003
CMD ["/app/tachikoma-chat"]
