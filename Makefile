.PHONY: all check test build lint format unit contract corpus-validate smoke-live submission-check \
        run-mcp run-api run-worker run-certifier run-web verify-demo bootstrap clean scan

# ─────────────────────────────────────────────────────────────────────────────
# Default
# ─────────────────────────────────────────────────────────────────────────────
all: check test

# ─────────────────────────────────────────────────────────────────────────────
# Rust workspace targets
# ─────────────────────────────────────────────────────────────────────────────

## Fast type-check (no codegen)
check:
	cargo check --workspace --all-targets

## All workspace unit + integration tests
unit:
	cargo test --workspace

## Clippy — deny warnings (matches CI)
lint:
	cargo clippy --workspace --all-targets -- -D warnings
	cd apps/adk-certifier && uv run ruff check src/

## Format code (Rust + Python)
format:
	cargo fmt --all
	cd apps/adk-certifier && uv run ruff format src/

## Release binary build + web production build
build:
	cargo build --workspace --release
	cd apps/web && pnpm build

# ─────────────────────────────────────────────────────────────────────────────
# Combined test targets
# ─────────────────────────────────────────────────────────────────────────────

## Rust + Python tests
test:
	cargo test --workspace
	cd apps/adk-certifier && uv run pytest tests/ -v

## Contract/schema tests only
contract:
	cargo test --package sentinel-contracts

# ─────────────────────────────────────────────────────────────────────────────
# Corpus validation
# ─────────────────────────────────────────────────────────────────────────────

## Validate all 80 corpus case files have required fields
corpus-validate:
	@echo "Checking development cases (expect 64)..."
	@count=$$(ls corpus/v1/cases/*.json 2>/dev/null | wc -l); \
		echo "  Found: $$count"; \
		[ "$$count" -eq 64 ] || (echo "FAIL: expected 64 development cases" && exit 1)
	@echo "Checking holdout cases (expect 16)..."
	@count=$$(ls corpus/v1/holdout/*.json 2>/dev/null | wc -l); \
		echo "  Found: $$count"; \
		[ "$$count" -eq 16 ] || (echo "FAIL: expected 16 holdout cases" && exit 1)
	@python3 -c "\
import json, sys, glob; \
required = {'schema_version','case_id','category','split','fixture','expectations','provenance'}; \
[sys.exit(1) or print(f'FAIL {f}: missing {required - set(json.load(open(f)).keys())}') \
  for f in glob.glob('corpus/v1/cases/*.json') + glob.glob('corpus/v1/holdout/*.json') \
  if required - set(json.load(open(f)).keys())]; \
print('OK — all 80 corpus cases pass schema validation')"

# ─────────────────────────────────────────────────────────────────────────────
# Local service runners
# ─────────────────────────────────────────────────────────────────────────────

## Enterprise ERP Adapter MCP server (Rust, :9090)
run-mcp:
	cargo run --bin sentinel-enterprise-erp-adapter -- --addr 127.0.0.1:9090

## Control plane REST API (Rust, :8080)
run-api:
	cargo run --bin sentinel-control-plane -- --addr 0.0.0.0:8080

## Workflow worker (Rust)
run-worker:
	cargo run --bin sentinel-workflow-worker

## ADK Certifier (Python, :8081)
run-certifier:
	cd apps/adk-certifier && uv run uvicorn sentinel_adk_certifier.server:create_app --factory --host 0.0.0.0 --port 8081

## Next.js dev server (:3000)
run-web:
	cd apps/web && pnpm dev

## All services locally (requires tmux or multiple terminals — shows commands)
run-all:
	@echo "Start each in a separate terminal:"
	@echo "  make run-mcp"
	@echo "  make run-api"
	@echo "  make run-worker"
	@echo "  make run-certifier"
	@echo "  make run-web"

# ─────────────────────────────────────────────────────────────────────────────
# Attestation verification
# ─────────────────────────────────────────────────────────────────────────────

## Offline verify the demo attestation envelope
verify-demo:
	cargo run --bin sentinelctl -- attestation-verify \
		--file demo/attestation-sample.json \
		--tenant 00000000-0000-0000-0000-000000000001 \
		--environment production

# ─────────────────────────────────────────────────────────────────────────────
# Live smoke test (requires GCP credentials + running Cloud Run services)
# ─────────────────────────────────────────────────────────────────────────────

## End-to-end smoke test against live GCP stack
smoke-live:
	@echo "=== Live smoke test against $$SENTINEL_API_URL ==="
	@[ -n "$$SENTINEL_API_URL" ] || (echo "Set SENTINEL_API_URL=https://..." && exit 1)
	@[ -n "$$GOOGLE_CLOUD_PROJECT" ] || (echo "Set GOOGLE_CLOUD_PROJECT" && exit 1)
	@# 1. Health check
	curl -sf "$$SENTINEL_API_URL/healthz" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['status']=='ok', d; print('OK healthz')"
	@# 2. Create candidate
	@CAND_ID=$$(curl -sf -X POST "$$SENTINEL_API_URL/v1/candidates" \
		-H "Content-Type: application/json" \
		-H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
		-d '{"tenant_id":"00000000-0000-0000-0000-000000000001","agent_name":"smoke-agent","agent_version":"0.0.1","source_digest":"sha256:aabbcc","model_ref":"vertex-ai:gemini-3.5-flash","prompt_digest":"sha256:001122","tool_digest":"sha256:334455","gateway_policy_digest":"sha256:667788","model_armor_template_digest":"sha256:99aabb","agent_identity":"sa-smoke@test.iam.gserviceaccount.com","registered_by":"smoke-test@local"}' \
		| python3 -c "import sys,json; print(json.load(sys.stdin)['candidate_id'])"); \
	echo "Created candidate: $$CAND_ID"; \
	@# 3. Create workflow
	@WF_ID=$$(curl -sf -X POST "$$SENTINEL_API_URL/v1/workflows" \
		-H "Content-Type: application/json" \
		-H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" \
		-d "{\"tenant_id\":\"00000000-0000-0000-0000-000000000001\",\"candidate_id\":\"$$CAND_ID\",\"policy_pack_id\":\"ap-agent-v1\",\"corpus_version\":\"v1.0.0\",\"requester_id\":\"smoke@local\"}" \
		| python3 -c "import sys,json; print(json.load(sys.stdin)['workflow_id'])"); \
	echo "Created workflow: $$WF_ID"; \
	@# 4. Fleet posture
	curl -sf "$$SENTINEL_API_URL/v1/fleet/posture" -H "X-Tenant-ID: 00000000-0000-0000-0000-000000000001" | python3 -c "import sys,json; d=json.load(sys.stdin); print('Fleet posture OK:', d)"
	@echo "=== Live smoke test PASSED ==="

# ─────────────────────────────────────────────────────────────────────────────
# Bootstrap (first-time setup)
# ─────────────────────────────────────────────────────────────────────────────

## Install all toolchain dependencies (Rust, Python uv, Node pnpm)
bootstrap:
	@echo "Checking Rust..."
	rustup show
	@echo "Checking uv..."
	uv --version || (echo "Install uv: curl -LsSf https://astral.sh/uv/install.sh | sh" && exit 1)
	@echo "Checking pnpm..."
	pnpm --version || (npm install -g pnpm && echo "Installed pnpm")
	@echo "Installing Python dependencies..."
	cd apps/adk-certifier && uv sync --all-extras
	@echo "Installing Node dependencies..."
	cd apps/web && pnpm install
	@echo "Bootstrap complete. Copy .env.example to .env and fill in your GCP values."

# ─────────────────────────────────────────────────────────────────────────────
# Security scanning
# ─────────────────────────────────────────────────────────────────────────────

## Run security advisories across all language ecosystems
scan:
	@echo "=== Rust: cargo audit ==="
	cargo audit
	@echo "=== Node: pnpm audit ==="
	cd apps/web && pnpm audit --audit-level high
	@echo "=== Python: pip-audit ==="
	cd apps/adk-certifier && uv run pip-audit
	@echo "=== Scan complete ==="

# ─────────────────────────────────────────────────────────────────────────────
# Pre-submission checklist
# ─────────────────────────────────────────────────────────────────────────────

## Full pre-submission checklist — run before hackathon deadline
submission-check: check lint corpus-validate verify-demo scan
	@echo ""
	@echo "=== Submission Checklist ==="
	@echo ""
	@echo "[Code Quality]"
	@echo "  ✓ cargo check passed"
	@echo "  ✓ clippy (deny warnings) passed"
	@echo "  ✓ corpus 80-case validation passed"
	@echo "  ✓ demo attestation verifies offline"
	@echo ""
	@echo "[Manual checks required before submission]"
	@echo "  □ terraform apply completed successfully"
	@echo "  □ bash infra/scripts/setup-managed-agents.sh ran without errors"
	@echo "  □ make smoke-live SENTINEL_API_URL=https://your-cloud-run-url passed"
	@echo "  □ Web console loads and shows PROVENANCE: LIVE from real API"
	@echo "  □ 4-minute demo video recorded and uploaded"
	@echo "  □ README.md links to live Cloud Run URLs"
	@echo "  □ NOTICE.md clean-room disclosure is present"
	@echo ""
	@echo "Run 'make smoke-live SENTINEL_API_URL=<url>' to validate the live stack."

# ─────────────────────────────────────────────────────────────────────────────
# Cleanup
# ─────────────────────────────────────────────────────────────────────────────

clean:
	cargo clean
	rm -rf apps/web/.next apps/web/out
	find . -type d -name __pycache__ -exec rm -rf {} + 2>/dev/null || true
	find . -name "*.pyc" -delete 2>/dev/null || true
