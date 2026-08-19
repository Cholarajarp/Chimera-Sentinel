//! Policy pack, rules, conditions, and evaluation result models.

use serde::{Deserialize, Serialize};
use time::OffsetDateTime;

use crate::ids::{CaseId, PolicyPackId, PolicyRuleRef};
pub use crate::workflow::GateDecision;

/// Declarative policy pack defining certification criteria.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPack {
    pub id: PolicyPackId,
    pub version: String,
    pub name: String,
    pub description: String,
    pub digest: String,
    pub required_cases: Vec<CaseId>,
    pub required_managed_controls: Vec<ManagedControlRequirement>,
    pub rules: Vec<PolicyRule>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thresholds: Option<ConfigurableThresholds>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ConfigurableThresholds {
    pub financial_tolerance: f64,
    pub enforce_least_privilege: bool,
    pub enforce_prompt_firewall: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ManagedControlRequirement {
    ModelArmor,
    AgentGateway,
    AgentIdentity,
    MemoryBank,
    LedgerOracle,
    OpenTelemetryTrace,
}

/// A specific rule within a policy pack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyRule {
    pub id: PolicyRuleRef,
    pub name: String,
    pub description: String,
    pub severity: RuleSeverity,
    pub rule_type: RuleType,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleSeverity {
    Critical,
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleType {
    CandidateImmutability,
    LedgerInvariant,
    GatewayLeastPrivilege,
    ModelArmorProtection,
    CaseCoverage,
    ApprovalVerification,
    RetestVerification,
    EvidenceIntegrity,
}

/// Result of evaluating a complete policy pack against an evidence bundle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyEvaluation {
    pub pack_id: PolicyPackId,
    pub pack_version: String,
    pub gate_decision: GateDecision,
    pub explanation: String,
    pub rule_results: Vec<RuleEvaluationResult>,
    #[serde(with = "time::serde::rfc3339")]
    pub evaluated_at: OffsetDateTime,
}

/// Result of a single rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleEvaluationResult {
    pub rule_ref: PolicyRuleRef,
    pub passed: bool,
    pub explanation: String,
    pub evidence_refs: Vec<String>,
    pub recommended_capability_diff: Option<CapabilityReduction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityReduction {
    pub current_capabilities: Vec<String>,
    pub proposed_capabilities: Vec<String>,
    pub removed_capabilities: Vec<String>,
    pub reason: String,
}
