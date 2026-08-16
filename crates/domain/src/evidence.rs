//! Evidence manifests, evidence objects, and provider observations.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::ids::{ApprovalId, CaseId, CaseRunId, PrincipalId, TenantId, WorkflowId};
use crate::provenance::Provenance;

/// Content-addressed manifest listing all evidence objects for a workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceManifestRef {
    pub schema_version: String,
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub objects: Vec<EvidenceObjectRef>,
    pub manifest_digest: String,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
}

impl EvidenceManifestRef {
    pub fn compute_digest(objects: &[EvidenceObjectRef]) -> String {
        let mut hasher = Sha256::new();
        for obj in objects {
            hasher.update(obj.path.as_bytes());
            hasher.update(obj.sha256_digest.as_bytes());
            hasher.update(obj.size_bytes.to_le_bytes());
        }
        format!("sha256:{}", hex::encode(hasher.finalize()))
    }

    pub fn verify_objects(
        &self,
        fetch: impl Fn(&str) -> Result<Vec<u8>, String>,
    ) -> Result<(), String> {
        for obj in &self.objects {
            let bytes = fetch(&obj.path)?;
            let mut hasher = Sha256::new();
            hasher.update(&bytes);
            let computed = format!("sha256:{}", hex::encode(hasher.finalize()));
            if computed != obj.sha256_digest {
                return Err(format!(
                    "Digest mismatch for {}: expected {}, computed {}",
                    obj.path, obj.sha256_digest, computed
                ));
            }
        }
        Ok(())
    }
}

/// Metadata entry for a single immutable object in Cloud Storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceObjectRef {
    pub path: String,
    pub media_type: String,
    pub schema_version: String,
    pub sha256_digest: String,
    pub size_bytes: u64,
    pub producer: String,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Execution evidence from a single test case run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaseEvidence {
    pub schema_version: String,
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub case_run_id: CaseRunId,
    pub case_id: CaseId,
    pub corpus_version: String,
    pub attempt: u32,
    pub expected_outcome: String,
    pub observed_outcome: String,
    pub model_armor_disposition: Option<String>,
    pub candidate_identity: PrincipalId,
    pub requested_tool: Option<String>,
    pub gateway_decision: Option<String>,
    pub ledger_snapshot_before_digest: String,
    pub ledger_snapshot_after_digest: String,
    pub latency_ms: u64,
    pub token_count: u64,
    pub cost_usd_micro: u64,
    pub trace_id: Option<String>,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Evidence of Model Armor inspection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelArmorEvidence {
    pub tenant_id: TenantId,
    pub case_id: CaseId,
    pub template_id: String,
    pub disposition: String, // "BLOCK", "ALLOW"
    pub matched_rules: Vec<String>,
    pub sanitized_explanation: String,
    pub provider_reference: String,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Evidence of Agent Gateway permission check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayDecisionEvidence {
    pub tenant_id: TenantId,
    pub principal: PrincipalId,
    pub action: String,
    pub allowed: bool,
    pub policy_reference: String,
    pub provider_reference: String,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Evidence of advisory Memory Bank retrieval.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryRetrievalEvidence {
    pub tenant_id: TenantId,
    pub memory_bank_id: String,
    pub retrieved_items: Vec<String>,
    pub analyst_approval_references: Vec<String>,
    pub provider_reference: String,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Trace correlation reference.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TraceReferenceEvidence {
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub trace_id: String,
    pub root_span_id: String,
    pub cloud_trace_url: String,
    pub provenance: Provenance,
    #[serde(with = "time::serde::rfc3339")]
    pub timestamp: OffsetDateTime,
}

/// Scoped human approval record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    pub approval_id: ApprovalId,
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub reviewer: PrincipalId,
    pub reviewer_role: String,
    pub proposed_capabilities: Vec<String>,
    pub approved_capabilities: Vec<String>,
    pub reason_code: String,
    pub note: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub issued_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub expires_at: OffsetDateTime,
    pub policy_version: String,
}
