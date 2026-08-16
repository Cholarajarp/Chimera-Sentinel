//! Append-only audit events for every state change.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{CaseRunId, PrincipalId, TenantId, WorkflowId};
use crate::provenance::Provenance;
use crate::workflow::{WorkflowCommand, WorkflowState};

/// Audit event for an authoritative record of every decision-relevant action.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub workflow_id: Option<WorkflowId>,
    pub case_run_id: Option<CaseRunId>,
    pub event_type: AuditEventType,
    pub actor: PrincipalId,
    pub before_state: Option<WorkflowState>,
    pub after_state: Option<WorkflowState>,
    pub command: Option<WorkflowCommand>,
    pub decision: Option<crate::workflow::GateDecision>,
    pub explanation: Option<String>,
    pub provenance: Provenance,
    pub timestamp: OffsetDateTime,
    /// Correlation IDs for tracing (W3C traceparent, etc.)
    pub trace_id: Option<String>,
    pub span_id: Option<String>,
}

/// Exhaustive list of auditable event types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AuditEventType {
    CandidateRegistered,
    CertificationStarted,
    WorkflowStarted,
    WorkerClaimed,
    CasesDispatched,
    EvidenceReceived,
    EvaluationCompleted,
    ApprovalRecorded,
    ApprovalRejected,
    RetestCompleted,
    AttestationRequested,
    AttestationSigned,
    AttestationVerified,
    WorkflowFailed,
    WorkflowCancelled,
    AttestationRevoked,
    ConfigurationDriftDetected,
}
