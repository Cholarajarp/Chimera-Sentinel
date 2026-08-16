//! Chimera Sentinel Control Plane API Server.
//!
//! Provides REST APIs for candidate management, durable certification workflows,
//! findings, human approvals, evidence manifests, and KMS attestation verification.
//! All handlers are fully wired to the persistence layer.

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, HeaderValue, Method, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use clap::Parser;
use sentinel_config::{load_config, ControlPlaneConfig};
use serde::Deserialize;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    limit::RequestBodyLimitLayer,
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{error, info, warn};
use uuid::Uuid;

use sentinel_attestation::verify_attestation;
use sentinel_contracts::{
    approval::{SubmitApprovalRequest, SubmitApprovalResponse},
    attestation::{AttestationResponse, AttestationVerifyRequest, AttestationVerifyResponse, WorkflowResponse},
    candidate::{CandidateDetailResponse, CandidateListResponse, CandidateRevisionRequest, CandidateRevisionResponse},
    evidence::EvidenceManifestResponse,
    finding::FindingsListResponse,
    workflow::{CreateCertificationRequest, CreateCertificationResponse, WorkflowListResponse},
    ErrorResponse,
};
use sentinel_domain::{
    candidate::CandidateRevision,
    evidence::Approval,
    ids::{ApprovalId, TenantId, WorkflowId},
    workflow::{Workflow, WorkflowCommand, WorkflowState},
    AuditEvent, AuditEventType,
};
use sentinel_metrics::{gather_metrics, HTTP_REQUESTS};
use sentinel_observability::init_tracing;
use sentinel_config::DatabaseConfig;
use sentinel_persistence::{
    firestore::FirestoreStore,
    memory::InMemoryStore,
    ApprovalRepository, AttestationRepository, AuditRepository,
    CandidateRepository, EvidenceRepository, FindingRepository, WorkflowRepository,
};

#[derive(Parser)]
#[command(name = "sentinel-control-plane")]
struct Args {
    #[arg(long, default_value = "/app/config/control-plane.toml")]
    config: String,
    #[arg(long, default_value = "0.0.0.0:8080")]
    addr: SocketAddr,
}

/// Unified store that delegates to either InMemoryStore or FirestoreStore.
/// Both implement all repository traits; this enum lets the rest of the code
/// stay identical regardless of which backend is configured.
#[derive(Clone)]
enum Store {
    Memory(Arc<InMemoryStore>),
    Firestore(Arc<FirestoreStore>),
}

// Implement each repository trait by delegating to the active backend.
macro_rules! impl_repo {
    ($trait:ident, $method:ident ( $($arg:ident : $ty:ty),* ) -> $ret:ty) => {
        // this macro pattern is for documentation only; see individual impls below
    };
}

#[async_trait::async_trait]
impl CandidateRepository for Store {
    async fn create(&self, c: &sentinel_domain::candidate::CandidateRevision) -> Result<(), String> {
        match self { Store::Memory(s) => s.create(c).await, Store::Firestore(s) => s.create(c).await }
    }
    async fn get(&self, t: sentinel_domain::ids::TenantId, r: &sentinel_domain::candidate::RevisionId) -> Result<Option<sentinel_domain::candidate::CandidateRevision>, String> {
        match self { Store::Memory(s) => s.get(t, r).await, Store::Firestore(s) => s.get(t, r).await }
    }
    async fn list(&self, t: sentinel_domain::ids::TenantId, l: usize, c: Option<String>) -> Result<Vec<sentinel_domain::candidate::CandidateRevision>, String> {
        match self { Store::Memory(s) => s.list(t, l, c).await, Store::Firestore(s) => s.list(t, l, c).await }
    }
}
#[async_trait::async_trait]
impl WorkflowRepository for Store {
    async fn create(&self, w: &sentinel_domain::workflow::Workflow) -> Result<(), String> {
        match self { Store::Memory(s) => s.create(w).await, Store::Firestore(s) => s.create(w).await }
    }
    async fn get(&self, t: sentinel_domain::ids::TenantId, w: sentinel_domain::ids::WorkflowId) -> Result<Option<sentinel_domain::workflow::Workflow>, String> {
        match self { Store::Memory(s) => s.get(t, w).await, Store::Firestore(s) => s.get(t, w).await }
    }
    async fn update(&self, w: &sentinel_domain::workflow::Workflow, v: u64) -> Result<(), String> {
        match self { Store::Memory(s) => s.update(w, v).await, Store::Firestore(s) => s.update(w, v).await }
    }
    async fn list(&self, t: sentinel_domain::ids::TenantId, st: Option<sentinel_domain::workflow::WorkflowState>, l: usize, c: Option<String>) -> Result<Vec<sentinel_domain::workflow::WorkflowSummary>, String> {
        match self { Store::Memory(s) => s.list(t, st, l, c).await, Store::Firestore(s) => s.list(t, st, l, c).await }
    }
}
#[async_trait::async_trait]
impl EvidenceRepository for Store {
    async fn store_manifest(&self, m: &sentinel_domain::evidence::EvidenceManifestRef) -> Result<(), String> {
        match self { Store::Memory(s) => s.store_manifest(m).await, Store::Firestore(s) => s.store_manifest(m).await }
    }
    async fn get_manifest(&self, t: sentinel_domain::ids::TenantId, w: sentinel_domain::ids::WorkflowId) -> Result<Option<sentinel_domain::evidence::EvidenceManifestRef>, String> {
        match self { Store::Memory(s) => s.get_manifest(t, w).await, Store::Firestore(s) => s.get_manifest(t, w).await }
    }
}
#[async_trait::async_trait]
impl AuditRepository for Store {
    async fn append(&self, e: &sentinel_domain::AuditEvent) -> Result<(), String> {
        match self { Store::Memory(s) => s.append(e).await, Store::Firestore(s) => s.append(e).await }
    }
    async fn query(&self, t: sentinel_domain::ids::TenantId, w: Option<sentinel_domain::ids::WorkflowId>, l: usize) -> Result<Vec<sentinel_domain::AuditEvent>, String> {
        match self { Store::Memory(s) => s.query(t, w, l).await, Store::Firestore(s) => s.query(t, w, l).await }
    }
}
#[async_trait::async_trait]
impl FindingRepository for Store {
    async fn store(&self, f: &sentinel_domain::finding::Finding) -> Result<(), String> {
        match self { Store::Memory(s) => s.store(f).await, Store::Firestore(s) => s.store(f).await }
    }
    async fn list(&self, t: sentinel_domain::ids::TenantId, w: sentinel_domain::ids::WorkflowId) -> Result<Vec<sentinel_domain::finding::Finding>, String> {
        match self { Store::Memory(s) => s.list(t, w).await, Store::Firestore(s) => s.list(t, w).await }
    }
}
#[async_trait::async_trait]
impl ApprovalRepository for Store {
    async fn create(&self, a: &sentinel_domain::evidence::Approval) -> Result<(), String> {
        match self { Store::Memory(s) => s.create(a).await, Store::Firestore(s) => s.create(a).await }
    }
    async fn get(&self, t: sentinel_domain::ids::TenantId, a: sentinel_domain::ids::ApprovalId) -> Result<Option<sentinel_domain::evidence::Approval>, String> {
        match self { Store::Memory(s) => s.get(t, a).await, Store::Firestore(s) => s.get(t, a).await }
    }
    async fn get_for_workflow(&self, t: sentinel_domain::ids::TenantId, w: sentinel_domain::ids::WorkflowId) -> Result<Option<sentinel_domain::evidence::Approval>, String> {
        match self { Store::Memory(s) => s.get_for_workflow(t, w).await, Store::Firestore(s) => s.get_for_workflow(t, w).await }
    }
}
#[async_trait::async_trait]
impl AttestationRepository for Store {
    async fn store(&self, w: sentinel_domain::ids::WorkflowId, a: &sentinel_domain::attestation::AttestationMetadata) -> Result<(), String> {
        match self { Store::Memory(s) => s.store(w, a).await, Store::Firestore(s) => s.store(w, a).await }
    }
    async fn get(&self, t: sentinel_domain::ids::TenantId, w: sentinel_domain::ids::WorkflowId) -> Result<Option<sentinel_domain::attestation::AttestationMetadata>, String> {
        match self { Store::Memory(s) => s.get(t, w).await, Store::Firestore(s) => s.get(t, w).await }
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
    config: Arc<ControlPlaneConfig>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let config: ControlPlaneConfig = load_config(&args.config).unwrap_or_default();
    sentinel_config::init_logging(&config.observability)?;
    let _guard = init_tracing(&config.observability.otel_service_name);

    info!("Starting Chimera Sentinel Control Plane v0.1.0");
    info!("Server addr: {}", args.addr);
    info!("Live Google Cloud services: {}", config.google_cloud.use_live_services);

    // Select persistence backend based on config.
    // DatabaseConfig::Firestore → durable Cloud Firestore (required for production)
    // DatabaseConfig::Memory    → in-process (local dev / tests only)
    let store: Arc<Store> = match &config.database {
        DatabaseConfig::Firestore { project_id, database_id } => {
            let db = database_id.clone().unwrap_or_else(|| "(default)".to_string());
            info!("Persistence backend: Firestore project={} database={}", project_id, db);
            Arc::new(Store::Firestore(Arc::new(FirestoreStore::new(project_id, db))))
        }
        _ => {
            warn!("Persistence backend: InMemoryStore — state is NOT durable across restarts. \
                   Set database.type=firestore in config for production.");
            Arc::new(Store::Memory(Arc::new(InMemoryStore::new())))
        }
    };

    let state = AppState {
        store,
        config: Arc::new(config),
    };

    let environment = std::env::var("SENTINEL_ENVIRONMENT")
        .unwrap_or_else(|_| "development".to_string());
    let allowed_origin = std::env::var("SENTINEL_ALLOWED_ORIGIN").unwrap_or_default();
    let cors = if !allowed_origin.is_empty() {
        let origin = HeaderValue::from_str(&allowed_origin)
            .map_err(|error| anyhow::anyhow!("SENTINEL_ALLOWED_ORIGIN is invalid: {error}"))?;
        CorsLayer::new()
            .allow_origin(origin)
            .allow_methods([Method::GET, Method::POST])
            .allow_headers(Any)
    } else if environment == "development" {
        warn!("CORS allows any origin in development mode");
        CorsLayer::permissive()
    } else {
        info!("Cross-origin browser requests are disabled");
        CorsLayer::new()
    };

    let middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(RequestBodyLimitLayer::new(1024 * 1024))
        .layer(TimeoutLayer::new(Duration::from_secs(30)))
        .layer(cors)
        .layer(axum::middleware::from_fn(metrics_middleware));

    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(metrics_endpoint))
        // Candidate management
        .route("/v1/candidates", post(handle_create_candidate).get(handle_list_candidates))
        .route("/v1/candidates/:revision_id", get(handle_get_candidate))
        // Workflow lifecycle
        .route("/v1/workflows", post(handle_create_workflow).get(handle_list_workflows))
        .route("/v1/workflows/:workflow_id", get(handle_get_workflow))
        .route("/v1/workflows/:workflow_id/dispatch", post(handle_dispatch_workflow))
        .route("/v1/workflows/:workflow_id/cancel", post(handle_cancel_workflow))
        // Findings, approvals, evidence, attestation
        .route("/v1/workflows/:workflow_id/findings", get(handle_get_findings))
        .route("/v1/workflows/:workflow_id/approvals", post(handle_submit_approval))
        .route("/v1/workflows/:workflow_id/evidence", get(handle_get_evidence_manifest))
        .route("/v1/workflows/:workflow_id/attestation", get(handle_get_attestation))
        // Standalone verification
        .route("/v1/attestations/verify", post(handle_verify_attestation))
        // Fleet posture
        .route("/v1/fleet/posture", get(handle_get_fleet_posture))
        .layer(middleware)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    info!("Control plane listening on {}", args.addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
            info!("Shutdown signal received");
        })
        .await?;

    info!("Control plane shutdown complete");
    Ok(())
}

// ─── Middleware ───────────────────────────────────────────────────────────────

async fn metrics_middleware(
    req: axum::http::Request<axum::body::Body>,
    next: axum::middleware::Next,
) -> impl axum::response::IntoResponse {
    let method = req.method().to_string();
    let path = req.uri().path().to_string();
    let resp = next.run(req).await;
    HTTP_REQUESTS
        .with_label_values(&[&method, &path, &resp.status().as_u16().to_string()])
        .inc();
    resp
}

async fn metrics_endpoint() -> impl IntoResponse {
    match gather_metrics() {
        Ok(m) => (StatusCode::OK, m),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
    }
}

// ─── Health ───────────────────────────────────────────────────────────────────

async fn health_check() -> impl IntoResponse {
    (StatusCode::OK, Json(serde_json::json!({"status":"healthy","service":"sentinel-control-plane","version":"0.1.0"})))
}

async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = vec![];
    let mut all_healthy = true;

    // Database reachability
    let demo_tenant = TenantId::parse("00000000-0000-0000-0000-000000000001").unwrap_or_default();
    match state.store.list(demo_tenant, None, 1, None).await {
        Ok(_) => checks.push(serde_json::json!({"name":"database","status":"ok","type":"in_memory"})),
        Err(e) => {
            checks.push(serde_json::json!({"name":"database","status":"error","detail":e}));
            all_healthy = false;
        }
    }

    checks.push(serde_json::json!({
        "name": "google_services",
        "status": if state.config.google_cloud.use_live_services { "configured" } else { "local_mode" },
        "live": state.config.google_cloud.use_live_services
    }));

    let code = if all_healthy { StatusCode::OK } else { StatusCode::SERVICE_UNAVAILABLE };
    (code, Json(serde_json::json!({"status": if all_healthy {"ready"} else {"not_ready"}, "checks": checks})))
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn extract_tenant(headers: &HeaderMap) -> Result<TenantId, (StatusCode, Json<ErrorResponse>)> {
    headers
        .get("X-Tenant-ID")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| TenantId::parse(s).ok())
        .ok_or_else(|| {
            (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse::new(400, "Missing Tenant Header", "X-Tenant-ID header with a valid UUID is required")),
            )
        })
}

fn not_found(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::NOT_FOUND, Json(ErrorResponse::new(404, "Not Found", detail)))
}

fn internal(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse::new(500, "Internal Server Error", detail)))
}

fn conflict(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::CONFLICT, Json(ErrorResponse::new(409, "Conflict", detail)))
}

fn forbidden(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (StatusCode::FORBIDDEN, Json(ErrorResponse::new(403, "Forbidden", detail)))
}

#[derive(Deserialize)]
struct ListQuery {
    limit: Option<usize>,
    cursor: Option<String>,
    state: Option<String>,
}

// ─── Candidate Handlers ───────────────────────────────────────────────────────

async fn handle_create_candidate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CandidateRevisionRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;

    let candidate = CandidateRevision::new(
        tenant_id,
        req.agent_id,
        req.abom,
        req.policy_pack_id,
        req.corpus_version,
    );
    let revision_id = candidate.revision_id.clone();

    state
        .store
        .create(&candidate)
        .await
        .map_err(|e| internal(e))?;

    let event = AuditEvent {
        event_id: Uuid::new_v4(),
        tenant_id,
        workflow_id: None,
        case_run_id: None,
        event_type: AuditEventType::CandidateRegistered,
        actor: sentinel_domain::ids::PrincipalId::new("control-plane"),
        before_state: None,
        after_state: None,
        command: None,
        decision: None,
        explanation: Some(format!("Candidate registered: {}", revision_id)),
        provenance: sentinel_domain::provenance::Provenance::Live,
        timestamp: OffsetDateTime::now_utc(),
        trace_id: None,
        span_id: None,
    };
    let _ = state.store.append(&event).await;

    Ok((StatusCode::CREATED, Json(CandidateRevisionResponse { revision_id })))
}

async fn handle_get_candidate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(revision_id_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let rev_id = sentinel_domain::candidate::RevisionId::from_digest(revision_id_str);

    match state.store.get(tenant_id, &rev_id).await.map_err(|e| internal(e))? {
        Some(c) => Ok((StatusCode::OK, Json(CandidateDetailResponse::from(c)))),
        None => Err(not_found("Candidate revision not found")),
    }
}

async fn handle_list_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let limit = q.limit.unwrap_or(20).min(100);

    let items = state
        .store
        .list(tenant_id, limit, q.cursor.clone())
        .await
        .map_err(|e| internal(e))?;

    let candidates: Vec<CandidateDetailResponse> = items.into_iter().map(CandidateDetailResponse::from).collect();
    Ok((StatusCode::OK, Json(CandidateListResponse { candidates, next_cursor: None })))
}

// ─── Workflow Handlers ────────────────────────────────────────────────────────

async fn handle_create_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(req): Json<CreateCertificationRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let now = OffsetDateTime::now_utc();

    // Verify candidate revision exists
    state
        .store
        .get(tenant_id, &req.candidate_revision_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Candidate revision not found"))?;

    let mut workflow = Workflow::new(
        tenant_id,
        req.candidate_revision_id,
        req.policy_pack_id,
        req.corpus_version,
    );

    // Immediately transition Registered → Queued
    workflow
        .apply(WorkflowCommand::Start, workflow.version, now)
        .map_err(|e| conflict(e.to_string()))?;

    state.store.create(&workflow).await.map_err(|e| internal(e))?;

    let event = AuditEvent {
        event_id: Uuid::new_v4(),
        tenant_id,
        workflow_id: Some(workflow.workflow_id),
        case_run_id: None,
        event_type: AuditEventType::WorkflowStarted,
        actor: sentinel_domain::ids::PrincipalId::new("control-plane"),
        before_state: Some(WorkflowState::Registered),
        after_state: Some(WorkflowState::Queued),
        command: None,
        decision: None,
        explanation: Some("Certification workflow queued for execution".into()),
        provenance: sentinel_domain::provenance::Provenance::Live,
        timestamp: now,
        trace_id: None,
        span_id: None,
    };
    let _ = state.store.append(&event).await;

    info!("Workflow {} created and queued for tenant {}", workflow.workflow_id, tenant_id);
    dispatch_worker_job(&state.config).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                503,
                "Workflow Persisted; Dispatch Failed",
                format!("Workflow {} is queued and can be retried via /dispatch: {e}", workflow.workflow_id),
            )),
        )
    })?;
    Ok((StatusCode::ACCEPTED, Json(CreateCertificationResponse { workflow_id: workflow.workflow_id })))
}

async fn handle_dispatch_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;
    let workflow = state
        .store
        .get(tenant_id, workflow_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    if !matches!(workflow.state, WorkflowState::Queued | WorkflowState::RetestRequired) {
        return Err(conflict(format!("Workflow is in state {:?}; dispatch is only valid for queued or retest-required work", workflow.state)));
    }

    dispatch_worker_job(&state.config).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(503, "Worker Dispatch Failed", e)),
        )
    })?;

    Ok((StatusCode::ACCEPTED, Json(serde_json::json!({
        "workflow_id": workflow_id,
        "dispatch_status": "ACCEPTED"
    }))))
}

async fn handle_get_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;

    match state.store.get(tenant_id, workflow_id).await.map_err(|e| internal(e))? {
        Some(w) => Ok((StatusCode::OK, Json(WorkflowResponse::from(w)))),
        None => Err(not_found("Workflow not found")),
    }
}

async fn handle_list_workflows(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let limit = q.limit.unwrap_or(20).min(100);

    let state_filter = q.state.as_deref().and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_uppercase())).ok());

    let workflows = state
        .store
        .list(tenant_id, state_filter, limit, q.cursor.clone())
        .await
        .map_err(|e| internal(e))?;

    Ok((StatusCode::OK, Json(WorkflowListResponse { workflows, next_cursor: None })))
}

async fn handle_cancel_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;
    let now = OffsetDateTime::now_utc();

    let mut workflow = state
        .store
        .get(tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Workflow not found"))?;

    let expected = workflow.version;
    workflow
        .apply(WorkflowCommand::Cancel { reason: "Cancelled via API".into() }, expected, now)
        .map_err(|e| conflict(e.to_string()))?;

    state.store.update(&workflow, expected).await.map_err(|e| internal(e))?;
    Ok((StatusCode::OK, Json(WorkflowResponse::from(workflow))))
}

// ─── Findings Handler ─────────────────────────────────────────────────────────

async fn handle_get_findings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;

    let findings = state.store.list(tenant_id, workflow_id).await.map_err(|e| internal(e))?;
    Ok((StatusCode::OK, Json(FindingsListResponse { findings })))
}

// ─── Approval Handler ─────────────────────────────────────────────────────────

async fn handle_submit_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
    Json(req): Json<SubmitApprovalRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;
    let now = OffsetDateTime::now_utc();

    // Load workflow and verify it's awaiting approval
    let mut workflow = state
        .store
        .get(tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Workflow not found"))?;

    if workflow.state != WorkflowState::ApprovalRequired {
        return Err(conflict(format!("Workflow is in state {:?}, not ApprovalRequired", workflow.state)));
    }

    // Fetch candidate to enforce separation of duties
    let candidate = state
        .store
        .get(tenant_id, &workflow.candidate_revision_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Candidate revision not found"))?;

    if req.reviewer == candidate.abom.owner {
        return Err(forbidden("Separation of duties violation: candidate owner cannot approve their own workflow"));
    }

    // Approval MUST reduce capabilities — never broaden
    for cap in &req.approved_capabilities {
        if !candidate.abom.requested_capabilities.contains(cap)
            && !req.proposed_capabilities.contains(cap)
        {
            return Err(forbidden(format!(
                "Approval cannot grant capability '{}' not in candidate request",
                cap
            )));
        }
    }

    let approval_id = ApprovalId::new();
    let expires_at = now + time::Duration::seconds(req.duration_seconds);

    let approval = Approval {
        approval_id,
        tenant_id,
        workflow_id,
        reviewer: req.reviewer,
        reviewer_role: req.reviewer_role,
        proposed_capabilities: req.proposed_capabilities,
        approved_capabilities: req.approved_capabilities.clone(),
        reason_code: req.reason_code,
        note: req.note,
        issued_at: now,
        expires_at,
        policy_version: "1.0.0".into(),
    };

    state.store.create(&approval).await.map_err(|e| internal(e))?;

    let expected = workflow.version;
    workflow
        .apply(
            WorkflowCommand::Approve {
                approval_id,
                constrained_capabilities: req.approved_capabilities,
            },
            expected,
            now,
        )
        .map_err(|e| conflict(e.to_string()))?;

    state.store.update(&workflow, expected).await.map_err(|e| internal(e))?;

    let event = AuditEvent {
        event_id: Uuid::new_v4(),
        tenant_id,
        workflow_id: Some(workflow_id),
        case_run_id: None,
        event_type: AuditEventType::ApprovalRecorded,
        actor: approval.reviewer.clone(),
        before_state: Some(WorkflowState::ApprovalRequired),
        after_state: Some(WorkflowState::RetestRequired),
        command: None,
        decision: None,
        explanation: Some(format!("Constrained approval recorded — expires {}", expires_at)),
        provenance: sentinel_domain::provenance::Provenance::Live,
        timestamp: now,
        trace_id: None,
        span_id: None,
    };
    let _ = state.store.append(&event).await;

    info!("Approval {} recorded for workflow {}", approval_id, workflow_id);
    dispatch_worker_job(&state.config).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                503,
                "Approval Persisted; Retest Dispatch Failed",
                format!("Workflow {workflow_id} requires a /dispatch retry: {e}"),
            )),
        )
    })?;
    Ok((StatusCode::CREATED, Json(SubmitApprovalResponse { approval_id })))
}

async fn dispatch_worker_job(config: &ControlPlaneConfig) -> Result<(), String> {
    if !config.google_cloud.use_live_services {
        return Ok(());
    }

    let project = std::env::var("GOOGLE_CLOUD_PROJECT")
        .map_err(|_| "GOOGLE_CLOUD_PROJECT is not configured".to_string())?;
    let region = std::env::var("GOOGLE_CLOUD_REGION").unwrap_or_else(|_| "us-east1".to_string());
    let job = std::env::var("SENTINEL_WORKFLOW_JOB")
        .unwrap_or_else(|_| "sentinel-workflow-worker".to_string());
    let token = metadata_access_token().await?;
    let url = format!(
        "https://run.googleapis.com/v2/projects/{project}/locations/{region}/jobs/{job}:run"
    );
    let response = reqwest::Client::new()
        .post(url)
        .bearer_auth(token)
        .json(&serde_json::json!({}))
        .timeout(Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Cloud Run Jobs request failed: {e}"))?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response.text().await.unwrap_or_default();
    Err(format!("Cloud Run Jobs returned {status}: {body}"))
}

async fn metadata_access_token() -> Result<String, String> {
    let response = reqwest::Client::new()
        .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token")
        .header("Metadata-Flavor", "Google")
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| format!("Workload identity token request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!("Workload identity token request returned {}", response.status()));
    }
    let body: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Workload identity token response was invalid: {e}"))?;
    body["access_token"]
        .as_str()
        .map(str::to_string)
        .ok_or_else(|| "Workload identity token response omitted access_token".to_string())
}

// ─── Evidence Handler ─────────────────────────────────────────────────────────

async fn handle_get_evidence_manifest(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;

    let manifest = state
        .store
        .get_manifest(tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?;

    Ok((StatusCode::OK, Json(EvidenceManifestResponse { manifest })))
}

// ─── Attestation Handlers ─────────────────────────────────────────────────────

async fn handle_get_attestation(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str)
        .map_err(|_| (StatusCode::BAD_REQUEST, Json(ErrorResponse::new(400, "Bad Request", "Invalid workflow_id UUID"))))?;

    let attestation = state
        .store
        .get(tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?;

    Ok((StatusCode::OK, Json(AttestationResponse { attestation })))
}

async fn handle_verify_attestation(
    State(_state): State<AppState>,
    Json(req): Json<AttestationVerifyRequest>,
) -> impl IntoResponse {
    let now = OffsetDateTime::now_utc();
    let result = verify_attestation(
        &req.attestation,
        &req.tenant_id,
        &req.environment,
        None,
        None,
        now,
    );
    (
        StatusCode::OK,
        Json(AttestationVerifyResponse {
            valid: result.valid,
            details: result.error.unwrap_or_else(|| "Attestation is valid".into()),
        }),
    )
}

// ─── Fleet Posture ────────────────────────────────────────────────────────────

async fn handle_get_fleet_posture(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;

    let all = state.store.list(tenant_id, None, 200, None).await.map_err(|e| internal(e))?;

    let certified = all.iter().filter(|w| w.state == WorkflowState::Certified).count();
    let blocked = all.iter().filter(|w| w.state == WorkflowState::Blocked).count();
    let approval_required = all.iter().filter(|w| w.state == WorkflowState::ApprovalRequired).count();
    let running = all.iter().filter(|w| w.state == WorkflowState::Running).count();
    let total = all.len();

    Ok((
        StatusCode::OK,
        Json(serde_json::json!({
            "tenant_id": tenant_id,
            "total_workflows": total,
            "certified": certified,
            "blocked": blocked,
            "approval_required": approval_required,
            "running": running,
            "posture": if blocked > 0 || approval_required > 0 { "ATTENTION_REQUIRED" } else { "HEALTHY" },
            "provenance": "LIVE"
        })),
    ))
}
