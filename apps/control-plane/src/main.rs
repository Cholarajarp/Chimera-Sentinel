//! Chimera Sentinel Control Plane API Server.
//!
//! Provides REST APIs for candidate management, durable certification workflows,
//! findings, human approvals, evidence manifests, and KMS attestation verification.
//! All handlers are fully wired to the persistence layer.

mod corpus;

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
    timeout::TimeoutLayer,
    trace::TraceLayer,
};
use tracing::{info, warn};
use uuid::Uuid;

use sentinel_attestation::verify_attestation;
use sentinel_config::DatabaseConfig;
use sentinel_contracts::{
    approval::{SubmitApprovalRequest, SubmitApprovalResponse},
    attestation::{
        AttestationResponse, AttestationVerifyRequest, AttestationVerifyResponse, WorkflowResponse,
    },
    audit::AuditListResponse,
    candidate::{
        CandidateDetailResponse, CandidateListResponse, CandidateRevisionRequest,
        CandidateRevisionResponse,
    },
    corpus::CorpusResponse,
    evidence::EvidenceManifestResponse,
    finding::FindingsListResponse,
    workflow::{CreateCertificationRequest, CreateCertificationResponse, WorkflowListResponse},
    ErrorResponse,
};
use sentinel_domain::{
    attestation::AttestationMetadata,
    candidate::{CandidateRevision, ScanResponse},
    evidence::{Approval, EvidenceManifestRef},
    finding::Finding,
    ids::{ApprovalId, RevisionId, TenantId, WorkflowId},
    workflow::{Workflow, WorkflowCommand, WorkflowState, WorkflowSummary},
    AuditEvent, AuditEventType,
};
use sentinel_metrics::{gather_metrics, HTTP_REQUESTS};
use sentinel_observability::init_telemetry;
use sentinel_persistence::{
    firestore::FirestoreStore, memory::InMemoryStore, ApprovalRepository, AttestationRepository,
    AuditRepository, CandidateRepository, EvidenceRepository, FindingRepository,
    WorkflowRepository,
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

#[async_trait::async_trait]
impl CandidateRepository for Store {
    async fn create(&self, candidate: &CandidateRevision) -> Result<(), String> {
        match self {
            Store::Memory(s) => CandidateRepository::create(&**s, candidate).await,
            Store::Firestore(s) => CandidateRepository::create(&**s, candidate).await,
        }
    }
    async fn get(
        &self,
        tenant_id: TenantId,
        revision_id: &RevisionId,
    ) -> Result<Option<CandidateRevision>, String> {
        match self {
            Store::Memory(s) => CandidateRepository::get(&**s, tenant_id, revision_id).await,
            Store::Firestore(s) => CandidateRepository::get(&**s, tenant_id, revision_id).await,
        }
    }
    async fn list(
        &self,
        tenant_id: TenantId,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<Vec<CandidateRevision>, String> {
        match self {
            Store::Memory(s) => CandidateRepository::list(&**s, tenant_id, limit, cursor).await,
            Store::Firestore(s) => CandidateRepository::list(&**s, tenant_id, limit, cursor).await,
        }
    }
}
#[async_trait::async_trait]
impl WorkflowRepository for Store {
    async fn create(&self, workflow: &Workflow) -> Result<(), String> {
        match self {
            Store::Memory(s) => WorkflowRepository::create(&**s, workflow).await,
            Store::Firestore(s) => WorkflowRepository::create(&**s, workflow).await,
        }
    }
    async fn get(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<Workflow>, String> {
        match self {
            Store::Memory(s) => WorkflowRepository::get(&**s, tenant_id, workflow_id).await,
            Store::Firestore(s) => WorkflowRepository::get(&**s, tenant_id, workflow_id).await,
        }
    }
    async fn update(&self, workflow: &Workflow, expected_version: u64) -> Result<(), String> {
        match self {
            Store::Memory(s) => WorkflowRepository::update(&**s, workflow, expected_version).await,
            Store::Firestore(s) => {
                WorkflowRepository::update(&**s, workflow, expected_version).await
            }
        }
    }
    async fn list(
        &self,
        tenant_id: TenantId,
        state: Option<WorkflowState>,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<Vec<WorkflowSummary>, String> {
        match self {
            Store::Memory(s) => {
                WorkflowRepository::list(&**s, tenant_id, state, limit, cursor).await
            }
            Store::Firestore(s) => {
                WorkflowRepository::list(&**s, tenant_id, state, limit, cursor).await
            }
        }
    }
}
#[async_trait::async_trait]
impl EvidenceRepository for Store {
    async fn store_manifest(&self, manifest: &EvidenceManifestRef) -> Result<(), String> {
        match self {
            Store::Memory(s) => EvidenceRepository::store_manifest(&**s, manifest).await,
            Store::Firestore(s) => EvidenceRepository::store_manifest(&**s, manifest).await,
        }
    }
    async fn get_manifest(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<EvidenceManifestRef>, String> {
        match self {
            Store::Memory(s) => {
                EvidenceRepository::get_manifest(&**s, tenant_id, workflow_id).await
            }
            Store::Firestore(s) => {
                EvidenceRepository::get_manifest(&**s, tenant_id, workflow_id).await
            }
        }
    }
}
#[async_trait::async_trait]
impl AuditRepository for Store {
    async fn append(&self, event: &AuditEvent) -> Result<(), String> {
        match self {
            Store::Memory(s) => AuditRepository::append(&**s, event).await,
            Store::Firestore(s) => AuditRepository::append(&**s, event).await,
        }
    }
    async fn query(
        &self,
        tenant_id: TenantId,
        workflow_id: Option<WorkflowId>,
        limit: usize,
    ) -> Result<Vec<AuditEvent>, String> {
        match self {
            Store::Memory(s) => AuditRepository::query(&**s, tenant_id, workflow_id, limit).await,
            Store::Firestore(s) => {
                AuditRepository::query(&**s, tenant_id, workflow_id, limit).await
            }
        }
    }
}
#[async_trait::async_trait]
impl FindingRepository for Store {
    async fn store(&self, finding: &Finding) -> Result<(), String> {
        match self {
            Store::Memory(s) => FindingRepository::store(&**s, finding).await,
            Store::Firestore(s) => FindingRepository::store(&**s, finding).await,
        }
    }
    async fn list(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Vec<Finding>, String> {
        match self {
            Store::Memory(s) => FindingRepository::list(&**s, tenant_id, workflow_id).await,
            Store::Firestore(s) => FindingRepository::list(&**s, tenant_id, workflow_id).await,
        }
    }
}
#[async_trait::async_trait]
impl ApprovalRepository for Store {
    async fn create(&self, approval: &Approval) -> Result<(), String> {
        match self {
            Store::Memory(s) => ApprovalRepository::create(&**s, approval).await,
            Store::Firestore(s) => ApprovalRepository::create(&**s, approval).await,
        }
    }
    async fn get(
        &self,
        tenant_id: TenantId,
        approval_id: ApprovalId,
    ) -> Result<Option<Approval>, String> {
        match self {
            Store::Memory(s) => ApprovalRepository::get(&**s, tenant_id, approval_id).await,
            Store::Firestore(s) => ApprovalRepository::get(&**s, tenant_id, approval_id).await,
        }
    }
    async fn get_for_workflow(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<Approval>, String> {
        match self {
            Store::Memory(s) => {
                ApprovalRepository::get_for_workflow(&**s, tenant_id, workflow_id).await
            }
            Store::Firestore(s) => {
                ApprovalRepository::get_for_workflow(&**s, tenant_id, workflow_id).await
            }
        }
    }
}
#[async_trait::async_trait]
impl AttestationRepository for Store {
    async fn store(
        &self,
        workflow_id: WorkflowId,
        attestation: &AttestationMetadata,
    ) -> Result<(), String> {
        match self {
            Store::Memory(s) => AttestationRepository::store(&**s, workflow_id, attestation).await,
            Store::Firestore(s) => {
                AttestationRepository::store(&**s, workflow_id, attestation).await
            }
        }
    }
    async fn get(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<AttestationMetadata>, String> {
        match self {
            Store::Memory(s) => AttestationRepository::get(&**s, tenant_id, workflow_id).await,
            Store::Firestore(s) => AttestationRepository::get(&**s, tenant_id, workflow_id).await,
        }
    }
}

#[derive(Clone)]
struct AppState {
    store: Arc<Store>,
    config: Arc<ControlPlaneConfig>,
    /// Digest-verified evaluation corpus, loaded once at startup.
    /// `None` when the corpus is absent or failed verification.
    corpus: Option<Arc<CorpusResponse>>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // A silent fallback to defaults downgrades production to the non-durable
    // in-memory store, so make the parse failure loud on stderr.
    let config: ControlPlaneConfig = match load_config(&args.config) {
        Ok(config) => config,
        Err(error) => {
            eprintln!(
                "CONFIG ERROR: {} could not be parsed ({error}). Falling back to defaults; \
                 persistence will NOT be durable.",
                args.config
            );
            ControlPlaneConfig::default()
        }
    };
    // Unified telemetry bootstrap: structured JSON logging (always on) plus
    // optional OTLP export to Cloud Trace when a real `otel_endpoint` is
    // configured. The GCP project becomes the `gcp.project_id` resource
    // attribute on exported spans so they land in the right Cloud Trace
    // project. Do NOT call `sentinel_config::init_logging` here — owning a
    // single global subscriber here avoids the double-init race.
    let _guard = init_telemetry(
        &config.observability,
        Some(&config.google_cloud.firestore_project),
    )
    .map_err(anyhow::Error::msg)?;

    info!("Starting Chimera Sentinel Control Plane v0.1.0");
    info!("Server addr: {}", args.addr);
    info!(
        "Live Google Cloud services: {}",
        config.google_cloud.use_live_services
    );

    // Select persistence backend based on config.
    // DatabaseConfig::Firestore → durable Cloud Firestore (required for production)
    // DatabaseConfig::Memory    → in-process (local dev / tests only)
    let store: Arc<Store> = match &config.database {
        DatabaseConfig::Firestore {
            project_id,
            database_id,
        } => {
            let db = database_id
                .clone()
                .unwrap_or_else(|| "(default)".to_string());
            info!(
                "Persistence backend: Firestore project={} database={}",
                project_id, db
            );
            Arc::new(Store::Firestore(Arc::new(FirestoreStore::new(
                project_id, db,
            ))))
        }
        _ => {
            warn!(
                "Persistence backend: InMemoryStore — state is NOT durable across restarts. \
                   Set database.type=firestore in config for production."
            );
            Arc::new(Store::Memory(Arc::new(InMemoryStore::new())))
        }
    };

    // Load and digest-verify the evaluation corpus once at startup so request
    // handlers never touch the filesystem and integrity is known before serving.
    let corpus = corpus::load().map(Arc::new);

    let state = AppState {
        store,
        config: Arc::new(config),
        corpus,
    };

    let environment =
        std::env::var("SENTINEL_ENVIRONMENT").unwrap_or_else(|_| "development".to_string());
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
        .layer(axum::extract::DefaultBodyLimit::max(1024 * 1024))
        .layer(TimeoutLayer::with_status_code(
            axum::http::StatusCode::REQUEST_TIMEOUT,
            Duration::from_secs(30),
        ))
        .layer(cors)
        .layer(axum::middleware::from_fn(metrics_middleware));

    let app = Router::new()
        .route("/healthz", get(health_check))
        .route("/readyz", get(readiness_check))
        .route("/metrics", get(metrics_endpoint))
        // Candidate management
        .route(
            "/v1/candidates",
            post(handle_create_candidate).get(handle_list_candidates),
        )
        .route("/v1/candidates/:revision_id", get(handle_get_candidate))
        // On-Demand vulnerability scanning for a candidate's container image.
        // Returns live CVEs (provenance: LIVE) when GCP creds are available, an
        // honest empty result (provenance: LOCAL) when they are not.
        .route(
            "/v1/candidates/:revision_id/scan",
            get(handle_scan_candidate),
        )
        // Workflow lifecycle
        .route(
            "/v1/workflows",
            post(handle_create_workflow).get(handle_list_workflows),
        )
        .route("/v1/workflows/:workflow_id", get(handle_get_workflow))
        .route(
            "/v1/workflows/:workflow_id/dispatch",
            post(handle_dispatch_workflow),
        )
        .route(
            "/v1/workflows/:workflow_id/cancel",
            post(handle_cancel_workflow),
        )
        // Findings, approvals, evidence, attestation
        .route(
            "/v1/workflows/:workflow_id/findings",
            get(handle_get_findings),
        )
        .route("/v1/workflows/:workflow_id/audit", get(handle_get_audit))
        .route(
            "/v1/workflows/:workflow_id/approvals",
            post(handle_submit_approval),
        )
        .route(
            "/v1/workflows/:workflow_id/evidence",
            get(handle_get_evidence_manifest),
        )
        .route(
            "/v1/workflows/:workflow_id/attestation",
            get(handle_get_attestation),
        )
        // Standalone verification
        .route("/v1/attestations/verify", post(handle_verify_attestation))
        // Fleet posture
        .route("/v1/fleet/posture", get(handle_get_fleet_posture))
        // Active policy pack for tenant
        .route("/v1/policy", get(handle_get_policy))
        // Versioned evaluation corpus (digest-verified case definitions)
        .route("/v1/corpus", get(handle_get_corpus))
        .layer(middleware)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(args.addr).await?;
    info!("Control plane listening on {}", args.addr);

    axum::serve(listener, app)
        .with_graceful_shutdown(async {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to install Ctrl+C handler");
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
    (
        StatusCode::OK,
        Json(
            serde_json::json!({"status":"healthy","service":"sentinel-control-plane","version":"0.1.0"}),
        ),
    )
}

async fn readiness_check(State(state): State<AppState>) -> impl IntoResponse {
    let mut checks = vec![];
    let mut all_healthy = true;

    // Report the configured backend, never a hardcoded label.
    let backend = match state.config.database {
        DatabaseConfig::Firestore { .. } => "firestore",
        DatabaseConfig::Postgres { .. } => "postgres",
        DatabaseConfig::Memory => "in_memory",
    };
    let durable = backend != "in_memory";

    // Database reachability
    let system_tenant = TenantId::parse("00000000-0000-0000-0000-000000000001").unwrap_or_default();
    match WorkflowRepository::list(&*state.store, system_tenant, None, 1, None).await {
        Ok(_) => checks.push(
            serde_json::json!({"name":"database","status":"ok","type":backend,"durable":durable}),
        ),
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

    let code = if all_healthy {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        code,
        Json(
            serde_json::json!({"status": if all_healthy {"ready"} else {"not_ready"}, "checks": checks}),
        ),
    )
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
                Json(ErrorResponse::new(
                    400,
                    "Missing Tenant Header",
                    "X-Tenant-ID header with a valid UUID is required",
                )),
            )
        })
}

fn not_found(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new(404, "Not Found", detail)),
    )
}

fn internal(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponse::new(500, "Internal Server Error", detail)),
    )
}

fn conflict(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::CONFLICT,
        Json(ErrorResponse::new(409, "Conflict", detail)),
    )
}

fn forbidden(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::FORBIDDEN,
        Json(ErrorResponse::new(403, "Forbidden", detail)),
    )
}

fn service_unavailable(detail: impl Into<String>) -> (StatusCode, Json<ErrorResponse>) {
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(ErrorResponse::new(503, "Service Unavailable", detail)),
    )
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

    CandidateRepository::create(&*state.store, &candidate)
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
    let _ = AuditRepository::append(&*state.store, &event).await;

    Ok((
        StatusCode::CREATED,
        Json(CandidateRevisionResponse { revision_id }),
    ))
}

async fn handle_get_candidate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(revision_id_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let rev_id = RevisionId(revision_id_str.to_string());

    match CandidateRepository::get(&*state.store, tenant_id, &rev_id)
        .await
        .map_err(|e| internal(e))?
    {
        Some(c) => Ok((StatusCode::OK, Json(CandidateDetailResponse::from(c)))),
        None => Err(not_found("Candidate revision not found")),
    }
}

/// `GET /v1/candidates/:revision_id/scan` — On-Demand vulnerability scan.
///
/// Scans the candidate agent's container image via the GCP On-Demand Scanning
/// API (`ondemandscanning.googleapis.com`). Behavior is strictly honest:
/// - Without live GCP services enabled, OR when no GCP access token is
///   reachable (local/CI), returns an empty [`ScanResponse`] with
///   [`Provenance::Local`] — never fabricates CVEs.
/// - When the candidate carries no resolvable container-image reference (and
///   no `SENTINEL_SCAN_IMAGE` override is set), returns an empty result with
///   `status = "NO_SCANNABLE_IMAGE"` and [`Provenance::Local`].
/// - When a real scan runs, returns upstream CVEs with [`Provenance::Live`].
/// - A live scan that fails upstream is surfaced as 503 (Service Unavailable)
///   rather than a misleading "clean" result.
async fn handle_scan_candidate(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(revision_id_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let rev_id = RevisionId(revision_id_str.to_string());

    let Some(candidate) = CandidateRepository::get(&*state.store, tenant_id, &rev_id)
        .await
        .map_err(|e| internal(e))?
    else {
        return Err(not_found("Candidate revision not found"));
    };

    // ── Honest LOCAL fallback (no GCP creds / not live) ──────────────────────
    let local_empty = || ScanResponse {
        status: "LOCAL".to_string(),
        provenance: sentinel_domain::provenance::Provenance::Local,
        vulnerabilities: Vec::new(),
    };

    if !state.config.google_cloud.use_live_services {
        return Ok((StatusCode::OK, Json(local_empty())));
    }

    // Reach the metadata server for a short-lived access token. Failure means
    // we are not on GCE / lack on-demand-scanning scope → honest LOCAL.
    let access_token = match metadata_access_token().await {
        Ok(t) => t,
        Err(e) => {
            warn!("Vulnerability scan downgraded to LOCAL (no GCP creds): {e}");
            return Ok((StatusCode::OK, Json(local_empty())));
        }
    };

    // Resolve the image to scan. Prefer an operator override, then an
    // Artifact/GCR Registry image reference carried by the ABOM. If neither is
    // resolvable, return an honest LOCAL empty result — never fabricate data.
    let resource_uri = std::env::var("SENTINEL_SCAN_IMAGE")
        .ok()
        .filter(|s| !s.trim().is_empty())
        .or_else(|| {
            let r = &candidate.abom.registry_resource;
            (r.contains("pkg.dev") || r.contains("gcr.io")).then(|| r.clone())
        });

    let Some(resource_uri) = resource_uri else {
        return Ok((
            StatusCode::OK,
            Json(ScanResponse {
                status: "NO_SCANNABLE_IMAGE".to_string(),
                provenance: sentinel_domain::provenance::Provenance::Local,
                vulnerabilities: Vec::new(),
            }),
        ));
    };

    let project = std::env::var("GOOGLE_CLOUD_PROJECT")
        .unwrap_or_else(|_| state.config.google_cloud.firestore_project.clone());
    let location = std::env::var("GOOGLE_CLOUD_REGION").unwrap_or_else(|_| "us-east1".to_string());

    let target = sentinel_google_adapters::artifact_registry::ScanTarget {
        project: &project,
        location: &location,
        resource_uri: &resource_uri,
    };
    match sentinel_google_adapters::artifact_registry::scan(&target, &access_token).await {
        Ok(scan) => {
            info!(
                "Live vulnerability scan for candidate {} reported {} CVEs",
                rev_id,
                scan.vulnerabilities.len()
            );
            Ok((StatusCode::OK, Json(scan)))
        }
        Err(e) => {
            warn!("On-Demand vulnerability scan failed for {rev_id}: {e}");
            Err(service_unavailable(format!(
                "vulnerability scan failed: {e}"
            )))
        }
    }
}

async fn handle_list_candidates(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(q): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let limit = q.limit.unwrap_or(20).min(100);

    let items = CandidateRepository::list(&*state.store, tenant_id, limit, q.cursor.clone())
        .await
        .map_err(|e| internal(e))?;

    let candidates: Vec<CandidateDetailResponse> = items
        .into_iter()
        .map(CandidateDetailResponse::from)
        .collect();
    Ok((
        StatusCode::OK,
        Json(CandidateListResponse {
            candidates,
            next_cursor: None,
        }),
    ))
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
    CandidateRepository::get(&*state.store, tenant_id, &req.candidate_revision_id)
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

    WorkflowRepository::create(&*state.store, &workflow)
        .await
        .map_err(|e| internal(e))?;

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
    let _ = AuditRepository::append(&*state.store, &event).await;

    info!(
        "Workflow {} created and queued for tenant {}",
        workflow.workflow_id, tenant_id
    );
    dispatch_worker_job(&state.config).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(
                503,
                "Workflow Persisted; Dispatch Failed",
                format!(
                    "Workflow {} is queued and can be retried via /dispatch: {e}",
                    workflow.workflow_id
                ),
            )),
        )
    })?;
    Ok((
        StatusCode::ACCEPTED,
        Json(CreateCertificationResponse {
            workflow_id: workflow.workflow_id,
        }),
    ))
}

async fn handle_dispatch_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;
    let workflow = WorkflowRepository::get(&*state.store, tenant_id, workflow_id)
        .await
        .map_err(internal)?
        .ok_or_else(|| not_found("Workflow not found"))?;

    if !matches!(
        workflow.state,
        WorkflowState::Queued | WorkflowState::RetestRequired
    ) {
        return Err(conflict(format!(
            "Workflow is in state {:?}; dispatch is only valid for queued or retest-required work",
            workflow.state
        )));
    }

    dispatch_worker_job(&state.config).await.map_err(|e| {
        (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponse::new(503, "Worker Dispatch Failed", e)),
        )
    })?;

    Ok((
        StatusCode::ACCEPTED,
        Json(serde_json::json!({
            "workflow_id": workflow_id,
            "dispatch_status": "ACCEPTED"
        })),
    ))
}

async fn handle_get_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;

    match WorkflowRepository::get(&*state.store, tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?
    {
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

    let state_filter = q
        .state
        .as_deref()
        .and_then(|s| serde_json::from_value(serde_json::Value::String(s.to_uppercase())).ok());

    let workflows = WorkflowRepository::list(
        &*state.store,
        tenant_id,
        state_filter,
        limit,
        q.cursor.clone(),
    )
    .await
    .map_err(|e| internal(e))?;

    Ok((
        StatusCode::OK,
        Json(WorkflowListResponse {
            workflows,
            next_cursor: None,
        }),
    ))
}

async fn handle_cancel_workflow(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;
    let now = OffsetDateTime::now_utc();

    let mut workflow = WorkflowRepository::get(&*state.store, tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Workflow not found"))?;

    let expected = workflow.version;
    workflow
        .apply(
            WorkflowCommand::Cancel {
                reason: "Cancelled via API".into(),
            },
            expected,
            now,
        )
        .map_err(|e| conflict(e.to_string()))?;

    WorkflowRepository::update(&*state.store, &workflow, expected)
        .await
        .map_err(|e| internal(e))?;
    Ok((StatusCode::OK, Json(WorkflowResponse::from(workflow))))
}

// ─── Findings Handler ─────────────────────────────────────────────────────────

async fn handle_get_findings(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;

    let findings = FindingRepository::list(&*state.store, tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?;
    Ok((StatusCode::OK, Json(FindingsListResponse { findings })))
}

// ─── Audit Handler ────────────────────────────────────────────────────────────

/// Returns the append-only audit trail recorded for a workflow.
async fn handle_get_audit(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
    Query(query): Query<ListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;

    let limit = query.limit.unwrap_or(100).min(500);
    let events = AuditRepository::query(&*state.store, tenant_id, Some(workflow_id), limit)
        .await
        .map_err(|e| internal(e))?;
    Ok((StatusCode::OK, Json(AuditListResponse { events })))
}

// ─── Approval Handler ─────────────────────────────────────────────────────────

async fn handle_submit_approval(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(wid_str): Path<String>,
    Json(req): Json<SubmitApprovalRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;
    let now = OffsetDateTime::now_utc();

    // Load workflow and verify it's awaiting approval
    let mut workflow = WorkflowRepository::get(&*state.store, tenant_id, workflow_id)
        .await
        .map_err(|e| internal(e))?
        .ok_or_else(|| not_found("Workflow not found"))?;

    if workflow.state != WorkflowState::ApprovalRequired {
        return Err(conflict(format!(
            "Workflow is in state {:?}, not ApprovalRequired",
            workflow.state
        )));
    }

    // Fetch candidate to enforce separation of duties
    let candidate =
        CandidateRepository::get(&*state.store, tenant_id, &workflow.candidate_revision_id)
            .await
            .map_err(|e| internal(e))?
            .ok_or_else(|| not_found("Candidate revision not found"))?;

    if req.reviewer == candidate.abom.owner {
        return Err(forbidden(
            "Separation of duties violation: candidate owner cannot approve their own workflow",
        ));
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

    ApprovalRepository::create(&*state.store, &approval)
        .await
        .map_err(|e| internal(e))?;

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

    WorkflowRepository::update(&*state.store, &workflow, expected)
        .await
        .map_err(|e| internal(e))?;

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
        explanation: Some(format!(
            "Constrained approval recorded — expires {}",
            expires_at
        )),
        provenance: sentinel_domain::provenance::Provenance::Live,
        timestamp: now,
        trace_id: None,
        span_id: None,
    };
    let _ = AuditRepository::append(&*state.store, &event).await;

    info!(
        "Approval {} recorded for workflow {}",
        approval_id, workflow_id
    );
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
    Ok((
        StatusCode::CREATED,
        Json(SubmitApprovalResponse { approval_id }),
    ))
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
        return Err(format!(
            "Workload identity token request returned {}",
            response.status()
        ));
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
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;

    let manifest = EvidenceRepository::get_manifest(&*state.store, tenant_id, workflow_id)
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
    let workflow_id = WorkflowId::parse(&wid_str).map_err(|_| {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse::new(
                400,
                "Bad Request",
                "Invalid workflow_id UUID",
            )),
        )
    })?;

    let attestation = AttestationRepository::get(&*state.store, tenant_id, workflow_id)
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
            details: result
                .error
                .unwrap_or_else(|| "Attestation is valid".into()),
        }),
    )
}

// ─── Fleet Posture ────────────────────────────────────────────────────────────

async fn handle_get_fleet_posture(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let tenant_id = extract_tenant(&headers)?;

    let all = WorkflowRepository::list(&*state.store, tenant_id, None, 200, None)
        .await
        .map_err(|e| internal(e))?;

    let certified = all
        .iter()
        .filter(|w| w.state == WorkflowState::Certified)
        .count();
    let blocked = all
        .iter()
        .filter(|w| w.state == WorkflowState::Blocked)
        .count();
    let approval_required = all
        .iter()
        .filter(|w| w.state == WorkflowState::ApprovalRequired)
        .count();
    let running = all
        .iter()
        .filter(|w| w.state == WorkflowState::Running)
        .count();
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

// ─── Evaluation Corpus ────────────────────────────────────────────────────────

/// Serves digest-verified evaluation-case definitions.
///
/// Returns 503 rather than approximated data when the corpus is unavailable, so
/// the console can never present reconstructed cases as real ones.
async fn handle_get_corpus(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let _tenant_id = extract_tenant(&headers)?;

    let corpus = state.corpus.as_ref().ok_or_else(|| {
        service_unavailable(
            "Evaluation corpus is not loaded. Ship corpus/v1 with the deployment or set \
             SENTINEL_CORPUS_DIR.",
        )
    })?;

    Ok((StatusCode::OK, Json(corpus.as_ref().clone())))
}

/// `GET /v1/policy` — returns the active policy pack loaded from disk.
///
/// Reads the policy pack from `SENTINEL_POLICY_PACK` (defaulting to
/// `/app/policy-packs/ap-agent-v1/pack.json`) so the console displays
/// the real rules rather than hardcoded HTML.
async fn handle_get_policy(
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let _tenant_id = extract_tenant(&headers)?;

    let path = std::env::var("SENTINEL_POLICY_PACK")
        .unwrap_or_else(|_| "/app/policy-packs/ap-agent-v1/pack.json".to_string());

    let bytes = tokio::fs::read(&path).await.map_err(|e| {
        service_unavailable(format!(
            "Policy pack not found at {path}: {e}. Deploy with policy-packs/ap-agent-v1/."
        ))
    })?;

    let pack: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| internal(format!("Policy pack at {path} is not valid JSON: {e}")))?;

    Ok((StatusCode::OK, Json(pack)))
}
