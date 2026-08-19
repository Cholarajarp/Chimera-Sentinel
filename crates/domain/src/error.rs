//! Domain errors.

use thiserror::Error;

/// An error produced while applying a domain rule or transition.
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("invalid transition from {from:?} via {command:?}")]
    InvalidTransition {
        from: crate::workflow::WorkflowState,
        command: crate::workflow::WorkflowCommand,
    },

    #[error("duplicate idempotent command {0}")]
    DuplicateCommand(uuid::Uuid),

    #[error("required evidence missing: {0}")]
    MissingEvidence(String),

    #[error("ledger invariant violated: {0}")]
    LedgerInvariant(String),

    #[error("approval scoped broader than request")]
    ApprovalBroadensCapability,

    #[error("approval expired at {0}")]
    ApprovalExpired(time::OffsetDateTime),

    #[error("attestation rejected: {0}")]
    AttestationRejected(String),

    #[error("must not broaden permissions or suppress failed invariants")]
    CannotOverrideInvariant,

    #[error("entity not found: {0}")]
    NotFound(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("invalid input: {0}")]
    Invalid(String),
}

/// Specialised error for state-machine violations that keep a workflow blocked.
#[derive(Debug, Error)]
pub enum TransitionError {
    #[error("illegal transition from {from:?} via {command:?}: {reason}")]
    Illegal {
        from: crate::workflow::WorkflowState,
        command: crate::workflow::WorkflowCommand,
        reason: &'static str,
    },

    #[error("terminal state {0:?} is immutable")]
    Terminal(crate::workflow::WorkflowState),

    #[error("version conflict: expected {expected}, got {actual}")]
    VersionConflict { expected: u64, actual: u64 },
}

impl From<TransitionError> for DomainError {
    fn from(value: TransitionError) -> Self {
        match value {
            TransitionError::Illegal { from, command, .. } => {
                DomainError::InvalidTransition { from, command }
            }
            TransitionError::Terminal(state) => {
                DomainError::Invalid(format!("terminal state {state:?}"))
            }
            TransitionError::VersionConflict { expected, actual } => {
                DomainError::Invalid(format!("version conflict expected {expected} got {actual}"))
            }
        }
    }
}
