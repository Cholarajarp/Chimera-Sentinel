//! Chimera Sentinel core domain.
//!
//! This crate holds pure domain types and the durable workflow state machine.
//! It deliberately has no I/O, no HTTP, and no provider dependencies: it is the
//! trusted spine of the control plane and is exhaustively unit tested.
//!
//! Authority rule: a domain type may describe evidence, recommendations, or
//! observations, but only [`policy`] (in the `sentinel-policy` crate, over
//! normalized evidence) computes the final gate decision. Nothing here can in
//! itself authorize a production promotion.

#![forbid(unsafe_code)]

pub mod attestation;
pub mod audit;
pub mod candidate;
pub mod clock;
pub mod error;
pub mod evidence;
pub mod finding;
pub mod ids;
pub mod ledger;
pub mod policy;
pub mod provenance;
pub mod tooling;
pub mod workflow;

pub use attestation::{AttestationMetadata, AttestationStatus, KeyReference};
pub use audit::{AuditEvent, AuditEventType};
pub use candidate::{AgentBillOfMaterials, CandidateRevision, RevisionDigest};
pub use clock::{Clock, SystemClock, TestClock};
pub use error::{DomainError, TransitionError};
pub use evidence::{EvidenceManifestRef, EvidenceObjectRef};
pub use finding::{Finding, FindingKind, FindingSeverity, Remediation};
pub use ids::{
    AgentId, ApprovalId, CaseId, CaseRunId, CommandId, IdempotencyKey, PolicyPackId, PolicyRuleRef,
    PrincipalId, RevisionId, TenantId, WorkflowId,
};
pub use policy::{GateDecision, PolicyPack};
pub use provenance::Provenance;
pub use tooling::{ToolAction, ToolManifest, ToolPermission};

// `workflow` is re-exported as a module path because callers need both the
// state enum and the aggregate root; exposing the module keeps that explicit.
pub use workflow::{Workflow, WorkflowCommand, WorkflowState};
