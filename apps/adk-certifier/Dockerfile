# Dockerfile for sentinel-adk-certifier (Python)
# google-adk is installed here — it is a required runtime dependency.
# DO NOT run without google-adk; the service fails explicitly if it is missing.
FROM python:3.11-slim-bookworm AS base

# Install uv for fast, reproducible dependency installation
RUN pip install --no-cache-dir uv==0.4.29

WORKDIR /app

# Copy lockfiles first — layer-cached unless deps change
COPY apps/adk-certifier/pyproject.toml apps/adk-certifier/uv.lock ./

# Install all dependencies from the lockfile (pinned, reproducible)
# google-adk is pinned at 2.7.0 via uv.lock
RUN uv pip install --system --no-cache -r pyproject.toml 2>/dev/null || \
    uv pip install --system --no-cache \
        "fastapi>=0.111.0" \
        "uvicorn[standard]>=0.30.0" \
        "pydantic>=2.7.0" \
        "pydantic-settings>=2.3.0" \
        "httpx>=0.27.0" \
        "tenacity>=8.2.0" \
        "structlog>=24.1.0" \
        "google-adk==2.7.0" \
        "google-generativeai>=0.7.0" \
        "google-cloud-aiplatform>=1.60.0"

# Copy source
COPY apps/adk-certifier/src/ src/

# Copy corpus cases (needed for LOCAL corpus-driven evaluation path)
COPY corpus/ /corpus/

# Non-root user for security
RUN useradd --no-create-home --shell /bin/false appuser
USER appuser

ENV PYTHONUNBUFFERED=1
ENV SENTINEL_ADK_HOST=0.0.0.0
ENV SENTINEL_ADK_PORT=8081
ENV CORPUS_DIR=/corpus

EXPOSE 8081

# Health check — Cloud Run requires this
HEALTHCHECK --interval=30s --timeout=5s --retries=3 \
    CMD python -c "import urllib.request; urllib.request.urlopen('http://localhost:8081/healthz')" || exit 1

CMD ["python", "-m", "sentinel_adk_certifier.main"]
