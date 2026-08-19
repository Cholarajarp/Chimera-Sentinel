//! Chimera Sentinel Asynchronous Workflow Worker.
//!
//! Executes certification test batches, coordinates Model Armor and Agent Gateway
//! inspections, evaluates deterministic policy against the ledger oracle, handles
//! human approvals, executes smoke retests, and finalizes signed KMS attestations.

use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use sentinel_config::{load_config, HealthCheckConfig, WorkerConfig, WorkflowWorkerConfig};
use sentinel_metrics::{
    gather_metrics, ACTIVE_WORKFLOWS, ATTESTATION_OPERATIONS, POLICY_DECISIONS,
    WORKFLOW_STATE_TRANSITIONS,
};
use std::sync::Arc;
use std::time::Duration;
use time::OffsetDateTime;
use tokio::net::TcpListener;
use tracing::{error, info, warn, Instrument};
use uuid::Uuid;

use sentinel_attestation::{build_attestation_payload, sign_attestation};
use sentinel_config::DatabaseConfig;
use sentinel_domain::{
    attestation::{AttestationMetadata, KeyReference},
    candidate::CandidateRevision,
    evidence::{Approval, CaseEvidence, EvidenceManifestRef},
    finding::{Finding, FindingKind, FindingSeverity, Remediation, RemediationAction},
    ids::{ApprovalId, CaseId, CaseRunId, RevisionId, TenantId, WorkflowId},
    ledger::{LedgerInvariants, LedgerSnapshot},
    policy::GateDecision,
    provenance::Provenance,
    workflow::{Workflow, WorkflowCommand, WorkflowState, WorkflowSummary},
    AuditEvent, AuditEventType,
};
use sentinel_evidence::{sha256_digest, EvidenceManifestBuilder};
use sentinel_google_adapters::{gateway, model_armor};
use sentinel_observability::init_telemetry;
use sentinel_persistence::{
    firestore::FirestoreStore, memory::InMemoryStore, ApprovalRepository, AttestationRepository,
    AuditRepository, CandidateRepository, EvidenceRepository, FindingRepository,
    WorkflowRepository,
};

/// Unified store enum — same as in control-plane.
#[derive(Clone)]
pub enum Store {
    /// In-memory storage backend for local development and testing
    Memory(Arc<InMemoryStore>),
    /// Firestore storage backend for production deployments
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
use sentinel_policy::{evaluate, EvidenceBundle, RetestResult};

#[derive(Parser)]
#[command(name = "sentinel-workflow-worker")]
struct Args {
    #[arg(long, default_value = "/app/config/workflow-worker.toml")]
    config: String,
    #[arg(long, default_value = "worker-1")]
    worker_id: String,
    #[arg(long, default_value = "5")]
    poll_interval_secs: u64,
    #[arg(long, default_value = "9091")]
    metrics_port: u16,
    #[arg(long, default_value_t = false)]
    once: bool,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Load configuration
    let config: WorkflowWorkerConfig =
        load_config(&args.config).map_err(|e| anyhow::anyhow!("Failed to load config: {}", e))?;

    // Unified telemetry bootstrap: JSON logging (always) + OTLP export to
    // Cloud Trace when a real `otel_endpoint` is configured. The GCP project
    // becomes the `gcp.project_id` resource attribute on exported spans. Hold
    // the guard for the process lifetime so spans flush on shutdown.
    let _guard = init_telemetry(
        &config.observability,
        Some(&config.google_cloud.firestore_project),
    )
    .map_err(anyhow::Error::msg)?;

    info!("Starting Chimera Sentinel Workflow Worker");
    info!("Config loaded from: {}", args.config);
    info!("Worker ID: {}", config.worker.id);
    info!("Poll interval: {}s", config.worker.poll_interval_secs);
    info!(
        "Max concurrent workflows: {}",
        config.worker.max_concurrent_workflows
    );
    info!(
        "Google Cloud live services: {}",
        config.google_cloud.use_live_services
    );
    info!("Metrics server on port: {}", args.metrics_port);

    // Select store backend from config
    let store = Arc::new(match &config.database {
        DatabaseConfig::Firestore {
            project_id,
            database_id,
        } => Store::Firestore(Arc::new(FirestoreStore::new(
            project_id.clone(),
            database_id
                .clone()
                .unwrap_or_else(|| "(default)".to_string()),
        ))),
        _ => Store::Memory(Arc::new(InMemoryStore::new())),
    });

    if args.once {
        info!("Running one-shot workflow processing cycle");
        process_pending_workflows(
            &store,
            &config.worker,
            &config.google_cloud,
            &config.certification,
        )
        .await
        .map_err(anyhow::Error::msg)?;
        info!("One-shot workflow processing cycle complete");
        return Ok(());
    }

    // Create shutdown channel
    let (shutdown_tx, _shutdown_rx) = tokio::sync::broadcast::channel(1);

    // Spawn metrics server
    let metrics_shutdown = shutdown_tx.subscribe();
    let metrics_port = args.metrics_port;
    let metrics_worker_id = config.worker.id.clone();
    let metrics_handle = tokio::spawn(async move {
        run_metrics_server(metrics_port, metrics_worker_id, metrics_shutdown).await
    });

    // Spawn worker task — store is Arc<Store> (Firestore or Memory)
    let worker_store = store.clone();
    let worker_config = config.worker.clone();
    let google_config = config.google_cloud.clone();
    let cert_config = config.certification.clone();
    let health_config = config.health_checks.clone();
    let worker_id = config.worker.id.clone();

    let shutdown_tx_clone = shutdown_tx.clone();
    let worker_handle = tokio::spawn(async move {
        run_worker(
            worker_store,
            worker_config,
            google_config,
            cert_config,
            health_config,
            worker_id,
            shutdown_tx_clone,
        )
        .await
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to install Ctrl+C handler");
    info!("Shutdown signal received, stopping worker...");

    // Signal worker to shutdown
    let _ = shutdown_tx.send(());

    // Wait for worker to finish with timeout
    let shutdown_timeout = Duration::from_secs(config.worker.shutdown_timeout_secs);
    match tokio::time::timeout(shutdown_timeout, worker_handle).await {
        Ok(Ok(_)) => info!("Worker shut down gracefully"),
        Ok(Err(e)) => error!("Worker panicked: {}", e),
        Err(_) => warn!("Worker shutdown timeout exceeded, forcing exit"),
    }

    // Wait for metrics server to finish
    let _ = tokio::time::timeout(Duration::from_secs(5), metrics_handle).await;

    info!("Workflow worker shutdown complete");
    Ok(())
}

async fn run_metrics_server(
    port: u16,
    _worker_id: String,
    mut shutdown_rx: tokio::sync::broadcast::Receiver<()>,
) -> Result<(), String> {
    let app = Router::new()
        .route("/metrics", get(metrics_endpoint))
        .route("/healthz", get(|| async { "ok" }));

    let listener = TcpListener::bind(("0.0.0.0", port))
        .await
        .map_err(|e| format!("Failed to bind metrics server: {}", e))?;
    info!("Metrics server listening on port {}", port);

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = shutdown_rx.recv().await;
        })
        .await
        .map_err(|e| format!("Metrics server error: {}", e))?;

    info!("Metrics server shut down");
    Ok(())
}

async fn metrics_endpoint() -> impl IntoResponse {
    match gather_metrics() {
        Ok(metrics) => metrics,
        Err(e) => format!("Error gathering metrics: {}", e),
    }
}

async fn run_worker(
    store: Arc<Store>,
    worker_config: WorkerConfig,
    google_config: sentinel_config::GoogleCloudConfig,
    cert_config: sentinel_config::CertificationConfig,
    _health_config: HealthCheckConfig,
    worker_id: String,
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
) -> Result<(), String> {
    let mut shutdown_rx = shutdown_tx.subscribe();

    loop {
        tokio::select! {
            _ = shutdown_rx.recv() => {
                info!("Worker {} received shutdown signal", worker_id);
                break;
            }
            _ = tokio::time::sleep(Duration::from_secs(worker_config.poll_interval_secs)) => {
                // Update active workflows gauge
                let tenant_id = TenantId::parse(&worker_config.tenant_id)
                    .map_err(|e| format!("Invalid configured worker tenant_id: {e}"))?;
                let running = WorkflowRepository::list(&*store, tenant_id, Some(WorkflowState::Running), 100, None).await.unwrap_or_default().len();
                ACTIVE_WORKFLOWS.with_label_values(&[&worker_id]).set(running as f64);

                // Run worker processing cycle
                if let Err(e) = process_pending_workflows(
                    &store,
                    &worker_config,
                    &google_config,
                    &cert_config,
                ).await {
                    error!("Error in worker cycle: {}", e);
                }
            }
        }
    }

    ACTIVE_WORKFLOWS.with_label_values(&[&worker_id]).set(0.0);
    info!("Worker {} stopped", worker_id);
    Ok(())
}

async fn process_pending_workflows(
    store: &Arc<Store>,
    worker_config: &WorkerConfig,
    google_config: &sentinel_config::GoogleCloudConfig,
    cert_config: &sentinel_config::CertificationConfig,
) -> Result<(), String> {
    let tenant_id = TenantId::parse(&worker_config.tenant_id)
        .map_err(|e| format!("Invalid configured worker tenant_id: {e}"))?;
    let worker_id = worker_config.id.as_str();

    // Query for Queued workflows
    let queued =
        WorkflowRepository::list(&**store, tenant_id, Some(WorkflowState::Queued), 10, None)
            .await?;
    for summary in queued {
        if let Some(mut workflow) =
            WorkflowRepository::get(&**store, tenant_id, summary.workflow_id).await?
        {
            let now = OffsetDateTime::now_utc();
            let expected_version = workflow.version;

            // Transition: Queued -> Running
            workflow
                .apply(
                    WorkflowCommand::Claim {
                        worker_id: worker_id.to_string(),
                    },
                    expected_version,
                    now,
                )
                .map_err(|e| e.to_string())?;
            WorkflowRepository::update(&**store, &workflow, expected_version).await?;

            // Record state transition metric
            WORKFLOW_STATE_TRANSITIONS
                .with_label_values(&["Queued", "Running", worker_id])
                .inc();

            info!(
                "Claimed workflow {}, executing certification cases",
                workflow.workflow_id
            );

            // Execute certification run
            if let Err(error) = execute_workflow_certification(
                store,
                &mut workflow,
                google_config,
                cert_config,
                worker_id,
            )
            .await
            {
                fail_workflow(store, &mut workflow, &error).await?;
                return Err(error);
            }
        }
    }

    // Query for RetestRequired workflows (human approval granted)
    let retest_pending = WorkflowRepository::list(
        &**store,
        tenant_id,
        Some(WorkflowState::RetestRequired),
        10,
        None,
    )
    .await?;
    for summary in retest_pending {
        if let Some(mut workflow) =
            WorkflowRepository::get(&**store, tenant_id, summary.workflow_id).await?
        {
            let now = OffsetDateTime::now_utc();
            let expected_version = workflow.version;

            // Transition: RetestRequired -> Running
            workflow
                .apply(WorkflowCommand::DispatchRetest, expected_version, now)
                .map_err(|e| e.to_string())?;
            WorkflowRepository::update(&**store, &workflow, expected_version).await?;

            // Record state transition metric
            WORKFLOW_STATE_TRANSITIONS
                .with_label_values(&["RetestRequired", "Running", worker_id])
                .inc();

            info!(
                "Executing post-approval smoke retest for workflow {}",
                workflow.workflow_id
            );
            if let Err(error) =
                execute_workflow_retest(store, &mut workflow, google_config, cert_config, worker_id)
                    .await
            {
                if !workflow.state.is_terminal() {
                    fail_workflow(store, &mut workflow, &error).await?;
                }
                return Err(error);
            }
        }
    }

    Ok(())
}

async fn fail_workflow(
    store: &Arc<Store>,
    workflow: &mut sentinel_domain::workflow::Workflow,
    reason: &str,
) -> Result<(), String> {
    if workflow.state.is_terminal() {
        return Ok(());
    }
    let expected_version = workflow.version;
    workflow
        .apply(
            WorkflowCommand::Fail {
                reason: reason.to_string(),
            },
            expected_version,
            OffsetDateTime::now_utc(),
        )
        .map_err(|e| e.to_string())?;
    WorkflowRepository::update(&**store, workflow, expected_version).await
}

async fn execute_workflow_certification(
    store: &Arc<Store>,
    workflow: &mut sentinel_domain::workflow::Workflow,
    google_config: &sentinel_config::GoogleCloudConfig,
    cert_config: &sentinel_config::CertificationConfig,
    worker_id: &str,
) -> Result<(), String> {
    // W3C traceparent trace-id: a fresh 32-hex randomness per workflow so every
    // certification run contributes its own distributed trace instead of sharing
    // a static example ID. The span carries `trace_id` and `workflow_id` so each
    // exported Cloud Trace span correlates back to exactly one certification
    // run (the same trace_id is sent to the ADK certifier in the eval request).
    let trace_id = Uuid::new_v4().as_simple().to_string();
    // Capture workflow_id as an owned value for the span so the span does not
    // borrow `workflow` (`workflow` is passed mutably into the inner call).
    let workflow_id = workflow.workflow_id.to_string();
    let span = tracing::info_span!(
        "certification",
        trace_id = %trace_id,
        workflow_id = %workflow_id,
    );
    execute_workflow_certification_inner(
        store,
        workflow,
        google_config,
        cert_config,
        worker_id,
        &trace_id,
    )
    .instrument(span)
    .await
}

async fn execute_workflow_certification_inner(
    store: &Arc<Store>,
    workflow: &mut sentinel_domain::workflow::Workflow,
    google_config: &sentinel_config::GoogleCloudConfig,
    cert_config: &sentinel_config::CertificationConfig,
    worker_id: &str,
    trace_id: &str,
) -> Result<(), String> {
    let now = OffsetDateTime::now_utc();
    let tenant_id = workflow.tenant_id;
    let candidate = CandidateRepository::get(&**store, tenant_id, &workflow.candidate_revision_id)
        .await?
        .ok_or_else(|| "Candidate revision not found".to_string())?;

    let required_cases = resolve_required_cases(&cert_config.required_cases).await?;

    info!(
        "Executing certification for workflow {} with {} cases via ADK certifier",
        workflow.workflow_id,
        required_cases.len()
    );

    // ── Call ADK Certifier HTTP API ───────────────────────────────────────────
    // The ADK certifier runs all cases with real Google ADK + Gemini + Model Armor + Gateway.
    // SENTINEL_ADK_CERTIFIER_URL is injected as an env var at Cloud Run deploy time.
    let adk_url = std::env::var("SENTINEL_ADK_CERTIFIER_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());

    let is_live = google_config.use_live_services;

    let eval_request = serde_json::json!({
        "tenant_id": tenant_id.to_string(),
        "workflow_id": workflow.workflow_id.to_string(),
        "candidate_revision_id": candidate.revision_id.to_string(),
        "candidate_identity": candidate.abom.agent_identity,
        "case_ids": required_cases,
        "policy_pack_id": workflow.policy_pack_id,
        "corpus_version": workflow.corpus_version,
        "trace_id": trace_id,
        "is_live": is_live,
    });

    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(1_800))
        .build()
        .map_err(|e| format!("Failed to build HTTP client: {e}"))?;

    let mut eval_builder = http_client
        .post(format!("{adk_url}/internal/v1/evaluations"))
        .json(&eval_request);
    if adk_url.starts_with("https://") {
        eval_builder = eval_builder.bearer_auth(cloud_run_identity_token(&adk_url).await?);
    }
    let eval_resp = eval_builder.send().await.map_err(|e| {
        format!("ADK certifier unreachable at {adk_url}: {e}. Certification blocked.")
    })?;

    if !eval_resp.status().is_success() {
        let status = eval_resp.status();
        let body = eval_resp.text().await.unwrap_or_default();
        return Err(format!(
            "ADK certifier returned {status}: {body}. Certification blocked."
        ));
    }

    let eval_result: serde_json::Value = eval_resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse ADK certifier response: {e}"))?;

    info!(
        "ADK certifier completed evaluation {} with status {:?}",
        eval_result["evaluation_id"], eval_result["status"]
    );

    // ── Convert ADK observations to domain CaseEvidence ───────────────────────
    let observations = eval_result["observations"]
        .as_array()
        .ok_or_else(|| "ADK certifier response missing 'observations' array".to_string())?;

    let mut cases = Vec::new();
    let mut model_armor_events = Vec::new();
    let mut gateway_decisions = Vec::new();

    for obs in observations {
        let case_id_str = obs["case_id"].as_str().unwrap_or("unknown");
        let provenance_str = obs["provenance"].as_str().unwrap_or("LOCAL");
        let provenance = match provenance_str {
            "LIVE" => Provenance::Live,
            "REPLAY" => Provenance::Replay,
            "SYSTEM_TEST" => Provenance::SystemTest,
            "INFERRED" => Provenance::Inferred,
            _ => Provenance::Local,
        };

        let evidence = CaseEvidence {
            schema_version: "sentinel.case.v1".to_string(),
            tenant_id,
            workflow_id: workflow.workflow_id,
            case_run_id: CaseRunId::new(),
            case_id: CaseId::new(case_id_str),
            corpus_version: obs["corpus_version"]
                .as_str()
                .unwrap_or(&workflow.corpus_version)
                .to_string(),
            attempt: obs["attempt"].as_u64().unwrap_or(1) as u32,
            expected_outcome: obs["expected_outcome"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            observed_outcome: obs["observed_outcome"]
                .as_str()
                .unwrap_or("UNKNOWN")
                .to_string(),
            model_armor_disposition: obs["model_armor_disposition"].as_str().map(str::to_string),
            candidate_identity: sentinel_domain::ids::PrincipalId::new(
                obs["candidate_identity"]
                    .as_str()
                    .unwrap_or(candidate.abom.agent_identity.0.as_str()),
            ),
            requested_tool: obs["requested_tool"].as_str().map(str::to_string),
            gateway_decision: obs["gateway_decision"].as_str().map(str::to_string),
            ledger_snapshot_before_digest: obs["ledger_snapshot_before_digest"]
                .as_str()
                .unwrap_or("sha256:unknown")
                .to_string(),
            ledger_snapshot_after_digest: obs["ledger_snapshot_after_digest"]
                .as_str()
                .unwrap_or("sha256:unknown")
                .to_string(),
            latency_ms: obs["latency_ms"].as_u64().unwrap_or(0),
            token_count: obs["token_count"].as_u64().unwrap_or(0),
            cost_usd_micro: obs["cost_usd_micro"].as_u64().unwrap_or(0) as u64,
            trace_id: obs["trace_id"].as_str().map(str::to_string),
            provenance,
            timestamp: now,
        };

        // Collect Model Armor events from observations that were blocked
        if obs["model_armor_disposition"].as_str() == Some("BLOCK") {
            if let Ok(armor_ev) = model_armor::inspect_content(
                tenant_id,
                &evidence.case_id,
                evidence.expected_outcome.as_str(),
                &google_config.model_armor_template,
                false, // skip re-calling if already done by ADK certifier
            )
            .await
            {
                model_armor_events.push(armor_ev);
            }
        }

        // Collect Gateway decisions from observations that were denied
        if obs["gateway_decision"].as_str() == Some("DENY") {
            if let Some(tool) = &evidence.requested_tool {
                if let Ok(gw_ev) = gateway::check_permission(
                    tenant_id,
                    &candidate.abom.agent_identity,
                    tool.as_str(),
                    false, // already checked by ADK certifier
                )
                .await
                {
                    gateway_decisions.push(gw_ev);
                }
            }
        }

        cases.push(evidence);
    }

    // 2. Assemble Evidence Bundle
    let bundle = EvidenceBundle {
        candidate: candidate.clone(),
        cases,
        model_armor_events,
        gateway_decisions,
        approval: None,
        retest_results: Vec::new(),
        trace_id: Some(trace_id.to_string()),
    };

    let ledger_before: LedgerSnapshot =
        serde_json::from_value(eval_result["ledger_snapshot_before"].clone()).map_err(|e| {
            format!("ADK certifier response omitted a valid before-ledger snapshot: {e}")
        })?;
    let ledger_after: LedgerSnapshot =
        serde_json::from_value(eval_result["ledger_snapshot_after"].clone()).map_err(|e| {
            format!("ADK certifier response omitted a valid after-ledger snapshot: {e}")
        })?;
    let mut ledger_invariants = LedgerInvariants::evaluate(&ledger_before, &ledger_after);
    ledger_invariants.expected_drafts = bundle
        .cases
        .iter()
        .filter(|case| {
            matches!(
                case.observed_outcome.as_str(),
                "DRAFT_CREATED" | "SAFE_TASK_COMPLETED"
            )
        })
        .count() as u64;

    // 3. Build & Store Content-Addressed Manifest
    let mut manifest_builder = EvidenceManifestBuilder::new(tenant_id, workflow.workflow_id);
    let evidence_provenance = if is_live {
        Provenance::Live
    } else {
        Provenance::Local
    };
    manifest_builder
        .add_object(
            "candidate.json",
            "application/json",
            "sentinel.candidate.v1",
            "control-plane",
            evidence_provenance,
            &candidate,
            now,
        )
        .map_err(|e| e.to_string())?;
    manifest_builder
        .add_object(
            "cases.json",
            "application/json",
            "sentinel.cases.v1",
            "adk-certifier",
            evidence_provenance,
            &bundle.cases,
            now,
        )
        .map_err(|e| e.to_string())?;
    manifest_builder
        .add_object(
            "ledger-before.json",
            "application/json",
            "sentinel.ledger.snapshot.v1",
            "enterprise-erp-oracle",
            evidence_provenance,
            &ledger_before,
            now,
        )
        .map_err(|e| e.to_string())?;
    manifest_builder
        .add_object(
            "ledger-after.json",
            "application/json",
            "sentinel.ledger.snapshot.v1",
            "enterprise-erp-oracle",
            evidence_provenance,
            &ledger_after,
            now,
        )
        .map_err(|e| e.to_string())?;
    let manifest = manifest_builder.build(now);
    EvidenceRepository::store_manifest(&**store, &manifest).await?;

    // 4. Run Policy Evaluation
    let pack = load_policy_pack().await?;
    let evaluation = evaluate(&pack, &bundle, &ledger_invariants, now);

    // Record policy decision metric
    POLICY_DECISIONS
        .with_label_values(&[format!("{:?}", evaluation.gate_decision).as_str()])
        .inc();

    // 5. Store Findings
    for r in &evaluation.rule_results {
        if !r.passed {
            let finding = Finding {
                finding_id: Uuid::new_v4(),
                tenant_id,
                workflow_id: workflow.workflow_id,
                case_run_id: None,
                case_id: None,
                rule_id: Some(r.rule_ref.clone()),
                kind: FindingKind::ApprovalRequired,
                severity: FindingSeverity::High,
                title: format!("Policy Rule Notice: {}", r.rule_ref.0),
                description: r.explanation.clone(),
                evidence_refs: r.evidence_refs.clone(),
                remediation: Some(Remediation {
                    action: RemediationAction::ReduceCapabilities,
                    detail:
                        "Reviewer approval required to reduce permissions to draft_invoice_payment"
                            .to_string(),
                    requires_approval: true,
                }),
                provenance: evidence_provenance,
                created_at: now,
            };
            FindingRepository::store(&**store, &finding).await?;
        }
    }

    // 6. Transition Workflow State
    let expected_version = workflow.version;
    workflow
        .apply(WorkflowCommand::EvidenceCollected, expected_version, now)
        .map_err(|e| e.to_string())?;
    WorkflowRepository::update(&**store, workflow, expected_version).await?;

    let expected_version = workflow.version;
    let from_state = workflow.state;
    workflow
        .apply(
            WorkflowCommand::EvaluationFinished {
                decision: evaluation.gate_decision,
                explanation: evaluation.explanation.clone(),
            },
            expected_version,
            now,
        )
        .map_err(|e| e.to_string())?;
    WorkflowRepository::update(&**store, workflow, expected_version).await?;

    // Record state transition metric
    WORKFLOW_STATE_TRANSITIONS
        .with_label_values(&[
            format!("{:?}", from_state).as_str(),
            format!("{:?}", workflow.state).as_str(),
            worker_id,
        ])
        .inc();

    info!(
        "Workflow {} advanced to state {:?}",
        workflow.workflow_id, workflow.state
    );
    Ok(())
}

async fn resolve_required_cases(configured: &[String]) -> Result<Vec<String>, String> {
    if configured != ["__ALL_CORPUS_CASES__"] {
        return Ok(configured.to_vec());
    }

    let manifest_path = std::env::var("SENTINEL_CORPUS_MANIFEST")
        .unwrap_or_else(|_| "/app/corpus/v1/manifest.json".to_string());
    let bytes = tokio::fs::read(&manifest_path)
        .await
        .map_err(|e| format!("Failed to read corpus manifest {manifest_path}: {e}"))?;
    let manifest: serde_json::Value = serde_json::from_slice(&bytes)
        .map_err(|e| format!("Failed to parse corpus manifest {manifest_path}: {e}"))?;
    let cases = manifest["cases"]
        .as_array()
        .ok_or_else(|| "Corpus manifest is missing cases array".to_string())?
        .iter()
        .map(|case| {
            case["case_id"]
                .as_str()
                .map(str::to_string)
                .ok_or_else(|| "Corpus manifest case is missing case_id".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;

    if cases.len() != 80 {
        return Err(format!("Expected 80 corpus cases, found {}", cases.len()));
    }
    Ok(cases)
}

async fn load_policy_pack() -> Result<sentinel_domain::policy::PolicyPack, String> {
    let path = std::env::var("SENTINEL_POLICY_PACK")
        .unwrap_or_else(|_| "/app/policy-packs/ap-agent-v1/pack.json".to_string());
    let bytes = tokio::fs::read(&path)
        .await
        .map_err(|e| format!("Failed to read policy pack {path}: {e}"))?;
    serde_json::from_slice(&bytes).map_err(|e| format!("Failed to parse policy pack {path}: {e}"))
}

async fn cloud_run_identity_token(audience: &str) -> Result<String, String> {
    let response = reqwest::Client::new()
        .get("http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/identity")
        .query(&[("audience", audience), ("format", "full")])
        .header("Metadata-Flavor", "Google")
        .timeout(Duration::from_secs(3))
        .send()
        .await
        .map_err(|e| format!("Cloud Run identity-token request failed: {e}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "Cloud Run identity-token request returned {}",
            response.status()
        ));
    }
    response
        .text()
        .await
        .map_err(|e| format!("Cloud Run identity-token body failed: {e}"))
}

async fn execute_workflow_retest(
    store: &Arc<Store>,
    workflow: &mut sentinel_domain::workflow::Workflow,
    google_config: &sentinel_config::GoogleCloudConfig,
    cert_config: &sentinel_config::CertificationConfig,
    worker_id: &str,
) -> Result<(), String> {
    // W3C traceparent trace-id (32 hex). A fresh ID per retest keeps Cloud Trace
    // spans distinct instead of reusing a static example string. The span carries
    // `trace_id` and `workflow_id` so each retest's Cloud Trace span correlates
    // back to exactly one run (the same trace_id is sent to the ADK certifier).
    let trace_id = Uuid::new_v4().as_simple().to_string();
    // Capture workflow_id as an owned value for the span so the span does not
    // borrow `workflow` (`workflow` is passed mutably into the inner call).
    let workflow_id = workflow.workflow_id.to_string();
    let span = tracing::info_span!(
        "retest",
        trace_id = %trace_id,
        workflow_id = %workflow_id,
    );
    execute_workflow_retest_inner(
        store,
        workflow,
        google_config,
        cert_config,
        worker_id,
        &trace_id,
    )
    .instrument(span)
    .await
}

async fn execute_workflow_retest_inner(
    store: &Arc<Store>,
    workflow: &mut sentinel_domain::workflow::Workflow,
    google_config: &sentinel_config::GoogleCloudConfig,
    cert_config: &sentinel_config::CertificationConfig,
    worker_id: &str,
    trace_id: &str,
) -> Result<(), String> {
    let now = OffsetDateTime::now_utc();
    let tenant_id = workflow.tenant_id;
    let candidate = CandidateRepository::get(&**store, tenant_id, &workflow.candidate_revision_id)
        .await?
        .ok_or_else(|| "Candidate not found".to_string())?;
    let approval =
        ApprovalRepository::get_for_workflow(&**store, tenant_id, workflow.workflow_id).await?;

    let adk_url = std::env::var("SENTINEL_ADK_CERTIFIER_URL")
        .unwrap_or_else(|_| "http://localhost:8081".to_string());
    let retest_request = serde_json::json!({
        "tenant_id": tenant_id.to_string(),
        "workflow_id": workflow.workflow_id.to_string(),
        "candidate_revision_id": candidate.revision_id.to_string(),
        "candidate_identity": candidate.abom.agent_identity,
        "case_ids": [cert_config.retest_draft_case.clone(), cert_config.retest_release_case.clone()],
        "policy_pack_id": workflow.policy_pack_id,
        "corpus_version": workflow.corpus_version,
        "trace_id": trace_id,
        "is_live": google_config.use_live_services,
    });
    let mut request = reqwest::Client::builder()
        .timeout(Duration::from_secs(600))
        .build()
        .map_err(|e| format!("Failed to build retest HTTP client: {e}"))?
        .post(format!("{adk_url}/internal/v1/evaluations"))
        .json(&retest_request);
    if adk_url.starts_with("https://") {
        request = request.bearer_auth(cloud_run_identity_token(&adk_url).await?);
    }
    let response = request
        .send()
        .await
        .map_err(|e| format!("Post-approval retest request failed: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Post-approval retest returned {status}: {body}"));
    }
    let result: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("Post-approval retest response was invalid: {e}"))?;
    let observations = result["observations"]
        .as_array()
        .ok_or_else(|| "Post-approval retest response omitted observations".to_string())?;
    let observed = |case_id: &str| {
        observations
            .iter()
            .find(|item| item["case_id"].as_str() == Some(case_id))
            .and_then(|item| item["observed_outcome"].as_str())
    };

    let retest_draft = RetestResult {
        case_id: cert_config.retest_draft_case.clone(),
        passed: matches!(
            observed(&cert_config.retest_draft_case),
            Some("DRAFT_CREATED" | "SAFE_TASK_COMPLETED")
        ),
        action_tested: "draft_invoice_payment".to_string(),
    };
    let retest_release = RetestResult {
        case_id: cert_config.retest_release_case.clone(),
        passed: matches!(
            observed(&cert_config.retest_release_case),
            Some("PREVENTED_AT_GATEWAY" | "PREVENTED_AT_ARMOR")
        ),
        action_tested: "release_payment".to_string(),
    };
    let retests_passed = retest_draft.passed && retest_release.passed;

    let expected_version = workflow.version;
    let from_state = workflow.state;
    workflow
        .apply(
            WorkflowCommand::RetestFinished {
                passed: retests_passed,
            },
            expected_version,
            now,
        )
        .map_err(|e| e.to_string())?;
    WorkflowRepository::update(&**store, workflow, expected_version).await?;

    if !retests_passed {
        return Err(format!(
            "Post-approval retest failed: draft_passed={}, release_denied={}",
            retest_draft.passed, retest_release.passed
        ));
    }

    // Record state transition metric
    WORKFLOW_STATE_TRANSITIONS
        .with_label_values(&[format!("{:?}", from_state).as_str(), "Attesting", worker_id])
        .inc();

    // Now in Attesting state: construct Attestation Payload and sign with KMS
    let manifest = EvidenceRepository::get_manifest(&**store, tenant_id, workflow.workflow_id)
        .await?
        .ok_or_else(|| "Manifest missing".to_string())?;

    let payload = build_attestation_payload(
        tenant_id,
        "production",
        workflow.workflow_id,
        &candidate,
        workflow.policy_pack_id.clone(),
        "1.0.0",
        "sha256:pack-ap-v1-digest",
        workflow.corpus_version.clone(),
        "sha256:corpus-v1-digest",
        "sha256:eval-all-passed",
        manifest.manifest_digest.clone(),
        vec!["draft_invoice_payment".to_string()],
        approval.as_ref(),
        GateDecision::Certifiable,
        now,
        time::Duration::days(90),
        "sentinel-authority@project.iam.gserviceaccount.com",
    );

    let key_ref = KeyReference {
        key_resource: google_config.kms_key_resource.clone(),
        key_version: google_config.kms_key_version.clone(),
        algorithm: "RSA_SIGN_PSS_2048_SHA256".to_string(),
    };

    let signed_attestation = sign_attestation(&payload, &key_ref, None).await?;
    let payload_digest =
        sha256_digest(&sentinel_evidence::canonicalize(&payload).map_err(|e| e.to_string())?);

    // Record attestation operation metric
    ATTESTATION_OPERATIONS
        .with_label_values(&["sign", "success"])
        .inc();

    AttestationRepository::store(&**store, workflow.workflow_id, &signed_attestation).await?;

    // Advance to Certified
    let expected_version = workflow.version;
    let from_state = workflow.state;
    workflow
        .apply(
            WorkflowCommand::AttestationSigned {
                attestation_digest: payload_digest,
                signature: signed_attestation.signature.clone(),
                key_version: "1".to_string(),
            },
            expected_version,
            now,
        )
        .map_err(|e| e.to_string())?;
    WorkflowRepository::update(&**store, workflow, expected_version).await?;

    // Record state transition metric
    WORKFLOW_STATE_TRANSITIONS
        .with_label_values(&[format!("{:?}", from_state).as_str(), "Certified", worker_id])
        .inc();

    // Record attestation operation metric
    ATTESTATION_OPERATIONS
        .with_label_values(&["store", "success"])
        .inc();

    // Record policy decision metric
    POLICY_DECISIONS.with_label_values(&["Certifiable"]).inc();

    // Record audit event
    let event = AuditEvent {
        event_id: Uuid::new_v4(),
        tenant_id,
        workflow_id: Some(workflow.workflow_id),
        case_run_id: None,
        event_type: AuditEventType::AttestationSigned,
        actor: sentinel_domain::ids::PrincipalId::new("sentinel-kms-signer"),
        before_state: Some(WorkflowState::Attesting),
        after_state: Some(WorkflowState::Certified),
        command: None,
        decision: Some(GateDecision::Certifiable),
        explanation: Some(
            "Candidate revision certified with valid KMS signed attestation".to_string(),
        ),
        provenance: Provenance::Live,
        timestamp: now,
        trace_id: None,
        span_id: None,
    };
    AuditRepository::append(&**store, &event).await?;

    info!("Workflow {} successfully CERTIFIED!", workflow.workflow_id);
    Ok(())
}
