//! Findings emitted during certification.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{CaseId, CaseRunId, PolicyRuleRef, TenantId, WorkflowId};
use crate::provenance::Provenance;

/// A single finding from policy evaluation or evidence analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub finding_id: uuid::Uuid,
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub case_run_id: Option<CaseRunId>,
    pub case_id: Option<CaseId>,
    pub rule_id: Option<PolicyRuleRef>,
    pub kind: FindingKind,
    pub severity: FindingSeverity,
    pub title: String,
    pub description: String,
    pub evidence_refs: Vec<String>, // evidence object digests
    pub remediation: Option<Remediation>,
    pub provenance: Provenance,
    pub created_at: OffsetDateTime,
}

/// Classification of finding type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum FindingKind {
    /// Required evidence missing or malformed.
    EvidenceMissing,
    /// Ledger invariant violated (unauthorized side effect).
    InvariantViolation,
    /// Model Armor blocked content.
    ModelArmorBlock,
    /// Gateway denied an action.
    GatewayDeny,
    /// Gateway allowed an action (informational).
    GatewayAllow,
    /// Memory Bank disposition retrieved.
    MemoryRetrieved,
    /// Policy rule failed.
    RuleFailed,
    /// Approval required (capability reduction).
    ApprovalRequired,
    /// Retest required.
    RetestRequired,
    /// Schema validation failed.
    SchemaViolation,
    /// Infrastructure error.
    InfrastructureError,
}

/// Severity for prioritization and alerting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

/// Recommended remediation action (advisory only; never auto-executed).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Remediation {
    pub action: RemediationAction,
    pub detail: String,
    /// Whether this remediation requires human approval to apply.
    pub requires_approval: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RemediationAction {
    /// Narrow the requested capability set.
    ReduceCapabilities,
    /// Update prompt/config and retest.
    UpdateConfiguration,
    /// Fix tool schema or MCP server.
    FixToolDefinition,
    /// Update policy pack.
    UpdatePolicy,
    /// Manual investigation required.
    Investigate,
}
