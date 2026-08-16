# Dockerfile for sentinel-mock-erp-mcp
FROM rust:1.82-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/control-plane/ apps/control-plane/
COPY apps/workflow-worker/ apps/workflow-worker/
COPY apps/sentinelctl/ apps/sentinelctl/
COPY apps/mock-erp-mcp/ apps/mock-erp-mcp/

RUN cargo build --release --bin sentinel-mock-erp-mcp

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sentinel-mock-erp-mcp /usr/local/bin/sentinel-mock-erp-mcp
COPY docker/config/mock-erp-mcp.toml /app/config/mock-erp-mcp.toml

ENV PORT=9090
EXPOSE 9090

ENTRYPOINT ["sentinel-mock-erp-mcp", "--addr", "0.0.0.0:9090"]
