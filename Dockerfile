# Build stage
FROM rust:1.83-alpine AS builder

RUN apk add --no-cache musl-dev openssl-dev pkgconfig

WORKDIR /app

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies
RUN cargo build --release

# Copy actual source code
COPY src ./src

# Build the application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM alpine:3.19

RUN apk add --no-cache ca-certificates openssl

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/brave-search-mcp-server /usr/local/bin/

# Create non-root user
RUN adduser -D -u 1000 mcp && chown -R mcp:mcp /app
USER mcp

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:3000/health || exit 1

# Run the binary
ENTRYPOINT ["/usr/local/bin/brave-search-mcp-server"]
