# Dockerfile for sentinel-workflow-worker
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

RUN cargo build --release --bin sentinel-workflow-worker

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sentinel-workflow-worker /usr/local/bin/sentinel-workflow-worker
COPY docker/config/workflow-worker.toml /app/config/workflow-worker.toml
COPY corpus/ /app/corpus/
COPY policy-packs/ /app/policy-packs/

ENV RUST_LOG=info
ENTRYPOINT ["sentinel-workflow-worker"]
