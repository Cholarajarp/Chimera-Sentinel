//! Candidate agent revisions and Agent Bill of Materials (ABOM).
//!
//! Every candidate revision represents an immutable snapshot of an agent's
//! complete security configuration, code, prompts, tools, identities, and policies.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use time::OffsetDateTime;

use crate::ids::{AgentId, PolicyPackId, PrincipalId, RevisionId, TenantId};
use crate::provenance::Provenance;

/// Cryptographic digest representing immutable content.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct RevisionDigest(pub String);

impl RevisionDigest {
    pub fn sha256(bytes: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(bytes);
        Self(format!("sha256:{}", hex::encode(hasher.finalize())))
    }

    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Agent Bill of Materials (ABOM).
///
/// Binds every security-critical attribute of an agent revision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentBillOfMaterials {
    /// SHA-256 digest of agent source code/artifacts.
    pub source_digest: RevisionDigest,
    /// Resource name in Google Agent Registry (e.g. projects/p/locations/l/agents/a).
    pub registry_resource: String,
    /// Resource name for Agent Runtime execution.
    pub runtime_resource: String,
    /// Dedicated service identity principal for this candidate.
    pub agent_identity: PrincipalId,
    /// Explicit eligible Gemini model reference (e.g. vertex-ai:gemini-3.5-flash@2026-08-01).
    pub model_ref: String,
    /// SHA-256 digest of system prompts and `temperature/top_p` configs.
    pub prompt_config_digest: RevisionDigest,
    /// SHA-256 digest of the declared tool manifest and MCP server definitions.
    pub tool_manifest_digest: RevisionDigest,
    /// SHA-256 digest of Memory Bank configuration and knowledge sources.
    pub memory_config_digest: RevisionDigest,
    /// SHA-256 digest of Agent Gateway policy configuration.
    pub gateway_policy_digest: RevisionDigest,
    /// SHA-256 digest of Model Armor filter template and configuration.
    pub model_armor_config_digest: RevisionDigest,
    /// Declared list of requested capabilities (e.g. `["draft_invoice_payment", "release_payment"]`).
    pub requested_capabilities: Vec<String>,
    /// Data classification labels handled by this agent.
    pub data_classification: Vec<String>,
    /// Target environment (e.g. "production", "staging", "test").
    pub environment: String,
    /// Attributable owner principal.
    pub owner: PrincipalId,
    /// Risk tier of this workload.
    pub risk_tier: RiskTier,
    /// Provenance of the ABOM record.
    pub provenance: Provenance,
    /// Timestamp when this ABOM was recorded.
    #[serde(with = "time::serde::rfc3339")]
    pub recorded_at: OffsetDateTime,
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RiskTier {
    Low,
    #[default]
    Medium,
    High,
    Critical,
}

/// An immutable candidate revision ready for admission testing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateRevision {
    pub tenant_id: TenantId,
    pub agent_id: AgentId,
    pub revision_id: RevisionId,
    pub abom: AgentBillOfMaterials,
    pub policy_pack_id: PolicyPackId,
    pub corpus_version: String,
    #[serde(default)]
    pub vulnerabilities: Vec<Vulnerability>,
}

impl CandidateRevision {
    /// Compute the immutable `RevisionId` as the SHA-256 digest of the canonical ABOM.
    pub fn compute_revision_id(abom: &AgentBillOfMaterials) -> RevisionId {
        let canonical_bytes = serde_json::to_vec(abom).expect("ABOM serializes");
        let digest = RevisionDigest::sha256(&canonical_bytes);
        RevisionId::from_digest(digest.0)
    }

    pub fn new(
        tenant_id: TenantId,
        agent_id: AgentId,
        abom: AgentBillOfMaterials,
        policy_pack_id: PolicyPackId,
        corpus_version: String,
        vulnerabilities: Vec<Vulnerability>,
    ) -> Self {
        let revision_id = Self::compute_revision_id(&abom);
        Self {
            tenant_id,
            agent_id,
            revision_id,
            abom,
            policy_pack_id,
            corpus_version,
            vulnerabilities,
        }
    }
}

/// Structural difference between two candidate revisions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RevisionComparison {
    pub base_revision_id: RevisionId,
    pub target_revision_id: RevisionId,
    pub source_changed: bool,
    pub model_changed: bool,
    pub prompt_changed: bool,
    pub tools_changed: bool,
    pub memory_changed: bool,
    pub gateway_policy_changed: bool,
    pub model_armor_changed: bool,
    pub added_capabilities: Vec<String>,
    pub removed_capabilities: Vec<String>,
    pub risk_tier_changed: bool,
}

impl RevisionComparison {
    pub fn compare(base: &AgentBillOfMaterials, target: &AgentBillOfMaterials) -> Self {
        let base_id = CandidateRevision::compute_revision_id(base);
        let target_id = CandidateRevision::compute_revision_id(target);

        let added_capabilities = target
            .requested_capabilities
            .iter()
            .filter(|c| !base.requested_capabilities.contains(c))
            .cloned()
            .collect();

        let removed_capabilities = base
            .requested_capabilities
            .iter()
            .filter(|c| !target.requested_capabilities.contains(c))
            .cloned()
            .collect();

        Self {
            base_revision_id: base_id,
            target_revision_id: target_id,
            source_changed: base.source_digest != target.source_digest,
            model_changed: base.model_ref != target.model_ref,
            prompt_changed: base.prompt_config_digest != target.prompt_config_digest,
            tools_changed: base.tool_manifest_digest != target.tool_manifest_digest,
            memory_changed: base.memory_config_digest != target.memory_config_digest,
            gateway_policy_changed: base.gateway_policy_digest != target.gateway_policy_digest,
            model_armor_changed: base.model_armor_config_digest != target.model_armor_config_digest,
            added_capabilities,
            removed_capabilities,
            risk_tier_changed: base.risk_tier != target.risk_tier,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub id: String,
    pub title: String,
    pub severity: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResponse {
    pub status: String,
    /// Provenance of the scan result:
    /// - [`Provenance::Local`] — no GCP credentials / metadata-token available
    ///   (local dev, CI, or a deployment without on-demand-scanning scope), so the
    ///   result is an honest empty placeholder, never fabricated CVEs.
    /// - [`Provenance::Live`] — produced by a live On-Demand Scanning API call.
    pub provenance: Provenance,
    pub vulnerabilities: Vec<Vulnerability>,
}
