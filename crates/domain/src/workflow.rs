//! Durable workflow state machine and aggregate root.
//!
//! State transitions follow the exact state machine defined in docs/ARCHITECTURE.md.
//! Terminal states are immutable. All mutations require command idempotency and
//! optimistic concurrency version checks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use time::OffsetDateTime;

use crate::error::TransitionError;
use crate::ids::{ApprovalId, PolicyPackId, RevisionId, TenantId, WorkflowId};

/// Exhaustive workflow states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WorkflowState {
    /// Newly registered candidate workflow, pending start.
    Registered,
    /// Certification started, queued for execution.
    Queued,
    /// Cases currently being executed by the worker and certifier.
    Running,
    /// Case execution complete, evidence bundle pending evaluation.
    EvidencePending,
    /// Excessive agency or risk detected; constrained human approval required.
    ApprovalRequired,
    /// Human approval recorded or smoke tests needed; retest execution required.
    RetestRequired,
    /// Policy evaluation passed; Cloud KMS attestation signing in progress.
    Attesting,
    /// Terminal release admission state; valid signed attestation generated.
    Certified,
    /// Attestation nearing expiry or material drift detected.
    RecertificationRequired,
    /// Attestation revoked due to security incident or drift.
    Revoked,
    /// Terminal failure: critical invariant violated or required evidence missing.
    Blocked,
    /// Terminal state: reviewer explicitly rejected the release candidate.
    Rejected,
    /// Terminal state: unrecoverable infrastructure failure occurred.
    Failed,
    /// Terminal state: authorized administrator cancelled the workflow.
    Cancelled,
}

impl WorkflowState {
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Certified | Self::Blocked | Self::Rejected | Self::Failed | Self::Cancelled | Self::Revoked
        )
    }

    pub fn is_active(&self) -> bool {
        !self.is_terminal()
    }
}

/// Deterministic policy gate decision outcomes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateDecision {
    Blocked,
    ConstrainedApprovalRequired,
    RetestRequired,
    Certifiable,
}

/// Commands driving the workflow state machine.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload")]
pub enum WorkflowCommand {
    Start,
    Claim { worker_id: String },
    EvidenceCollected,
    EvaluationFinished {
        decision: GateDecision,
        explanation: String,
    },
    Approve {
        approval_id: ApprovalId,
        constrained_capabilities: Vec<String>,
    },
    Reject {
        reason: String,
    },
    DispatchRetest,
    RetestFinished {
        passed: bool,
    },
    AttestationSigned {
        attestation_digest: String,
        signature: String,
        key_version: String,
    },
    Revoke {
        reason: String,
    },
    Fail {
        reason: String,
    },
    Cancel {
        reason: String,
    },
}

/// Durable workflow aggregate root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub tenant_id: TenantId,
    pub workflow_id: WorkflowId,
    pub candidate_revision_id: RevisionId,
    pub state: WorkflowState,
    pub version: u64,
    pub policy_pack_id: PolicyPackId,
    pub corpus_version: String,
    pub gate_decision: Option<GateDecision>,
    pub gate_explanation: Option<String>,
    pub approval_id: Option<ApprovalId>,
    pub attestation_digest: Option<String>,
    pub attestation_signature: Option<String>,
    pub attestation_key_version: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
    pub provider_refs: HashMap<String, String>,
}

impl Workflow {
    pub fn new(
        tenant_id: TenantId,
        candidate_revision_id: RevisionId,
        policy_pack_id: PolicyPackId,
        corpus_version: String,
    ) -> Self {
        let now = OffsetDateTime::now_utc();
        Self {
            tenant_id,
            workflow_id: WorkflowId::new(),
            candidate_revision_id,
            state: WorkflowState::Registered,
            version: 1,
            policy_pack_id,
            corpus_version,
            gate_decision: None,
            gate_explanation: None,
            approval_id: None,
            attestation_digest: None,
            attestation_signature: None,
            attestation_key_version: None,
            created_at: now,
            updated_at: now,
            provider_refs: HashMap::new(),
        }
    }

    /// Apply a command and return the new state if valid.
    pub fn apply(
        &mut self,
        command: WorkflowCommand,
        expected_version: u64,
        now: OffsetDateTime,
    ) -> Result<WorkflowState, TransitionError> {
        if self.version != expected_version {
            return Err(TransitionError::VersionConflict {
                expected: expected_version,
                actual: self.version,
            });
        }

        if self.state.is_terminal() {
            // Only Revoke is allowed from Certified
            if let (WorkflowState::Certified, WorkflowCommand::Revoke { .. }) = (&self.state, &command) {
                self.state = WorkflowState::Revoked;
                self.version += 1;
                self.updated_at = now;
                return Ok(self.state);
            }
            return Err(TransitionError::Terminal(self.state));
        }

        let next_state = match (&self.state, &command) {
            (WorkflowState::Registered, WorkflowCommand::Start) => WorkflowState::Queued,
            (WorkflowState::Queued, WorkflowCommand::Claim { .. }) => WorkflowState::Running,
            (WorkflowState::Running, WorkflowCommand::EvidenceCollected) => WorkflowState::EvidencePending,
            (
                WorkflowState::EvidencePending,
                WorkflowCommand::EvaluationFinished {
                    decision: GateDecision::Blocked,
                    explanation,
                },
            ) => {
                self.gate_decision = Some(GateDecision::Blocked);
                self.gate_explanation = Some(explanation.clone());
                WorkflowState::Blocked
            }
            (
                WorkflowState::EvidencePending,
                WorkflowCommand::EvaluationFinished {
                    decision: GateDecision::ConstrainedApprovalRequired,
                    explanation,
                },
            ) => {
                self.gate_decision = Some(GateDecision::ConstrainedApprovalRequired);
                self.gate_explanation = Some(explanation.clone());
                WorkflowState::ApprovalRequired
            }
            (
                WorkflowState::EvidencePending,
                WorkflowCommand::EvaluationFinished {
                    decision: GateDecision::RetestRequired,
                    explanation,
                },
            ) => {
                self.gate_decision = Some(GateDecision::RetestRequired);
                self.gate_explanation = Some(explanation.clone());
                WorkflowState::RetestRequired
            }
            (
                WorkflowState::EvidencePending,
                WorkflowCommand::EvaluationFinished {
                    decision: GateDecision::Certifiable,
                    explanation,
                },
            ) => {
                self.gate_decision = Some(GateDecision::Certifiable);
                self.gate_explanation = Some(explanation.clone());
                WorkflowState::Attesting
            }
            (WorkflowState::ApprovalRequired, WorkflowCommand::Approve { approval_id, .. }) => {
                self.approval_id = Some(*approval_id);
                WorkflowState::RetestRequired
            }
            (WorkflowState::ApprovalRequired, WorkflowCommand::Reject { reason }) => {
                self.gate_explanation = Some(reason.clone());
                WorkflowState::Rejected
            }
            (WorkflowState::RetestRequired, WorkflowCommand::DispatchRetest) => WorkflowState::Running,
            (WorkflowState::Running, WorkflowCommand::RetestFinished { passed: true }) => {
                WorkflowState::Attesting
            }
            (WorkflowState::Running, WorkflowCommand::RetestFinished { passed: false }) => {
                WorkflowState::Blocked
            }
            (
                WorkflowState::Attesting,
                WorkflowCommand::AttestationSigned {
                    attestation_digest,
                    signature,
                    key_version,
                },
            ) => {
                self.attestation_digest = Some(attestation_digest.clone());
                self.attestation_signature = Some(signature.clone());
                self.attestation_key_version = Some(key_version.clone());
                WorkflowState::Certified
            }
            (_, WorkflowCommand::Fail { reason }) => {
                self.gate_explanation = Some(reason.clone());
                WorkflowState::Failed
            }
            (WorkflowState::Registered | WorkflowState::Queued, WorkflowCommand::Cancel { reason }) => {
                self.gate_explanation = Some(reason.clone());
                WorkflowState::Cancelled
            }
            _ => {
                return Err(TransitionError::Illegal {
                    from: self.state,
                    command,
                    reason: "invalid transition according to state machine",
                });
            }
        };

        self.state = next_state;
        self.version += 1;
        self.updated_at = now;
        Ok(self.state)
    }

    pub fn to_summary(&self) -> WorkflowSummary {
        WorkflowSummary {
            workflow_id: self.workflow_id,
            tenant_id: self.tenant_id,
            candidate_revision_id: self.candidate_revision_id.clone(),
            state: self.state,
            gate_decision: self.gate_decision,
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}

/// Lightweight projection of a workflow for listing and dashboards.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowSummary {
    pub workflow_id: WorkflowId,
    pub tenant_id: TenantId,
    pub candidate_revision_id: RevisionId,
    pub state: WorkflowState,
    pub gate_decision: Option<GateDecision>,
    #[serde(with = "time::serde::rfc3339")]
    pub created_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339")]
    pub updated_at: OffsetDateTime,
}
