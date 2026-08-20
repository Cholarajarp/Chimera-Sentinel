"""HTTP server for Google ADK Certifier."""

import logging
import uuid

from fastapi import FastAPI, HTTPException
from pydantic import BaseModel

from sentinel_adk_certifier.config import Settings
from sentinel_adk_certifier.evaluator import (
    CaseObservation,
    EvaluationBatchResult,
    execute_case,
    execute_case_live,
    get_erp_snapshot,
    reset_erp_fixture,
)

logger = logging.getLogger(__name__)


class EvaluationRequest(BaseModel):
    tenant_id: str
    workflow_id: str
    candidate_revision_id: str
    candidate_identity: str = "sa-candidate@project.iam.gserviceaccount.com"
    case_ids: list[str]
    policy_pack_id: str
    corpus_version: str
    trace_id: str | None = None
    is_live: bool = True  # When False, use LOCAL corpus-driven path


class EvaluationStatusResponse(BaseModel):
    evaluation_id: str
    status: str
    result: EvaluationBatchResult | None = None


# In-memory evaluation store (evaluations are stateless results; durable state lives in Rust).
# Capped at 500 entries — oldest evicted first — to prevent unbounded memory growth in
# long-running Cloud Run containers. Durable state always lives in Firestore (Rust side).
_MAX_STORE_SIZE = 500
evaluations_store: dict[str, EvaluationBatchResult] = {}


def create_app(settings: Settings) -> FastAPI:
    app = FastAPI(
        title="Chimera Sentinel Google ADK Certifier",
        description=(
            "Certified adversarial evaluation using Google ADK, Gemini, "
            "Model Armor, and Agent Gateway. Typed observations only — "
            "no approval, signing, or canonical state authority."
        ),
        version="0.1.0",
    )

    @app.get("/healthz")
    async def healthz() -> dict[str, str]:
        return {"status": "ok", "service": "sentinel-adk-certifier", "version": "0.1.0"}

    @app.get("/readyz")
    async def readyz() -> dict[str, str]:
        gcp_configured = bool(settings.google_cloud_project and settings.gemini_model_ref)
        return {
            "status": "ok",
            "gcp_project": settings.google_cloud_project or "not_configured",
            "gemini_model_batch": settings.gemini_model_ref or "not_configured",
            "gemini_model_agent": settings.gemini_model_ref_agent or "not_configured",
            "gemini_model_deep": settings.gemini_model_ref_deep or "not_configured",
            "live_capable": str(gcp_configured),
        }

    @app.post("/internal/v1/evaluations", response_model=EvaluationBatchResult)
    async def create_evaluation(request: EvaluationRequest) -> EvaluationBatchResult:
        eval_id = f"eval-{uuid.uuid4().hex[:12]}"
        observations: list[CaseObservation] = []

        use_live = (
            request.is_live
            and bool(settings.google_cloud_project)
            and bool(settings.gemini_model_ref)
        )
        ledger_before = None
        ledger_after = None

        if use_live:
            try:
                await reset_erp_fixture(
                    settings.erp_mcp_url,
                    request.tenant_id,
                    request.workflow_id,
                )
                ledger_before = await get_erp_snapshot(
                    settings.erp_mcp_url,
                    request.tenant_id,
                    request.workflow_id,
                )
            except Exception as exc:
                raise HTTPException(
                    status_code=502,
                    detail=f"ERP oracle initialization failed: {exc}. Certification blocked.",
                ) from exc

        for case_id in request.case_ids:
            try:
                if use_live:
                    obs = await execute_case_live(
                        tenant_id=request.tenant_id,
                        workflow_id=request.workflow_id,
                        case_id=case_id,
                        candidate_identity=request.candidate_identity,
                        corpus_version=request.corpus_version,
                        google_project=settings.google_cloud_project,
                        google_region=settings.google_cloud_region,
                        gemini_model_ref=settings.gemini_model_ref,  # gemini-3.5-flash-lite
                        gemini_model_ref_agent=settings.gemini_model_ref_agent,  # gemini-3.5-flash
                        gemini_model_ref_deep=settings.gemini_model_ref_deep,  # gemini-3.1-pro
                        model_armor_template=settings.model_armor_template,
                        gateway_resource=settings.gateway_resource,
                        memory_bank_resource=settings.memory_bank_resource,
                        erp_url=settings.erp_mcp_url,
                        trace_id=request.trace_id,
                        is_live=True,
                    )
                else:
                    # Explicitly LOCAL mode — labeled as such
                    obs = execute_case(
                        tenant_id=request.tenant_id,
                        workflow_id=request.workflow_id,
                        case_id=case_id,
                        candidate_identity=request.candidate_identity,
                        corpus_version=request.corpus_version,
                        trace_id=request.trace_id,
                    )
            except Exception as exc:
                # Individual case failure is recorded — never silently dropped
                logger.exception("Case %s failed", case_id)
                raise HTTPException(
                    status_code=502,
                    detail=f"Case {case_id} execution failed: {exc}. Certification blocked.",
                )
            observations.append(obs)

        if use_live:
            try:
                ledger_after = await get_erp_snapshot(
                    settings.erp_mcp_url,
                    request.tenant_id,
                    request.workflow_id,
                )
            except Exception as exc:
                raise HTTPException(
                    status_code=502,
                    detail=f"ERP final snapshot failed: {exc}. Certification blocked.",
                ) from exc

        result = EvaluationBatchResult(
            evaluation_id=eval_id,
            tenant_id=request.tenant_id,
            workflow_id=request.workflow_id,
            candidate_revision_id=request.candidate_revision_id,
            observations=observations,
            ledger_snapshot_before=ledger_before,
            ledger_snapshot_after=ledger_after,
            gemini_model_ref=settings.gemini_model_ref if use_live else "LOCAL",
            gemini_model_ref_agent=settings.gemini_model_ref_agent if use_live else "LOCAL",
            gemini_model_ref_deep=settings.gemini_model_ref_deep if use_live else "LOCAL",
            status="COMPLETED",
        )

        if len(evaluations_store) >= _MAX_STORE_SIZE:
            # Evict the oldest entry (insertion-order dict, Python 3.7+)
            oldest_key = next(iter(evaluations_store))
            del evaluations_store[oldest_key]
            logger.debug("Evicted oldest evaluation %s from in-memory store", oldest_key)
        evaluations_store[eval_id] = result
        logger.info(
            "Evaluation %s completed: %d cases, live=%s",
            eval_id,
            len(observations),
            use_live,
        )
        return result

    @app.get("/internal/v1/evaluations/{evaluation_id}", response_model=EvaluationStatusResponse)
    async def get_evaluation(evaluation_id: str) -> EvaluationStatusResponse:
        if evaluation_id in evaluations_store:
            return EvaluationStatusResponse(
                evaluation_id=evaluation_id,
                status="COMPLETED",
                result=evaluations_store[evaluation_id],
            )
        raise HTTPException(status_code=404, detail="Evaluation ID not found")

    @app.post("/internal/v1/evaluations/{evaluation_id}/cancel")
    async def cancel_evaluation(evaluation_id: str) -> dict[str, str]:
        if evaluation_id in evaluations_store:
            evaluations_store[evaluation_id].status = "CANCELLED"
            return {"evaluation_id": evaluation_id, "status": "CANCELLED"}
        raise HTTPException(status_code=404, detail="Evaluation ID not found")

    return app
