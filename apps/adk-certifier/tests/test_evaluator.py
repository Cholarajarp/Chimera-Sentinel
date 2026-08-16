"""Tests for ADK Case Evaluator and Server."""

import json
from pathlib import Path

import pytest
from httpx import AsyncClient, ASGITransport
from sentinel_adk_certifier.config import Settings
from sentinel_adk_certifier.evaluator import CaseObservation, execute_case
import sentinel_adk_certifier.server as server_module
from sentinel_adk_certifier.server import create_app


TENANT_ID = "00000000-0000-0000-0000-000000000001"
WORKFLOW_ID = "00000000-0000-0000-0000-000000000002"
REVISION_ID = "sha256:d8a9f3e0984c1a70519f8e4b85c74a0058b871c54b2d5612666ad63d3957fb41"


def evaluation_payload(case_ids: list[str], *, is_live: bool = True) -> dict[str, object]:
    return {
        "tenant_id": TENANT_ID,
        "workflow_id": WORKFLOW_ID,
        "candidate_revision_id": REVISION_ID,
        "candidate_identity": "sa@test.iam",
        "case_ids": case_ids,
        "policy_pack_id": "ap-agent-v1",
        "corpus_version": "v1.0.0",
        "is_live": is_live,
    }


def live_observation(case_id: str) -> CaseObservation:
    return CaseObservation(
        tenant_id=TENANT_ID,
        workflow_id=WORKFLOW_ID,
        case_run_id="00000000-0000-0000-0000-000000000003",
        case_id=case_id,
        corpus_version="v1.0.0",
        expected_outcome="SAFE_TASK_COMPLETED",
        observed_outcome="DRAFT_CREATED",
        model_armor_disposition="ALLOW",
        candidate_identity="sa@test.iam",
        requested_tool="draft_invoice_payment",
        gateway_decision="ALLOW",
        ledger_snapshot_before_digest="sha256:before",
        ledger_snapshot_after_digest="sha256:after",
        latency_ms=10,
        token_count=20,
        cost_usd_micro=5,
        provenance="LIVE",
    )


def test_execute_safe_draft_case():
    obs = execute_case(
        tenant_id=TENANT_ID,
        workflow_id="wf-001",
        case_id="case-safe-invoice-001",
        candidate_identity="sa@test.iam",
        corpus_version="v1.0.0",
    )
    assert obs.expected_outcome == "SAFE_TASK_COMPLETED"
    assert obs.observed_outcome == "DRAFT_CREATED"
    assert obs.model_armor_disposition == "ALLOW"
    assert obs.gateway_decision == "ALLOW"
    assert obs.provenance == "LOCAL"


def test_execute_prompt_injection_case():
    obs = execute_case(
        tenant_id=TENANT_ID,
        workflow_id="wf-001",
        case_id="case-prompt-injection-direct-001",
        candidate_identity="sa@test.iam",
        corpus_version="v1.0.0",
    )
    assert obs.expected_outcome == "PREVENTED_AT_ARMOR"
    assert obs.observed_outcome == "PREVENTED_AT_ARMOR"
    assert obs.model_armor_disposition == "BLOCK"
    assert obs.requested_tool is None


def test_execute_excessive_agency_case():
    obs = execute_case(
        tenant_id=TENANT_ID,
        workflow_id="wf-001",
        case_id="case-excessive-agency-001",
        candidate_identity="sa@test.iam",
        corpus_version="v1.0.0",
    )
    assert obs.expected_outcome == "PREVENTED_AT_GATEWAY"
    assert obs.observed_outcome == "PREVENTED_AT_GATEWAY"
    assert obs.gateway_decision == "DENY"


@pytest.mark.asyncio
async def test_server_evaluations_endpoint():
    app = create_app(Settings())
    transport = ASGITransport(app=app)
    async with AsyncClient(transport=transport, base_url="http://test") as client:
        # Health check
        res = await client.get("/healthz")
        assert res.status_code == 200
        assert res.json()["status"] == "ok"
        assert res.json()["service"] == "sentinel-adk-certifier"

        # Create evaluation batch
        eval_payload = evaluation_payload(
            ["case-safe-invoice-001", "case-prompt-injection-direct-001"],
            is_live=False,
        )
        res = await client.post("/internal/v1/evaluations", json=eval_payload)
        assert res.status_code == 200
        data = res.json()
        assert data["status"] == "COMPLETED"
        assert len(data["observations"]) == 2


def test_all_manifest_cases_load_from_corpus():
    manifest_path = Path(__file__).parents[3] / "corpus" / "v1" / "manifest.json"
    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    case_ids = [item["case_id"] for item in manifest["cases"]]

    assert len(case_ids) == 80
    observations = [
        execute_case(
            tenant_id=TENANT_ID,
            workflow_id=WORKFLOW_ID,
            case_id=case_id,
            candidate_identity="sa@test.iam",
            corpus_version="v1.0.0",
        )
        for case_id in case_ids
    ]
    assert {observation.case_id for observation in observations} == set(case_ids)
    assert all(observation.provenance == "LOCAL" for observation in observations)


@pytest.mark.asyncio
async def test_live_batch_returns_workflow_scoped_oracle_evidence(monkeypatch):
    calls: list[tuple[str, str, str]] = []
    snapshots = [
        {"digest": "sha256:before", "total_drafts": 0},
        {"digest": "sha256:after", "total_drafts": 1},
    ]

    async def fake_reset(erp_url: str, tenant_id: str, workflow_id: str) -> None:
        calls.append(("reset", tenant_id, workflow_id))

    async def fake_snapshot(erp_url: str, tenant_id: str, workflow_id: str):
        calls.append(("snapshot", tenant_id, workflow_id))
        return snapshots.pop(0)

    async def fake_execute_case_live(**kwargs):
        return live_observation(kwargs["case_id"])

    monkeypatch.setattr(server_module, "reset_erp_fixture", fake_reset)
    monkeypatch.setattr(server_module, "get_erp_snapshot", fake_snapshot)
    monkeypatch.setattr(server_module, "execute_case_live", fake_execute_case_live)

    app = create_app(
        Settings(
            google_cloud_project="chimera-sentinel",
            gemini_model_ref="gemini-test",
            erp_mcp_url="https://erp.example.run.app",
        )
    )
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
        response = await client.post(
            "/internal/v1/evaluations",
            json=evaluation_payload(["case-safe-invoice-001"]),
        )

    assert response.status_code == 200
    body = response.json()
    assert body["ledger_snapshot_before"]["digest"] == "sha256:before"
    assert body["ledger_snapshot_after"]["digest"] == "sha256:after"
    assert body["observations"][0]["provenance"] == "LIVE"
    assert calls == [
        ("reset", TENANT_ID, WORKFLOW_ID),
        ("snapshot", TENANT_ID, WORKFLOW_ID),
        ("snapshot", TENANT_ID, WORKFLOW_ID),
    ]


@pytest.mark.asyncio
async def test_live_batch_fails_closed_when_oracle_reset_fails(monkeypatch):
    async def failing_reset(erp_url: str, tenant_id: str, workflow_id: str) -> None:
        raise RuntimeError("oracle unavailable")

    monkeypatch.setattr(server_module, "reset_erp_fixture", failing_reset)
    app = create_app(
        Settings(
            google_cloud_project="chimera-sentinel",
            gemini_model_ref="gemini-test",
            erp_mcp_url="https://erp.example.run.app",
        )
    )
    async with AsyncClient(transport=ASGITransport(app=app), base_url="http://test") as client:
        response = await client.post(
            "/internal/v1/evaluations",
            json=evaluation_payload(["case-safe-invoice-001"]),
        )

    assert response.status_code == 502
    assert "Certification blocked" in response.json()["detail"]
