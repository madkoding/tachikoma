# ============================================
# Stage 1: Chef planner
# ============================================
FROM rust:1.83-slim AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ============================================
# Stage 2: Prepare recipe
# ============================================
FROM chef AS planner
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

# ============================================
# Stage 3: Build dependencies (CACHED)
# ============================================
FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Build app
COPY Cargo.toml Cargo.lock* ./
COPY src ./src
RUN cargo build --release

# ============================================
# Stage 4: Runtime
# ============================================
FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/tachikoma-image /app/tachikoma-image

ENV PORT=3011
EXPOSE 3011

CMD ["/app/tachikoma-image"]
