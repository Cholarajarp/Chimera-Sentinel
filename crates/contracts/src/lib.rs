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
    ids::{
        AgentId, ApprovalId, IdempotencyKey, PolicyPackId, PrincipalId, RevisionId, TenantId,
        WorkflowId,
    },
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
        #[serde(default)]
        pub vulnerabilities: Vec<sentinel_domain::candidate::Vulnerability>,
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
        #[serde(default)]
        pub vulnerabilities: Vec<sentinel_domain::candidate::Vulnerability>,
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
                vulnerabilities: c.vulnerabilities,
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

/// Versioned evaluation-corpus contracts.
///
/// These types describe case *definitions* only: static, content-addressed
/// artifacts checked into the repository. They never carry observed run
/// results, which are evidence and are served from workflow findings instead.
pub mod corpus {
    use super::*;

    /// Assertions a case makes about the candidate's required behaviour.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CaseExpectations {
        pub expected_outcome: String,
        pub expected_model_armor_disposition: String,
        pub expected_gateway_disposition: String,
        pub allowed_tools: Vec<String>,
        pub forbidden_tools: Vec<String>,
        pub requested_tool: Option<String>,
        pub unauthorized_released_payment_delta: i64,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CorpusCase {
        pub case_id: String,
        pub category: String,
        pub category_name: String,
        pub split: String,
        pub title: String,
        pub description: String,
        /// Sealed holdout payloads are withheld so evaluation stays unbiased.
        pub sealed: bool,
        /// Attack payload as authored. `None` when `sealed` is true. Untyped
        /// because the corpus intentionally contains malformed fixtures.
        pub fixture: Option<serde_json::Value>,
        pub expectations: CaseExpectations,
        pub source_path: String,
        /// Digest recorded in the corpus manifest.
        pub sha256: String,
        /// True when the file's bytes re-hash to `sha256`.
        pub digest_verified: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CorpusCategorySummary {
        pub id: String,
        pub name: String,
        pub dev_count: u32,
        pub holdout_count: u32,
        pub total: u32,
    }

    /// Result of re-hashing every case file against the manifest at load time.
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CorpusIntegrity {
        pub manifest_sha256: String,
        pub declared_cases: usize,
        pub loaded_cases: usize,
        pub digest_verified_cases: usize,
        pub all_digests_verified: bool,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct CorpusResponse {
        pub schema_version: String,
        pub corpus_version: String,
        pub name: String,
        pub description: String,
        pub total_cases: usize,
        pub development_cases: usize,
        pub holdout_cases: usize,
        pub categories: Vec<CorpusCategorySummary>,
        pub integrity: CorpusIntegrity,
        pub cases: Vec<CorpusCase>,
        /// Always `CORPUS_DEFINITION`; distinguishes definitions from evidence.
        pub provenance: String,
    }
}

pub mod audit {
    use sentinel_domain::AuditEvent;

    use super::*;

    /// Append-only workflow audit events as recorded by the trusted core.
    #[derive(Debug, Clone, Serialize, Deserialize, Default)]
    pub struct AuditListResponse {
        pub events: Vec<AuditEvent>,
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
