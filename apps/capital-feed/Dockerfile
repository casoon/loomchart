# Build stage
FROM rust:1.83-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy workspace files
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY apps/capital-feed ./apps/capital-feed

# Build release binary
RUN cargo build --release --bin capital-feed

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder
COPY --from=builder /app/target/release/capital-feed /usr/local/bin/capital-feed

# Run as non-root
RUN useradd -m -u 1000 app
USER app

CMD ["capital-feed"]
