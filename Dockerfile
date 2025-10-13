# Multi-stage Docker build for NMEA Parser CLI
FROM rust:1.75-slim as builder

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY . .

# Build the application in release mode
RUN cargo build --release --bin nmea-cli

# Runtime image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/nmea-cli /usr/local/bin/nmea-cli

# Create a non-root user
RUN useradd --create-home --shell /bin/bash nmea
USER nmea

# Expose default port (if running as a service)
EXPOSE 8080

# Health check using our built-in functionality
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD nmea-cli --health-check || exit 1

# Default command (can be overridden)
CMD ["nmea-cli", "--help"]