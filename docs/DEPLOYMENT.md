# Chimera Sentinel — GCP Deployment Guide

Everything you need to go from a fresh GCP project to a fully live deployment.
Follow these steps **in order**. Each step takes about 2–5 minutes.

---

## Prerequisites — install once

```bash
# 1. Google Cloud SDK
# https://cloud.google.com/sdk/docs/install
gcloud --version        # must be ≥ 455

# 2. Terraform
# https://developer.hashicorp.com/terraform/install
terraform --version     # must be ≥ 1.5

# 3. Docker Desktop (for building images)
docker --version

# 4. Rust (for local cargo check)
rustup --version

# 5. Node + pnpm
node --version && pnpm --version
```

---

## Step 1 — Create the GCP project

```bash
export PROJECT_ID=chimera-sentinel
export REGION=us-central1

# Set it as the active project
gcloud config set project $PROJECT_ID

# Link billing (required for Cloud Run, KMS, etc.)
# List billing accounts:
gcloud billing accounts list
# Link (replace BILLING_ACCOUNT_ID):
gcloud billing projects link $PROJECT_ID \
  --billing-account=BILLING_ACCOUNT_ID

# Authenticate Application Default Credentials
gcloud auth application-default login
```

---

## Step 2 — Bootstrap Terraform state bucket

```bash
# The backend is configured as chimera-sentinel-tf-state in main.tf.
gcloud storage buckets describe gs://chimera-sentinel-tf-state >/dev/null 2>&1 || \
  gcloud storage buckets create gs://chimera-sentinel-tf-state \
    --project=chimera-sentinel \
    --location=us-central1 \
    --uniform-bucket-level-access
```

---

## Step 3 — Configure terraform.tfvars

```bash
# Create your tfvars from the example
cp infra/terraform/terraform.tfvars.example infra/terraform/terraform.tfvars

# The example already targets project_id = "chimera-sentinel".
# Set hot_path_min_instances = 1 only for a live demo window; return it to 0 afterward.
# Leave managed agent vars empty for now — fill after Step 5
```

---

## Step 4 — First terraform apply (infrastructure only)

```bash
cd infra/terraform

terraform init

# Preview what will be created
terraform plan -var="project_id=$PROJECT_ID" -var="region=$REGION"

# Apply — this creates: Firestore, KMS, GCS, Artifact Registry, IAM, Cloud Run stubs
terraform apply -var="project_id=$PROJECT_ID" -var="region=$REGION"

cd ../..
```

Terraform creates and owns Firestore, KMS, GCS, Artifact Registry, IAM, and the
Cloud Run resources. Do not create Firestore separately with `gcloud`, or Terraform
will require an import before it can manage the database.

This takes about 3–5 minutes. At the end you'll see URLs for all services.

---

## Step 5 — Provision managed agent services

```bash
export KEY_RING=sentinel-ring
export ATTESTATION_KEY=attestation-signer
export SENTINEL_SA=sentinel-control-plane@${PROJECT_ID}.iam.gserviceaccount.com
export CANDIDATE_SA=sa-ap-candidate@${PROJECT_ID}.iam.gserviceaccount.com

bash infra/scripts/setup-managed-agents.sh
```

This creates: **Model Armor template**, **Agent Gateway**, **Memory Bank** (Vertex AI RAG Corpus),
**Agent Registry entry**, and **Agent Runtime**. It writes the resource names to
`infra/scripts/managed-resources.env`.

Now update your `terraform.tfvars` with the resource names:

```bash
source infra/scripts/managed-resources.env

# Add to infra/terraform/terraform.tfvars:
cat >> infra/terraform/terraform.tfvars <<EOF
model_armor_template   = "${MODEL_ARMOR_TEMPLATE}"
agent_gateway_resource = "${GATEWAY_RESOURCE}"
memory_bank_resource   = "${MEMORY_BANK_RESOURCE}"
EOF
```

---

## Step 6 — Build and push Docker images

```bash
export PROJECT_ID=chimera-sentinel
export REGION=us-central1

bash infra/scripts/deploy.sh
```

This builds all 5 Docker images (control-plane, workflow-worker, enterprise-erp-adapter,
adk-certifier, web), pushes them to Artifact Registry, and re-runs `terraform apply`
to deploy them to Cloud Run.

The web image is environment-agnostic. At runtime, the web service reads `API_URL`
and proxies same-origin `/api/*` requests to the control plane. No control-plane URL
is baked into browser JavaScript.

---

## Step 7 — Run smoke test

```bash
# Get the live control plane URL
SENTINEL_API_URL=$(terraform -chdir=infra/terraform output -raw control_plane_url)
echo "Control plane: $SENTINEL_API_URL"

# Health check
curl -sf "$SENTINEL_API_URL/healthz" | python3 -m json.tool

# Run full smoke test
make smoke-live SENTINEL_API_URL=$SENTINEL_API_URL GOOGLE_CLOUD_PROJECT=$PROJECT_ID
```

---

## Step 8 — Register a candidate and run certification

```bash
# Register a real AP agent candidate via the control plane API
cargo run --bin sentinelctl -- seed \
  --api-url "$SENTINEL_API_URL" \
  --tenant 00000000-0000-0000-0000-000000000001

# Open the web console
WEB_URL=$(terraform -chdir=infra/terraform output -raw web_url)
echo "Web console: $WEB_URL"
# Sign in and navigate to the Certification tab to start a certification run
```

---

## Step 9 — Run the release checklist

```bash
make release-check
```

---

## Quick reference: service URLs after deploy

```bash
terraform -chdir=infra/terraform output
```

| Output | Description |
|--------|-------------|
| `control_plane_url` | REST API — set as `SENTINEL_API_URL` |
| `web_url` | Public web console |
| `adk_certifier_url` | Internal ADK certifier |
| `erp_adapter_url` | Internal Enterprise ERP Adapter |
| `kms_key` | KMS key resource name |
| `evidence_bucket` | GCS evidence archive |

---

## Troubleshooting

**Cloud Run service fails to start (container crashed)**
```bash
gcloud run services logs read sentinel-control-plane --region=$REGION --project=$PROJECT_ID
```

**KMS signing fails**
- Verify the service account has `roles/cloudkms.signerVerifier` on the crypto key
- Check `SENTINEL_GOOGLE_CLOUD__KMS_KEY_RESOURCE` env var is set correctly in Cloud Run

**ADK certifier returns 502**
- Confirm `GOOGLE_CLOUD_PROJECT` and `GEMINI_MODEL_REF` are set in the Cloud Run service
- Check `MODEL_ARMOR_TEMPLATE` and `AGENT_GATEWAY_RESOURCE` are set to live resource names

**Terraform: "bucket does not exist"**
```bash
gsutil mb -p $PROJECT_ID -l $REGION gs://${PROJECT_ID}-tf-state
terraform -chdir=infra/terraform init -reconfigure \
  -backend-config="bucket=${PROJECT_ID}-tf-state"
```

**Docker push fails: "denied"**
```bash
gcloud auth configure-docker ${REGION}-docker.pkg.dev
```
