# =============================================================================
# Checklists Service Rust - Dockerfile
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
RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy recipe and build ONLY dependencies (this layer is cached!)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Now copy source and build the actual app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies (curl needed for Docker healthcheck)
RUN apt-get update && apt-get install -y ca-certificates curl && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/tachikoma-checklists /app/tachikoma-checklists

# Set environment
ENV RUST_LOG=info
ENV PORT=3001

EXPOSE 3001

CMD ["/app/tachikoma-checklists"]
