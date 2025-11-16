# Multi-stage build for Shazamq Operator
FROM rust:latest AS builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml ./

# Copy source
COPY src ./src

# Build (Cargo.lock will be generated)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user (group may exist, so use -g to specify system group)
RUN groupadd -g 1000 shazamq || true && \
    useradd -m -u 1000 -g 1000 shazamq

# Copy binary
COPY --from=builder /app/target/release/shazamq-operator /usr/local/bin/shazamq-operator

# Set ownership
RUN chown shazamq:shazamq /usr/local/bin/shazamq-operator

USER shazamq

ENTRYPOINT ["/usr/local/bin/shazamq-operator"]

