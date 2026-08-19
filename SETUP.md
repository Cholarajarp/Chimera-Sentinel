# Chimera Sentinel — Setup & End-to-End Input Flow

This document explains **how the system works end-to-end**: what input it takes, where that input goes, which services process it, and what the system produces. Read this before deploying or operating Sentinel in production.

---

## Table of Contents

1. [What Is the Input?](#1-what-is-the-input)
2. [How a Certification Run Works](#2-how-a-certification-run-works)
3. [Service Map](#3-service-map)
4. [Running Locally (No GCP)](#4-running-locally-no-gcp)
5. [Running Against the Real GCP Stack](#5-running-against-the-real-gcp-stack)
6. [Environment Variables That Matter](#6-environment-variables-that-matter)
7. [How the Web Console Connects to the Backend](#7-how-the-web-console-connects-to-the-backend)
8. [How to Submit a Real Agent for Certification](#8-how-to-submit-a-real-agent-for-certification)
9. [How to Approve and Get an Attestation](#9-how-to-approve-and-get-an-attestation)
10. [Trust Boundaries and What Each Layer Can and Cannot Do](#10-trust-boundaries)

---

## 1. What Is the Input?

Sentinel's primary input is an **Agent Bill of Materials (ABOM)** — a content-addressed, immutable manifest that binds a specific candidate agent revision to all the resources it requires to run:

| Field | Meaning |
|---|---|
| `source_digest` | SHA-256 of the agent's source code or container image |
| `registry_resource` | Artifact Registry image path (e.g. `us-central1-docker.pkg.dev/project/repo/agent:sha`) |
| `runtime_resource` | Agent Runtime resource name (e.g. `projects/p/locations/l/services/agent-rev`) |
| `agent_identity` | IAM service account (e.g. `agent-v2@project.iam.gserviceaccount.com`) |
| `model_ref` | Vertex AI model name (e.g. `gemini-2.5-flash-preview-05-20`) |
| `prompt_config_digest` | SHA-256 of the prompt configuration |
| `tool_manifest_digest` | SHA-256 of the declared tool set |
| `memory_config_digest` | SHA-256 of the Memory Bank configuration |
| `gateway_policy_digest` | SHA-256 of the Agent Gateway policy |
| `model_armor_config_digest` | SHA-256 of the Model Armor template configuration |
| `requested_capabilities` | Tools the agent claims to need (e.g. `["draft_invoice_payment", "read_customer_record"]`) |
| `data_classification` | Data tiers the agent will access (e.g. `["PII", "FINANCIAL"]`) |
| `environment` | `production` or `staging` |
| `owner` | Team email address responsible for this agent |
| `risk_tier` | `HIGH`, `MEDIUM`, or `LOW` |

**You register a candidate by POSTing this manifest to the control plane API.** The control plane stores the ABOM in Firestore and returns a `revision_id` you use in subsequent calls.

---

## 2. How a Certification Run Works

```
Platform Engineer
        │
        │  POST /v1/candidates          (register ABOM)
        ▼
┌─────────────────────┐
│  Rust Control Plane │  ◄── Firestore (durable state)
│  (Axum / Cloud Run) │  ◄── Cloud KMS (signing)
└──────────┬──────────┘
           │  POST /v1/workflows         (start certification)
           │
           ▼
┌─────────────────────┐
│  Rust Workflow      │
│  Worker             │  Reads workflow from Firestore
└──────────┬──────────┘
           │  POST /evaluate             (80-case batch)
           ▼
┌─────────────────────────────────────────────────────┐
│  Python ADK Certifier (FastAPI / Cloud Run)         │
│                                                     │
│  For each of 80 corpus cases:                       │
│    1. Model Armor   — content inspection            │
│    2. Agent Gateway — identity-scoped tool call     │
│    3. ERP Oracle    — ledger snapshot delta check   │
│    4. Gemini        — structured observation        │
│                                                     │
│  Returns: typed CaseObservation[]                   │
│  (no admission authority — observations only)       │
└──────────┬──────────────────────────────────────────┘
           │  Typed observations returned to Worker
           ▼
┌─────────────────────┐
│  Rust Policy Engine │  Deterministic decision: APPROVED / BLOCKED
│  (crates/policy)    │  Produces structured explanation
└──────────┬──────────┘
           │
           ▼  state → APPROVAL_REQUIRED
           │
           │  POST /v1/workflows/{id}/approvals   (human reviewer)
           ▼
┌─────────────────────┐
│  Rust Control Plane │  Separation-of-duties enforcement:
│                     │  candidate owner CANNOT self-approve
└──────────┬──────────┘
           │
           ▼  state → ATTESTING
           │
           │  Cloud KMS SignAsymmetric
           ▼
┌─────────────────────┐
│  RFC 8785 Canonical │  RSA-PSS-2048-SHA256 signed attestation
│  Attestation        │  expires_at = now + 90 days
│  (crates/attestation│  stored in Firestore + GCS
└─────────────────────┘
           │
           ▼  state → CERTIFIED
```

The attestation is independently verifiable offline with:

```bash
cargo run --bin sentinelctl -- attestation-verify \
  --file path/to/attestation.json \
  --tenant <tenant-uuid> \
  --environment production
```

---

## 3. Service Map

| Service | Language | Port | Deployed as | Responsibility |
|---|---|---|---|---|
| **control-plane** | Rust (Axum) | 8080 | Cloud Run | AuthN, AuthZ, tenancy, ABOM storage, workflow state machine, KMS signing |
| **workflow-worker** | Rust (async) | — | Cloud Run Job | Drives workflow state transitions, calls ADK certifier, runs policy engine |
| **adk-certifier** | Python (FastAPI) | 8081 | Cloud Run | Orchestrates Gemini case analysis, Model Armor, Agent Gateway, ERP oracle |
| **enterprise-erp-adapter** | Rust (MCP) | 9090 | Cloud Run | Deterministic ledger oracle — verifies zero unauthorized side effects |
| **web** | Next.js (TypeScript) | 3000 | Cloud Run | Enterprise console UI — purely a view/controller, no canonical state |

All five services are deployed via Terraform (`infra/terraform/main.tf`) and communicate over Cloud Run internal networking with OIDC identity tokens.

---

## 4. Running Locally (No GCP)

Use this path to develop UI changes or explore the API without standing up the full GCP stack.

```bash
# 1. Install dependencies
make bootstrap

# 2. Copy environment config
cp .env.example .env
# Leave API_URL commented out for local mode

# 3. Start services (separate terminals)
make run-mcp        # ERP adapter    → :9090
make run-api        # Control plane  → :8080
make run-worker     # Workflow worker
make run-certifier  # ADK certifier  → :8081
make run-web        # Web console    → :3000

# Or with Docker Compose:
docker compose up --build
```

Open `http://localhost:3000`.

**What happens locally when `API_URL` is not set:**
- The Next.js web console handles all API calls through an in-process fallback module (`apps/web/src/lib/control-plane.ts`)
- State lives in process-local `Map`s — it resets on server restart
- The "signature" in attestations is a SHA-256 stand-in, not a real KMS signature
- Every response carries `provenance: "LOCAL_FALLBACK"` so you cannot mistake it for production data
- No real Gemini, Model Armor, Agent Gateway, or ERP oracle calls are made

This mode is **only for UI development**. To test the real certification pipeline, see Section 5.

---

## 5. Running Against the Real GCP Stack

### Prerequisites

- GCP project with billing enabled
- `gcloud` CLI authenticated: `gcloud auth application-default login`
- Terraform ≥ 1.5.0
- Docker (for building images)

### Deploy

```bash
export PROJECT_ID=your-gcp-project-id
export REGION=us-central1

gcloud config set project $PROJECT_ID
gcloud auth configure-docker ${REGION}-docker.pkg.dev

# Create Terraform state bucket (one-time)
gcloud storage buckets create gs://${PROJECT_ID}-tf-state \
  --project=$PROJECT_ID --location=$REGION --uniform-bucket-level-access

# Configure and deploy all 5 services
cp infra/terraform/terraform.tfvars.example infra/terraform/terraform.tfvars
# Edit terraform.tfvars: set project_id = "$PROJECT_ID"

terraform -chdir=infra/terraform init
terraform -chdir=infra/terraform apply

# Set up Model Armor, Agent Gateway, Memory Bank, and Agent Identity
bash infra/scripts/setup-managed-agents.sh
```

### Get Service URLs

```bash
terraform -chdir=infra/terraform output
# Outputs:
#   control_plane_url  = "https://sentinel-control-plane-HASH.run.app"
#   web_url            = "https://sentinel-web-HASH.run.app"
#   adk_certifier_url  = "https://sentinel-adk-certifier-HASH.run.app"
#   kms_key            = "projects/.../cryptoKeys/attestation-signer"
#   evidence_bucket    = "gs://chimera-sentinel-evidence-..."
```

### Verify the Stack

```bash
export SENTINEL_API_URL=$(terraform -chdir=infra/terraform output -raw control_plane_url)

# Health check
curl -sf "$SENTINEL_API_URL/healthz" | python3 -m json.tool

# Full smoke test
make smoke-live SENTINEL_API_URL=$SENTINEL_API_URL GOOGLE_CLOUD_PROJECT=$PROJECT_ID
```

When the web console is deployed on Cloud Run, `API_URL` is already wired to the control plane URL by Terraform. Every API request the browser makes goes through the Next.js server, which proxies it to the Rust control plane with a Cloud Run OIDC identity token.

---

## 6. Environment Variables That Matter

### Web Console (Next.js — Cloud Run)

| Variable | Where Set | Purpose |
|---|---|---|
| `API_URL` | Terraform / Cloud Run env | **Server-side.** URL the Next.js proxy sends every API request to. When set → all data comes from the Rust control plane. When absent → in-process fallback (local mode). |
| `NEXTAUTH_SECRET` | Secret Manager | Session cookie signing secret. |
| `NEXTAUTH_URL` | Terraform | Public URL of the web console. |

> **`NEXT_PUBLIC_API_URL` is not needed.** The console uses server-side proxying exclusively — no client-side API calls bypass the Next.js server.

### Control Plane (Rust — Cloud Run)

| Variable | Purpose |
|---|---|
| `SENTINEL_ENVIRONMENT` | `production` or `staging` — controls Firestore vs in-memory store |
| `GOOGLE_CLOUD_PROJECT` | GCP project ID |
| `KMS_KEY_RESOURCE` | Full KMS key resource name for attestation signing |
| `SENTINEL_DEFAULT_TENANT` | Default tenant UUID |

### ADK Certifier (Python — Cloud Run)

| Variable | Purpose |
|---|---|
| `GEMINI_MODEL_REF` | Vertex AI model (e.g. `gemini-2.5-flash-preview-05-20`) |
| `MODEL_ARMOR_TEMPLATE` | Full resource name of the Model Armor template |
| `AGENT_GATEWAY_RESOURCE` | Full resource name of the Agent Gateway |
| `MEMORY_BANK_RESOURCE` | RAG corpus resource name for Memory Bank |
| `ERP_MCP_URL` | Internal URL of the Enterprise ERP Adapter |
| `GOOGLE_CLOUD_PROJECT` | GCP project ID |

---

## 7. How the Web Console Connects to the Backend

The web console is a **pure view/controller**. It never owns canonical state.

```
Browser
  │
  │  fetch("/api/v1/fleet/posture")
  ▼
Next.js Server (apps/web)
  │
  │  [apps/web/src/app/api/[...path]/route.ts]
  │
  ├── If API_URL is set (production):
  │     Forward request → Rust control plane (with OIDC token)
  │     Stream response back → browser
  │
  └── If API_URL is absent (local dev):
        Handle in-process (control-plane.ts fallback)
        All responses carry provenance: "LOCAL_FALLBACK"
```

**The browser never calls the Rust API directly.** All calls go through the `/api/*` Next.js route handler, which either proxies to Rust (production) or uses the local fallback.

This means:
- CORS is not an issue — only the Next.js server makes requests to the Rust API
- Service-to-service auth (OIDC) is handled entirely server-side
- The browser only ever sees the Next.js domain

---

## 8. How to Submit a Real Agent for Certification

### Step 1 — Register the candidate

```bash
# Using sentinelctl
cargo run --bin sentinelctl -- candidate register \
  --api-url "$SENTINEL_API_URL" \
  --tenant 00000000-0000-0000-0000-000000000001 \
  --abom-file path/to/your-agent-abom.json

# Or using curl
curl -X POST "$SENTINEL_API_URL/v1/candidates" \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d @path/to/your-agent-abom.json
```

The ABOM JSON must match the schema in [`schemas/candidate.json`](schemas/candidate.json).

### Step 2 — Start a certification workflow

```bash
REVISION_ID="<revision_id from step 1>"
IDEMPOTENCY_KEY=$(uuidgen)

curl -X POST "$SENTINEL_API_URL/v1/workflows" \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d "{
    \"candidate_revision_id\": \"$REVISION_ID\",
    \"policy_pack_id\": \"enterprise-v2\",
    \"corpus_version\": \"v1.0.0\",
    \"idempotency_key\": \"$IDEMPOTENCY_KEY\"
  }"
```

This returns a `workflow_id`. The workflow immediately starts progressing:
`REGISTERED → QUEUED → RUNNING → EVIDENCE_PENDING → APPROVAL_REQUIRED`

### Step 3 — Monitor progress

```bash
WORKFLOW_ID="<workflow_id from step 2>"

# Poll state
curl "$SENTINEL_API_URL/v1/workflows/$WORKFLOW_ID" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"

# View audit trail
curl "$SENTINEL_API_URL/v1/workflows/$WORKFLOW_ID/audit" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001"
```

Or open the web console and navigate to the **Certification Timeline** tab.

---

## 9. How to Approve and Get an Attestation

Once the workflow reaches `APPROVAL_REQUIRED`:

### In the web console

1. Navigate to the **Reviewer Approval** tab
2. Select role `Security Reviewer` or `Platform Lead` (not `Candidate Owner` — separation of duties blocks self-approval)
3. Enter your principal identifier and click **Authorize Certification**
4. The workflow advances to `ATTESTING` then `CERTIFIED`
5. Navigate to the **KMS Attestation Verifier** tab to download and verify the attestation

### Via API

```bash
curl -X POST "$SENTINEL_API_URL/v1/workflows/$WORKFLOW_ID/approvals" \
  -H "Content-Type: application/json" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  -d "{
    \"reviewer_role\": \"security-reviewer\",
    \"reviewer_principal\": \"alice@your-company.com\",
    \"expiry_days\": 90
  }"
```

### Verify the attestation offline

```bash
# Download attestation
curl "$SENTINEL_API_URL/v1/workflows/$WORKFLOW_ID/attestation" \
  -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
  > attestation.json

# Verify with sentinelctl (no network required after download)
cargo run --bin sentinelctl -- attestation-verify \
  --file attestation.json \
  --tenant 00000000-0000-0000-0000-000000000001 \
  --environment production
```

The verifier checks:
- RSA-PSS-2048-SHA256 signature against the KMS public key
- RFC 8785 canonical JSON before verifying the signature
- Attestation has not expired
- Tenant and environment match
- Evidence manifest digest is intact

---

## 10. Trust Boundaries

Understanding what each layer can and cannot do is essential for operating Sentinel correctly.

| Layer | Can Do | Cannot Do |
|---|---|---|
| **Rust Control Plane** | Own all admission decisions, write Firestore state, sign with KMS, enforce separation of duties | Run Gemini, call Model Armor or Gateway directly |
| **Python ADK Certifier** | Call Gemini, inspect with Model Armor, enforce Gateway policy, query ERP oracle, return typed observations | Make admission decisions, write Firestore state, call KMS, grant approval |
| **Candidate Agent** | Invoke tools in its manifest via Agent Gateway | Call the control plane directly, bypass Gateway, modify its own ABOM |
| **Next.js Console** | Display data, proxy requests to control plane, enforce UI-level role hints | Own canonical state, sign anything, approve workflows server-side |
| **Memory Bank** | Provide advisory context to the certifier | Grant permissions or influence the policy decision |

### Separation of Duties Rule

The control plane enforces that `reviewer_role` submitted with an approval request must **not** match the `owner` principal on the ABOM. A candidate owner cannot approve their own agent's certification. This is validated in the Rust handler — the UI enforces it visually but the control plane is the authoritative check.

### Provenance Labels

Every piece of data in the system carries an explicit `provenance` field:

| Value | Meaning |
|---|---|
| `LIVE` | Produced by a successful live managed service call (Model Armor, Gateway, KMS, Firestore) |
| `LOCAL_FALLBACK` | Produced by the Next.js in-process module when `API_URL` is absent — never proof of a managed service |
| `REPLAY` | Immutable prior evidence rendered without new side effects |
| `INFERRED` | Model/statistical interpretation, not a direct observation |

If you see `LOCAL_FALLBACK` in a production deployment, `API_URL` is not set on the web Cloud Run service. Check the Terraform output and ensure `API_URL` is wired correctly.

---

## Quick Troubleshooting

**Web console shows `provenance: LOCAL_FALLBACK` in production**
→ `API_URL` is not set on the web Cloud Run service. Run `terraform -chdir=infra/terraform apply` to re-wire it.

**Workflow stays in `QUEUED` or `RUNNING` indefinitely**
→ The workflow-worker is not running or cannot reach the ADK certifier. Check Cloud Run logs for `sentinel-workflow-worker`.

**Approval returns 409 Conflict**
→ The reviewer's role matches the ABOM owner — separation of duties enforced. Use a different reviewer principal.

**`attestation-verify` fails with signature error**
→ The attestation was produced in local fallback mode (SHA-256 stand-in, not KMS). Only attestations produced against a live KMS key will verify.

**ADK certifier returns `is_live: false` observations**
→ One or more managed service env vars (`MODEL_ARMOR_TEMPLATE`, `AGENT_GATEWAY_RESOURCE`, `MEMORY_BANK_RESOURCE`) are not set. Run `bash infra/scripts/setup-managed-agents.sh` and redeploy.
