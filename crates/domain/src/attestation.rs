//! Attestation payload, envelope, and verification types.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{ApprovalId, PolicyPackId, PrincipalId, RevisionId, TenantId, WorkflowId};

/// Immutable reference to a Cloud KMS key version used for signing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyReference {
    pub key_resource: String, // e.g., projects/p/locations/l/keyRings/kr/cryptoKeys/k
    pub key_version: String,  // e.g., 1
    pub algorithm: String,    // e.g., RSA_SIGN_PSS_2048_SHA256
}

/// Canonical attestation payload. This is what gets signed by Cloud KMS.
/// Every field is bound to the exact certification execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationPayload {
    pub schema_version: String, // e.g., sentinel.attestation.v1
    pub tenant_id: TenantId,
    pub environment: String,
    pub workflow_id: WorkflowId,
    pub candidate_revision_id: RevisionId,
    pub agent_registry_resource: String,
    pub agent_runtime_resource: String,
    pub agent_identity: PrincipalId,
    pub source_digest: String,
    pub prompt_config_digest: String,
    pub model_ref: String,
    pub tool_manifest_digest: String,
    pub memory_config_digest: String,
    pub gateway_policy_digest: String,
    pub model_armor_config_digest: String,
    pub policy_pack_id: PolicyPackId,
    pub policy_pack_version: String,
    pub policy_pack_digest: String,
    pub corpus_version: String,
    pub corpus_digest: String,
    pub evaluation_digest: String,
    pub evidence_manifest_digest: String,
    pub constrained_capabilities: Vec<String>,
    pub approval_ids: Vec<ApprovalId>,
    pub approval_expiry: Option<OffsetDateTime>,
    pub decision: crate::workflow::GateDecision,
    pub issued_at: OffsetDateTime,
    pub not_before: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub issuer: String, // e.g., chimera-sentinel@project.iam.gserviceaccount.com
    pub nonce: String,  // cryptographically random
}

/// Full attestation envelope returned to callers and stored.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationMetadata {
    pub payload: AttestationPayload,
    pub signature: String, // base64-encoded signature
    pub key_reference: KeyReference,
    pub canonicalization_algorithm: String, // e.g., RFC8785
    pub status: AttestationStatus,
}

/// Current status of an attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AttestationStatus {
    Active,
    Expired,
    Revoked,
    Superseded,
}

/// Verification result for offline/cli verification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationResult {
    pub valid: bool,
    pub payload: Option<AttestationPayload>,
    pub error: Option<String>,
    pub checks: VerificationChecks,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VerificationChecks {
    pub signature_valid: bool,
    pub key_matches: bool,
    pub not_expired: bool,
    pub not_revoked: bool,
    pub audience_matches: bool,
    pub environment_matches: bool,
    pub revision_matches: bool,
    pub digest_matches: bool,
}
