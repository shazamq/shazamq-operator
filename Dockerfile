# Multi-stage build for Shazamq Operator
FROM rust:1.75-slim as builder

# Install build dependencies
RUN apt-get update && \
    apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src

# Build
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m -u 1000 operator

# Copy binary
COPY --from=builder /app/target/release/shazamq-operator /usr/local/bin/shazamq-operator

# Set ownership
RUN chown operator:operator /usr/local/bin/shazamq-operator

USER operator

ENTRYPOINT ["/usr/local/bin/shazamq-operator"]

