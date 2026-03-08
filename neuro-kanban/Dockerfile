# =============================================================================
# Stage 1: Chef - Install cargo-chef
# =============================================================================
FROM rust:1.83-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# =============================================================================
# Stage 2: Planner - Prepare recipe
# =============================================================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# =============================================================================
# Stage 3: Builder - Build dependencies (cached) and app
# =============================================================================
FROM chef AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy and cook recipe (dependencies - this is cached!)
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build application
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# =============================================================================
# Stage 4: Runtime - Minimal image
# =============================================================================
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/tachikoma-kanban /app/tachikoma-kanban

# Create non-root user
RUN useradd -r -s /bin/false appuser && \
    chown -R appuser:appuser /app
USER appuser

# Expose port
EXPOSE 3006

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3006/api/health || exit 1

# Run
CMD ["./tachikoma-kanban"]
