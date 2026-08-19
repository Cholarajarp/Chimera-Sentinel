#!/usr/bin/env bash
# infra/scripts/setup-managed-agents.sh
#
# Provisions all Google Cloud managed agent services required by Chimera Sentinel.
# Must be run ONCE per environment after `terraform apply`.
#
# Required env vars:
#   PROJECT_ID          — GCP project (e.g. sentinel-prod)
#   REGION              — GCP region  (e.g. us-central1)
#   KEY_RING            — KMS key ring name
#   ATTESTATION_KEY     — KMS asymmetric signing key name
#   SENTINEL_SA         — Sentinel control plane service account email
#   CANDIDATE_SA        — AP agent candidate service account email
#
# Usage:
#   export PROJECT_ID=sentinel-prod REGION=us-central1 ...
#   bash infra/scripts/setup-managed-agents.sh

set -euo pipefail

: "${PROJECT_ID:?Required}"
: "${REGION:?Required}"
: "${KEY_RING:=sentinel-ring}"
: "${ATTESTATION_KEY:=attestation-signer}"
: "${SENTINEL_SA:=sentinel-control-plane@${PROJECT_ID}.iam.gserviceaccount.com}"
: "${CANDIDATE_SA:=sa-ap-candidate@${PROJECT_ID}.iam.gserviceaccount.com}"

ARMOR_TEMPLATE_ID="ap-agent-armor-v1"
GATEWAY_ID="ap-agent-gateway"
MEMORY_BANK_ID="ap-agent-memory-bank"
AGENT_REGISTRY_ID="ap-agent-prod"
AGENT_RUNTIME_ID="ap-runtime-01"

echo "=== Chimera Sentinel — Managed Agent Services Setup ==="
echo "  Project:  $PROJECT_ID"
echo "  Region:   $REGION"
echo ""

# ── 1. Model Armor Template ───────────────────────────────────────────────────
echo "[1/6] Creating Model Armor template: $ARMOR_TEMPLATE_ID"
gcloud model-armor templates create "$ARMOR_TEMPLATE_ID" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --filter-config='{
    "raiSettings": {
      "raiFilters": [
        {"filterType": "HATE_SPEECH", "confidenceLevel": "LOW_AND_ABOVE"},
        {"filterType": "SEXUALLY_EXPLICIT", "confidenceLevel": "LOW_AND_ABOVE"},
        {"filterType": "HARASSMENT", "confidenceLevel": "LOW_AND_ABOVE"},
        {"filterType": "DANGEROUS_CONTENT", "confidenceLevel": "LOW_AND_ABOVE"}
      ]
    },
    "piAndJailbreakFilterSettings": {
      "filterEnforcement": "ENABLED",
      "confidenceLevel": "LOW_AND_ABOVE"
    }
  }' \
  --description="Chimera Sentinel AP Agent content safety template" \
  || echo "  (Template may already exist — continuing)"

ARMOR_TEMPLATE_RESOURCE="projects/${PROJECT_ID}/locations/${REGION}/templates/${ARMOR_TEMPLATE_ID}"
echo "  Template resource: $ARMOR_TEMPLATE_RESOURCE"

# ── 2. Agent Gateway ──────────────────────────────────────────────────────────
echo ""
echo "[2/6] Creating Agent Gateway: $GATEWAY_ID"
gcloud agent-gateway gateways create "$GATEWAY_ID" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --description="Chimera Sentinel AP agent least-privilege gateway" \
  || echo "  (Gateway may already exist — continuing)"

# Apply tool-scoped deny policy: block release_payment for candidate identity
echo "  Applying gateway policy: deny release_payment for $CANDIDATE_SA"
gcloud agent-gateway gateways set-iam-policy "$GATEWAY_ID" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --policy-file=- <<EOF
{
  "version": 1,
  "bindings": [
    {
      "role": "roles/agentgateway.toolCaller",
      "members": ["serviceAccount:${CANDIDATE_SA}"],
      "condition": {
        "title": "allow_draft_only",
        "expression": "request.tool_name in ['draft_invoice_payment', 'get_payment_status']"
      }
    }
  ]
}
EOF

GATEWAY_RESOURCE="projects/${PROJECT_ID}/locations/${REGION}/gateways/${GATEWAY_ID}"
echo "  Gateway resource: $GATEWAY_RESOURCE"

# ── 3. Memory Bank (Vertex AI RAG Corpus) ─────────────────────────────────────
echo ""
echo "[3/6] Creating Memory Bank (Vertex AI RAG Corpus): $MEMORY_BANK_ID"
CORPUS_NAME=$(gcloud ai rag-corpora create \
  --project="$PROJECT_ID" \
  --region="$REGION" \
  --display-name="Chimera Sentinel AP Agent Memory Bank" \
  --format="value(name)" \
  2>/dev/null) || CORPUS_NAME=""

if [ -z "$CORPUS_NAME" ]; then
  # Corpus already exists — look it up
  CORPUS_NAME=$(gcloud ai rag-corpora list \
    --project="$PROJECT_ID" \
    --region="$REGION" \
    --filter="displayName=Chimera Sentinel AP Agent Memory Bank" \
    --format="value(name)" | head -1)
fi

echo "  Memory bank resource: $CORPUS_NAME"
MEMORY_BANK_RESOURCE="$CORPUS_NAME"

# ── 4. Agent Registry Entry ───────────────────────────────────────────────────
echo ""
echo "[4/6] Registering agent in Agent Registry: $AGENT_REGISTRY_ID"
gcloud agent-registry agents create "$AGENT_REGISTRY_ID" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --display-name="AP Autonomous Agent (Chimera Sentinel)" \
  --description="Accounts Payable autonomous agent under Fortified Fleet admission control" \
  || echo "  (Agent may already exist — continuing)"

AGENT_RESOURCE="projects/${PROJECT_ID}/locations/${REGION}/agents/${AGENT_REGISTRY_ID}"
echo "  Agent resource: $AGENT_RESOURCE"

# ── 5. Agent Runtime ──────────────────────────────────────────────────────────
echo ""
echo "[5/6] Creating Agent Runtime: $AGENT_RUNTIME_ID"
gcloud agent-gateway runtimes create "$AGENT_RUNTIME_ID" \
  --project="$PROJECT_ID" \
  --location="$REGION" \
  --description="Chimera Sentinel AP agent runtime environment" \
  || echo "  (Runtime may already exist — continuing)"

RUNTIME_RESOURCE="projects/${PROJECT_ID}/locations/${REGION}/runtimes/${AGENT_RUNTIME_ID}"
echo "  Runtime resource: $RUNTIME_RESOURCE"

# ── 6. Output env config ──────────────────────────────────────────────────────
echo ""
echo "[6/6] Writing managed resource references to infra/scripts/managed-resources.env"
cat > infra/scripts/managed-resources.env <<ENVFILE
# Auto-generated by infra/scripts/setup-managed-agents.sh
# Source this file before running smoke tests or deploying services.
GOOGLE_CLOUD_PROJECT=${PROJECT_ID}
GOOGLE_CLOUD_REGION=${REGION}
MODEL_ARMOR_TEMPLATE=${ARMOR_TEMPLATE_RESOURCE}
AGENT_GATEWAY_RESOURCE=${GATEWAY_RESOURCE}
MEMORY_BANK_RESOURCE=${MEMORY_BANK_RESOURCE}
AGENT_RESOURCE=${AGENT_RESOURCE}
RUNTIME_RESOURCE=${RUNTIME_RESOURCE}
KMS_KEY_RESOURCE=projects/${PROJECT_ID}/locations/global/keyRings/${KEY_RING}/cryptoKeys/${ATTESTATION_KEY}
ENVFILE

echo ""
echo "=== Setup complete ==="
echo "Source the env file and redeploy Cloud Run services to pick up the new resource references:"
echo "  source infra/scripts/managed-resources.env"
echo "  terraform -chdir=infra/terraform apply"
