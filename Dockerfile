FROM alpine:latest

ARG TARGETARCH

RUN apk add --no-cache ca-certificates

WORKDIR /app

# Copy pre-built binary based on architecture
COPY target/x86_64-unknown-linux-musl/release/brave-search-mcp-server /usr/local/bin/brave-search-mcp-server-amd64
COPY target/aarch64-unknown-linux-musl/release/brave-search-mcp-server /usr/local/bin/brave-search-mcp-server-arm64

# Select the correct binary for this architecture
RUN if [ "$TARGETARCH" = "arm64" ]; then \
      ln -s /usr/local/bin/brave-search-mcp-server-arm64 /usr/local/bin/brave-search-mcp-server; \
    else \
      ln -s /usr/local/bin/brave-search-mcp-server-amd64 /usr/local/bin/brave-search-mcp-server; \
    fi

# Create non-root user
RUN adduser -D -u 1000 mcp && chown -R mcp:mcp /app
USER mcp

# Expose port
EXPOSE 3000

# Run the binary
ENTRYPOINT ["/usr/local/bin/brave-search-mcp-server"]
