//! Versioned transport and storage contracts.
//!
//! This crate holds the public and internal API contracts (request/response types,
//! serialization models) shared between the control plane, worker, certifier,
//! CLI, and web console.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use sentinel_domain::{
    attestation::AttestationMetadata,
    candidate::{AgentBillOfMaterials, CandidateRevision},
    evidence::{EvidenceManifestRef, EvidenceObjectRef},
    finding::Finding,
    ids::{AgentId, ApprovalId, IdempotencyKey, PolicyPackId, PrincipalId, RevisionId, TenantId, WorkflowId},
    workflow::{GateDecision, WorkflowState, WorkflowSummary},
};

/// RFC 9457 compliant error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub r#type: String,
    pub title: String,
    pub status: u16,
    pub detail: String,
    pub instance: Option<String>,
    pub request_id: Uuid,
    pub timestamp: OffsetDateTime,
}

impl ErrorResponse {
    pub fn new(status: u16, title: impl Into<String>, detail: impl Into<String>) -> Self {
        Self {
            r#type: format!("https://sentinel.enterprise/errors/{}", status),
            title: title.into(),
            status,
            detail: detail.into(),
            instance: None,
            request_id: Uuid::new_v4(),
            timestamp: OffsetDateTime::now_utc(),
        }
    }
}

pub mod candidate {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CandidateRevisionRequest {
        pub tenant_id: TenantId,
        pub agent_id: AgentId,
        pub abom: AgentBillOfMaterials,
        pub policy_pack_id: PolicyPackId,
        pub corpus_version: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CandidateRevisionResponse {
        pub revision_id: RevisionId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CandidateDetailResponse {
        pub tenant_id: TenantId,
        pub agent_id: AgentId,
        pub revision_id: RevisionId,
        pub abom: AgentBillOfMaterials,
        pub policy_pack_id: PolicyPackId,
        pub corpus_version: String,
        #[serde(with = "time::serde::rfc3339")]
        pub created_at: OffsetDateTime,
    }

    impl From<CandidateRevision> for CandidateDetailResponse {
        fn from(c: CandidateRevision) -> Self {
            Self {
                tenant_id: c.tenant_id,
                agent_id: c.agent_id,
                revision_id: c.revision_id,
                abom: c.abom,
                policy_pack_id: c.policy_pack_id,
                corpus_version: c.corpus_version,
                created_at: OffsetDateTime::now_utc(),
            }
        }
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CandidateListResponse {
        pub candidates: Vec<CandidateDetailResponse>,
        pub next_cursor: Option<String>,
    }
}

pub mod workflow {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateCertificationRequest {
        pub tenant_id: TenantId,
        pub candidate_revision_id: RevisionId,
        pub policy_pack_id: PolicyPackId,
        pub corpus_version: String,
        pub idempotency_key: IdempotencyKey,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CreateCertificationResponse {
        pub workflow_id: WorkflowId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowResponse {
        pub workflow_id: WorkflowId,
        pub tenant_id: TenantId,
        pub candidate_revision_id: RevisionId,
        pub state: WorkflowState,
        pub version: u64,
        pub policy_pack_id: PolicyPackId,
        pub corpus_version: String,
        pub gate_decision: Option<GateDecision>,
        pub gate_explanation: Option<String>,
        pub approval_id: Option<ApprovalId>,
        pub attestation_digest: Option<String>,
        #[serde(with = "time::serde::rfc3339")]
        pub created_at: OffsetDateTime,
        #[serde(with = "time::serde::rfc3339")]
        pub updated_at: OffsetDateTime,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowListResponse {
        pub workflows: Vec<WorkflowSummary>,
        pub next_cursor: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct WorkflowEventResponse {
        pub event_id: Uuid,
        pub workflow_id: WorkflowId,
        pub event_type: String,
        pub state: WorkflowState,
        pub message: String,
        #[serde(with = "time::serde::rfc3339")]
        pub timestamp: OffsetDateTime,
    }
}

pub mod finding {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FindingsListResponse {
        pub findings: Vec<Finding>,
    }
}

pub mod approval {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubmitApprovalRequest {
        pub tenant_id: TenantId,
        pub reviewer: PrincipalId,
        pub reviewer_role: String,
        pub proposed_capabilities: Vec<String>,
        pub approved_capabilities: Vec<String>,
        pub reason_code: String,
        pub note: Option<String>,
        pub duration_seconds: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SubmitApprovalResponse {
        pub approval_id: ApprovalId,
    }
}

pub mod evidence {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EvidenceManifestRequest {
        pub tenant_id: TenantId,
        pub workflow_id: WorkflowId,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct EvidenceManifestResponse {
        pub manifest: Option<EvidenceManifestRef>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EvidenceObjectResponse {
        pub object: EvidenceObjectRef,
        pub content: Option<serde_json::Value>,
    }
}

pub mod attestation {
    use super::*;

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AttestationVerifyRequest {
        pub tenant_id: TenantId,
        pub attestation: AttestationMetadata,
        pub environment: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct AttestationVerifyResponse {
        pub valid: bool,
        pub details: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct AttestationResponse {
        pub attestation: Option<AttestationMetadata>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct WorkflowResponse {
        pub workflow_id: Option<WorkflowId>,
        pub tenant_id: Option<TenantId>,
        pub candidate_revision_id: Option<RevisionId>,
        pub state: Option<WorkflowState>,
        pub version: Option<u64>,
        pub policy_pack_id: Option<PolicyPackId>,
        pub corpus_version: Option<String>,
        pub gate_decision: Option<GateDecision>,
        pub gate_explanation: Option<String>,
        pub approval_id: Option<ApprovalId>,
        pub attestation_digest: Option<String>,
    }

    impl From<sentinel_domain::workflow::Workflow> for WorkflowResponse {
        fn from(w: sentinel_domain::workflow::Workflow) -> Self {
            Self {
                workflow_id: Some(w.workflow_id),
                tenant_id: Some(w.tenant_id),
                candidate_revision_id: Some(w.candidate_revision_id),
                state: Some(w.state),
                version: Some(w.version),
                policy_pack_id: Some(w.policy_pack_id),
                corpus_version: Some(w.corpus_version),
                gate_decision: w.gate_decision,
                gate_explanation: w.gate_explanation,
                approval_id: w.approval_id,
                attestation_digest: w.attestation_digest,
            }
        }
    }
}
