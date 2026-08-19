# Dockerfile for sentinel-control-plane
# Multi-stage build: builder (Rust) → minimal runtime image
# Base image tracks rust-toolchain.toml / workspace rust-version (1.92.0).
FROM rust:1.92-slim-bookworm AS builder

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
COPY apps/enterprise-erp-adapter/ apps/enterprise-erp-adapter/

# -p is required: the app packages are not workspace default-members, so --bin
# alone resolves only against the default set.
RUN cargo build --release -p sentinel-control-plane --bin sentinel-control-plane

# ── Runtime image ─────────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /build/target/release/sentinel-control-plane /usr/local/bin/sentinel-control-plane
COPY docker/config/control-plane.toml /app/config/control-plane.toml

# The control plane digest-verifies the corpus at startup, so it must ship here.
COPY corpus/ /app/corpus/
COPY policy-packs/ /app/policy-packs/

# Cloud Run uses $PORT
ENV PORT=8080
EXPOSE 8080

ENTRYPOINT ["sentinel-control-plane", "--addr", "0.0.0.0:8080"]
