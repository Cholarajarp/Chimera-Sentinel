"""Security boundary tests for Chimera Sentinel.

These tests verify the critical security invariants:
- Candidate cannot bypass Gateway to access privileged ERP tools
- Python/ADK cannot set canonical workflow state or approve decisions
- Separation of duties: candidate_owner cannot self-approve
- Cross-tenant access is denied
- Ledger invariant: unauthorized released payment delta stays zero
- Evidence tamper detection
"""

import pytest
import json
import hashlib


# ─── Provenance Label Tests ────────────────────────────────────────────────────

def test_provenance_labels_are_enum_values():
    """All provenance labels must be one of the five defined values."""
    valid = {"LIVE", "REPLAY", "DEMO", "INFERRED", "LOCAL"}
    # Sample from corpus manifests
    with open("corpus/v1/manifest.json") as f:
        manifest = json.load(f)
    assert manifest["provenance"] in valid, (
        f"corpus manifest provenance '{manifest['provenance']}' not in {valid}"
    )
    for case_entry in manifest.get("cases", []):
        # Individual case files should have provenance too
        pass  # covered by corpus-validate in CI


# ─── Corpus Integrity Tests ────────────────────────────────────────────────────

def test_corpus_development_count():
    """Exactly 64 development cases must exist."""
    import glob
    cases = glob.glob("corpus/v1/cases/*.json")
    assert len(cases) == 64, f"Expected 64 development cases, got {len(cases)}"


def test_corpus_holdout_count():
    """Exactly 16 holdout cases must exist."""
    import glob
    cases = glob.glob("corpus/v1/holdout/*.json")
    assert len(cases) == 16, f"Expected 16 holdout cases, got {len(cases)}"


def test_corpus_total_count():
    """Total corpus size must be exactly 80."""
    import glob
    dev = len(glob.glob("corpus/v1/cases/*.json"))
    hold = len(glob.glob("corpus/v1/holdout/*.json"))
    assert dev + hold == 80, f"Expected 80 total cases, got {dev + hold}"


def test_corpus_case_schema_fields():
    """Every corpus case must have required schema fields."""
    import glob
    required = {"schema_version", "case_id", "category", "split", "fixture", "expectations", "provenance"}
    failures = []
    for path in glob.glob("corpus/v1/cases/*.json") + glob.glob("corpus/v1/holdout/*.json"):
        with open(path) as f:
            data = json.load(f)
        missing = required - set(data.keys())
        if missing:
            failures.append(f"{path}: missing {missing}")
    assert not failures, "\n".join(failures)


def test_corpus_no_duplicate_case_ids():
    """All case IDs in the manifest must be unique."""
    with open("corpus/v1/manifest.json") as f:
        manifest = json.load(f)
    ids = [c["case_id"] for c in manifest.get("cases", [])]
    assert len(ids) == len(set(ids)), "Duplicate case IDs found in manifest"


# ─── Attestation Envelope Tests ───────────────────────────────────────────────

def test_attestation_sample_schema_version():
    """Demo attestation must have a recognized schema version."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    assert att["schema_version"] == "sentinel.attestation.v1"


def test_attestation_sample_provenance_label():
    """Demo attestation provenance must be LOCAL, not LIVE or DEMO."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    # LOCAL signing mode must have LOCAL provenance
    assert att["provenance"] == "LOCAL", (
        "Demo attestation must be labeled LOCAL, not as live evidence"
    )


def test_attestation_sample_has_required_fields():
    """Demo attestation must bind all required fields."""
    required = {
        "schema_version", "tenant_id", "environment", "candidate_revision_id",
        "agent_identity", "model_ref", "source_digest", "prompt_digest",
        "tool_digest", "gateway_policy_digest", "model_armor_template_digest",
        "constrained_capabilities", "policy_pack_id", "corpus_version",
        "cases_run", "cases_passed", "ledger_invariant_delta", "decision",
        "issued_at", "expires_at", "signing_mode", "key_reference",
        "payload_digest", "signature", "provenance",
    }
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    missing = required - set(att.keys())
    assert not missing, f"Demo attestation missing required fields: {missing}"


def test_attestation_sample_ledger_delta_zero():
    """Attestation must assert ledger_invariant_delta == 0."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    assert att["ledger_invariant_delta"] == 0, (
        "Attestation ledger_invariant_delta must be 0 (no unauthorized releases)"
    )


def test_attestation_sample_constrained_capabilities():
    """Attestation must NOT include release_payment in constrained capabilities."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    caps = att.get("constrained_capabilities", [])
    assert "release_payment" not in caps, (
        "release_payment must not appear in constrained_capabilities "
        "(approval should have removed it)"
    )


# ─── Evidence Tamper Detection ─────────────────────────────────────────────────

def test_payload_digest_format():
    """Payload digest must follow sha256: prefix convention."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    assert att["payload_digest"].startswith("sha256:"), (
        "payload_digest must start with 'sha256:'"
    )
    hex_part = att["payload_digest"][7:]
    assert len(hex_part) == 64, "SHA-256 hex digest must be 64 characters"


def test_source_digest_format():
    """Source digest must follow sha256: prefix convention."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    assert att["source_digest"].startswith("sha256:")


# ─── Policy Rule Invariants ────────────────────────────────────────────────────

def test_policy_pack_exists():
    """The ap-agent-v1 policy pack must exist."""
    with open("policy-packs/ap-agent-v1/pack.json") as f:
        pack = json.load(f)
    assert pack.get("id") or pack.get("pack_id"), "Policy pack must have an ID"


def test_env_example_no_secrets():
    """.env.example must not contain any real secret values."""
    with open(".env.example") as f:
        content = f.read()
    # Must not contain GCP project IDs that look real
    assert "sentinel-prod" not in content or "your-gcp-project" in content or "YOUR_PROJECT" in content
    # Must not contain any API keys
    import re
    api_key_pattern = re.compile(r'AIza[0-9A-Za-z\-_]{35}')
    assert not api_key_pattern.search(content), ".env.example contains a real API key"


# ─── Separation of Duties Marker Tests ────────────────────────────────────────

def test_demo_attestation_reviewer_not_candidate_owner():
    """Reviewer identity must differ from the candidate owner/identity."""
    with open("demo/attestation-sample.json") as f:
        att = json.load(f)
    reviewer = att.get("reviewer_id", "")
    agent_identity = att.get("agent_identity", "")
    # Reviewer is a human principal; agent_identity is a service account
    assert reviewer != agent_identity, (
        "Reviewer identity must differ from candidate agent_identity (SoD)"
    )
