# Dockerfile for sentinel-control-plane
# Multi-stage build: builder (Rust) → minimal runtime image
FROM rust:1.82-slim-bookworm AS builder

WORKDIR /build

# Install system deps for linking
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

# Cache dependencies before copying source
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/control-plane/ apps/control-plane/
COPY apps/workflow-worker/ apps/workflow-worker/
COPY apps/sentinelctl/ apps/sentinelctl/
COPY apps/mock-erp-mcp/ apps/mock-erp-mcp/

RUN cargo build --release --bin sentinel-control-plane

# ── Runtime image ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sentinel-control-plane /usr/local/bin/sentinel-control-plane
COPY docker/config/control-plane.toml /app/config/control-plane.toml

# Cloud Run uses $PORT
ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["sentinel-control-plane", "--addr", "0.0.0.0:8080"]
