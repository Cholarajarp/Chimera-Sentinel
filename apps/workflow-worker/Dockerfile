# Dockerfile for sentinel-workflow-worker
# Base image tracks rust-toolchain.toml / workspace rust-version (1.92.0).
FROM rust:1.92-slim-bookworm AS builder

WORKDIR /build

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY apps/control-plane/ apps/control-plane/
COPY apps/workflow-worker/ apps/workflow-worker/
COPY apps/sentinelctl/ apps/sentinelctl/
COPY apps/enterprise-erp-adapter/ apps/enterprise-erp-adapter/

# -p is required: the app packages are not workspace default-members.
RUN cargo build --release -p sentinel-workflow-worker --bin sentinel-workflow-worker

FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sentinel-workflow-worker /usr/local/bin/sentinel-workflow-worker
COPY docker/config/workflow-worker.toml /app/config/workflow-worker.toml
COPY corpus/ /app/corpus/
COPY policy-packs/ /app/policy-packs/

ENV RUST_LOG=info
ENTRYPOINT ["sentinel-workflow-worker"]
