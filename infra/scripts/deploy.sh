#!/usr/bin/env bash
# infra/scripts/deploy.sh
#
# One-shot deploy: build all Docker images, push to Artifact Registry,
# then run terraform apply.
#
# Prerequisites:
#   - gcloud auth login && gcloud auth configure-docker $REGION-docker.pkg.dev
#   - terraform installed
#   - terraform.tfvars filled in (copy from terraform.tfvars.example)
#
# Usage:
#   export PROJECT_ID=my-project REGION=us-central1
#   bash infra/scripts/deploy.sh

set -euo pipefail

: "${PROJECT_ID:?Required — export PROJECT_ID=your-project-id}"
: "${REGION:=us-central1}"

REPO="${REGION}-docker.pkg.dev/${PROJECT_ID}/sentinel"
GIT_SHA=$(git rev-parse --short HEAD 2>/dev/null || echo "local")

echo "=== Chimera Sentinel Deploy ==="
echo "  Project:  $PROJECT_ID"
echo "  Region:   $REGION"
echo "  Registry: $REPO"
echo "  Commit:   $GIT_SHA"
echo ""

# ── 0. Bootstrap state and configuration ─────────────────────────────────────
echo "[0/8] Bootstrapping Terraform state and configuration..."
gcloud storage buckets describe "gs://${PROJECT_ID}-tf-state" >/dev/null 2>&1 || \
  gcloud storage buckets create "gs://${PROJECT_ID}-tf-state" \
    --project="${PROJECT_ID}" \
    --location="${REGION}" \
    --uniform-bucket-level-access

if [ ! -f infra/terraform/terraform.tfvars ]; then
  cp infra/terraform/terraform.tfvars.example infra/terraform/terraform.tfvars
  echo "Created terraform.tfvars from example. Edit it to add managed agent resource names."
fi

terraform -chdir=infra/terraform init -reconfigure \
  -backend-config="bucket=${PROJECT_ID}-tf-state"

echo "[1/8] Bootstrapping APIs and Artifact Registry..."
terraform -chdir=infra/terraform apply -auto-approve \
  -target='google_project_service.apis' \
  -target='google_artifact_registry_repository.sentinel' \
  -var="project_id=${PROJECT_ID}" \
  -var="region=${REGION}"

echo "[2/8] Authenticating Docker to Artifact Registry..."
gcloud auth configure-docker "${REGION}-docker.pkg.dev" --quiet

# ── 1. Build & push control-plane ─────────────────────────────────────────────
echo "[3/8] Building sentinel-control-plane..."
docker build \
  -f apps/control-plane/Dockerfile \
  -t "${REPO}/control-plane:${GIT_SHA}" \
  -t "${REPO}/control-plane:latest" \
  .
docker push "${REPO}/control-plane:${GIT_SHA}"
docker push "${REPO}/control-plane:latest"

# ── 2. Build & push workflow-worker ───────────────────────────────────────────
echo "[4/8] Building sentinel-workflow-worker..."
docker build \
  -f apps/workflow-worker/Dockerfile \
  -t "${REPO}/workflow-worker:${GIT_SHA}" \
  -t "${REPO}/workflow-worker:latest" \
  .
docker push "${REPO}/workflow-worker:${GIT_SHA}"
docker push "${REPO}/workflow-worker:latest"

# ── 3. Build & push mock-erp-mcp ──────────────────────────────────────────────
echo "[5/8] Building sentinel-mock-erp-mcp..."
docker build \
  -f apps/mock-erp-mcp/Dockerfile \
  -t "${REPO}/mock-erp-mcp:${GIT_SHA}" \
  -t "${REPO}/mock-erp-mcp:latest" \
  .
docker push "${REPO}/mock-erp-mcp:${GIT_SHA}"
docker push "${REPO}/mock-erp-mcp:latest"

# ── 4. Build & push adk-certifier ─────────────────────────────────────────────
echo "[6/8] Building sentinel-adk-certifier..."
docker build \
  -f apps/adk-certifier/Dockerfile \
  -t "${REPO}/adk-certifier:${GIT_SHA}" \
  -t "${REPO}/adk-certifier:latest" \
  .
docker push "${REPO}/adk-certifier:${GIT_SHA}"
docker push "${REPO}/adk-certifier:latest"

# ── 5. Build & push web console ───────────────────────────────────────────────
echo "[7/8] Building sentinel-web..."
docker build \
  -f apps/web/Dockerfile \
  -t "${REPO}/web:${GIT_SHA}" \
  -t "${REPO}/web:latest" \
  .
docker push "${REPO}/web:${GIT_SHA}"
docker push "${REPO}/web:latest"

# ── 8. Terraform apply ────────────────────────────────────────────────────────
echo "[8/8] Running terraform apply..."

cd infra/terraform

terraform init -reconfigure \
  -backend-config="bucket=${PROJECT_ID}-tf-state"

terraform apply -auto-approve \
  -var="project_id=${PROJECT_ID}" \
  -var="region=${REGION}"

cd ../..

# ── Output URLs ───────────────────────────────────────────────────────────────
echo ""
echo "=== Deploy Complete ==="
echo ""
echo "Service URLs:"
terraform -chdir=infra/terraform output -raw control_plane_url 2>/dev/null | xargs -I{} echo "  Control Plane: {}"
terraform -chdir=infra/terraform output -raw web_url 2>/dev/null | xargs -I{} echo "  Web Console:   {}"
terraform -chdir=infra/terraform output -raw adk_certifier_url 2>/dev/null | xargs -I{} echo "  ADK Certifier: {}"
echo ""
echo "Next steps:"
echo "  1. bash infra/scripts/setup-managed-agents.sh"
echo "  2. Fill in managed resource names in infra/terraform/terraform.tfvars"
echo "  3. Re-run: bash infra/scripts/deploy.sh"
echo "  4. make smoke-live SENTINEL_API_URL=\$(terraform -chdir=infra/terraform output -raw control_plane_url)"
