#![allow(missing_docs)]
//! Persistence layer providing abstract repository traits and concrete implementations.
//!
//! Two backends:
//!   - `memory::InMemoryStore`  — local dev, tests, standalone runs (no persistence across restarts)
//!   - `firestore::FirestoreStore` — production; uses Firestore REST API with workload identity
//!
//! Select via `DatabaseConfig` in `sentinel-config`:
//!   - `DatabaseConfig::Memory`    → InMemoryStore
//!   - `DatabaseConfig::Firestore` → FirestoreStore
//!
//! Enforces optimistic concurrency controls, tenant isolation, and version checks.

pub mod firestore;

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use sentinel_domain::{
    attestation::AttestationMetadata,
    candidate::CandidateRevision,
    evidence::{Approval, EvidenceManifestRef},
    finding::Finding,
    ids::{ApprovalId, RevisionId, TenantId, WorkflowId},
    workflow::{Workflow, WorkflowState, WorkflowSummary},
    AuditEvent,
};

#[async_trait]
pub trait CandidateRepository: Send + Sync {
    async fn create(&self, candidate: &CandidateRevision) -> Result<(), String>;
    async fn get(
        &self,
        tenant_id: TenantId,
        revision_id: &RevisionId,
    ) -> Result<Option<CandidateRevision>, String>;
    async fn list(
        &self,
        tenant_id: TenantId,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<Vec<CandidateRevision>, String>;
    async fn delete(&self, tenant_id: TenantId, revision_id: &RevisionId) -> Result<(), String>;
}

#[async_trait]
pub trait WorkflowRepository: Send + Sync {
    async fn create(&self, workflow: &Workflow) -> Result<(), String>;
    async fn get(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<Workflow>, String>;
    async fn update(&self, workflow: &Workflow, expected_version: u64) -> Result<(), String>;
    async fn list(
        &self,
        tenant_id: TenantId,
        state: Option<WorkflowState>,
        limit: usize,
        cursor: Option<String>,
    ) -> Result<Vec<WorkflowSummary>, String>;
    async fn delete(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<(), String>;
}

#[async_trait]
pub trait EvidenceRepository: Send + Sync {
    async fn store_manifest(&self, manifest: &EvidenceManifestRef) -> Result<(), String>;
    async fn get_manifest(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<EvidenceManifestRef>, String>;
}

#[async_trait]
pub trait AuditRepository: Send + Sync {
    async fn append(&self, event: &AuditEvent) -> Result<(), String>;
    async fn query(
        &self,
        tenant_id: TenantId,
        workflow_id: Option<WorkflowId>,
        limit: usize,
    ) -> Result<Vec<AuditEvent>, String>;
}

#[async_trait]
pub trait FindingRepository: Send + Sync {
    async fn store(&self, finding: &Finding) -> Result<(), String>;
    async fn list(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Vec<Finding>, String>;
}

#[async_trait]
pub trait ApprovalRepository: Send + Sync {
    async fn create(&self, approval: &Approval) -> Result<(), String>;
    async fn get(
        &self,
        tenant_id: TenantId,
        approval_id: ApprovalId,
    ) -> Result<Option<Approval>, String>;
    async fn get_for_workflow(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<Approval>, String>;
}

#[async_trait]
pub trait AttestationRepository: Send + Sync {
    async fn store(
        &self,
        workflow_id: WorkflowId,
        attestation: &AttestationMetadata,
    ) -> Result<(), String>;
    async fn get(
        &self,
        tenant_id: TenantId,
        workflow_id: WorkflowId,
    ) -> Result<Option<AttestationMetadata>, String>;
}

/// In-memory implementations for reliable local dev, integration tests, and standalone runs.
pub mod memory {
    use super::*;

    #[derive(Clone, Default)]
    pub struct InMemoryStore {
        candidates: Arc<RwLock<HashMap<(TenantId, String), CandidateRevision>>>,
        workflows: Arc<RwLock<HashMap<(TenantId, WorkflowId), Workflow>>>,
        manifests: Arc<RwLock<HashMap<(TenantId, WorkflowId), EvidenceManifestRef>>>,
        audit_events: Arc<RwLock<Vec<AuditEvent>>>,
        findings: Arc<RwLock<Vec<Finding>>>,
        approvals: Arc<RwLock<HashMap<(TenantId, ApprovalId), Approval>>>,
        attestations: Arc<RwLock<HashMap<(TenantId, WorkflowId), AttestationMetadata>>>,
    }

    impl InMemoryStore {
        pub fn new() -> Self {
            Self::default()
        }
    }

    #[async_trait]
    impl CandidateRepository for InMemoryStore {
        async fn create(&self, candidate: &CandidateRevision) -> Result<(), String> {
            let mut data = self.candidates.write().unwrap();
            let key = (candidate.tenant_id, candidate.revision_id.0.clone());
            data.insert(key, candidate.clone());
            Ok(())
        }

        async fn get(
            &self,
            tenant_id: TenantId,
            revision_id: &RevisionId,
        ) -> Result<Option<CandidateRevision>, String> {
            let data = self.candidates.read().unwrap();
            Ok(data.get(&(tenant_id, revision_id.0.clone())).cloned())
        }

        async fn list(
            &self,
            tenant_id: TenantId,
            limit: usize,
            _cursor: Option<String>,
        ) -> Result<Vec<CandidateRevision>, String> {
            let data = self.candidates.read().unwrap();
            let mut results: Vec<_> = data
                .iter()
                .filter(|((t, _), _)| *t == tenant_id)
                .map(|(_, v)| v.clone())
                .collect();
            results.sort_by(|a, b| b.abom.recorded_at.cmp(&a.abom.recorded_at));
            results.truncate(limit);
            Ok(results)
        }

        async fn delete(
            &self,
            tenant_id: TenantId,
            revision_id: &RevisionId,
        ) -> Result<(), String> {
            let mut data = self.candidates.write().unwrap();
            data.remove(&(tenant_id, revision_id.0.clone()));
            Ok(())
        }
    }

    #[async_trait]
    impl WorkflowRepository for InMemoryStore {
        async fn create(&self, workflow: &Workflow) -> Result<(), String> {
            let mut data = self.workflows.write().unwrap();
            data.insert((workflow.tenant_id, workflow.workflow_id), workflow.clone());
            Ok(())
        }

        async fn get(
            &self,
            tenant_id: TenantId,
            workflow_id: WorkflowId,
        ) -> Result<Option<Workflow>, String> {
            let data = self.workflows.read().unwrap();
            Ok(data.get(&(tenant_id, workflow_id)).cloned())
        }

        async fn update(&self, workflow: &Workflow, expected_version: u64) -> Result<(), String> {
            let mut data = self.workflows.write().unwrap();
            let key = (workflow.tenant_id, workflow.workflow_id);
            if let Some(existing) = data.get(&key) {
                if existing.version != expected_version {
                    return Err(format!(
                        "Optimistic concurrency failure: expected version {}, found {}",
                        expected_version, existing.version
                    ));
                }
            }
            data.insert(key, workflow.clone());
            Ok(())
        }

        async fn list(
            &self,
            tenant_id: TenantId,
            state: Option<WorkflowState>,
            limit: usize,
            _cursor: Option<String>,
        ) -> Result<Vec<WorkflowSummary>, String> {
            let data = self.workflows.read().unwrap();
            let mut results: Vec<_> = data
                .iter()
                .filter(|((t, _), w)| *t == tenant_id && state.map_or(true, |s| w.state == s))
                .map(|(_, w)| w.to_summary())
                .collect();
            results.sort_by(|a, b| b.created_at.cmp(&a.created_at));
            results.truncate(limit);
            Ok(results)
        }

        async fn delete(&self, tenant_id: TenantId, workflow_id: WorkflowId) -> Result<(), String> {
            let mut data = self.workflows.write().unwrap();
            data.remove(&(tenant_id, workflow_id));
            Ok(())
        }
    }

    #[async_trait]
    impl EvidenceRepository for InMemoryStore {
        async fn store_manifest(&self, manifest: &EvidenceManifestRef) -> Result<(), String> {
            let mut data = self.manifests.write().unwrap();
            data.insert((manifest.tenant_id, manifest.workflow_id), manifest.clone());
            Ok(())
        }

        async fn get_manifest(
            &self,
            tenant_id: TenantId,
            workflow_id: WorkflowId,
        ) -> Result<Option<EvidenceManifestRef>, String> {
            let data = self.manifests.read().unwrap();
            Ok(data.get(&(tenant_id, workflow_id)).cloned())
        }
    }

    #[async_trait]
    impl AuditRepository for InMemoryStore {
        async fn append(&self, event: &AuditEvent) -> Result<(), String> {
            let mut data = self.audit_events.write().unwrap();
            data.push(event.clone());
            Ok(())
        }

        async fn query(
            &self,
            tenant_id: TenantId,
            workflow_id: Option<WorkflowId>,
            limit: usize,
        ) -> Result<Vec<AuditEvent>, String> {
            let data = self.audit_events.read().unwrap();
            let mut results: Vec<_> = data
                .iter()
                .filter(|e| {
                    e.tenant_id == tenant_id
                        && workflow_id.map_or(true, |wid| e.workflow_id == Some(wid))
                })
                .cloned()
                .collect();
            results.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
            results.truncate(limit);
            Ok(results)
        }
    }

    #[async_trait]
    impl FindingRepository for InMemoryStore {
        async fn store(&self, finding: &Finding) -> Result<(), String> {
            let mut data = self.findings.write().unwrap();
            data.push(finding.clone());
            Ok(())
        }

        async fn list(
            &self,
            tenant_id: TenantId,
            workflow_id: WorkflowId,
        ) -> Result<Vec<Finding>, String> {
            let data = self.findings.read().unwrap();
            let results: Vec<_> = data
                .iter()
                .filter(|f| f.tenant_id == tenant_id && f.workflow_id == workflow_id)
                .cloned()
                .collect();
            Ok(results)
        }
    }

    #[async_trait]
    impl ApprovalRepository for InMemoryStore {
        async fn create(&self, approval: &Approval) -> Result<(), String> {
            let mut data = self.approvals.write().unwrap();
            data.insert((approval.tenant_id, approval.approval_id), approval.clone());
            Ok(())
        }

        async fn get(
            &self,
            tenant_id: TenantId,
            approval_id: ApprovalId,
        ) -> Result<Option<Approval>, String> {
            let data = self.approvals.read().unwrap();
            Ok(data.get(&(tenant_id, approval_id)).cloned())
        }

        async fn get_for_workflow(
            &self,
            tenant_id: TenantId,
            workflow_id: WorkflowId,
        ) -> Result<Option<Approval>, String> {
            let data = self.approvals.read().unwrap();
            Ok(data
                .iter()
                .find(|((t, _), a)| *t == tenant_id && a.workflow_id == workflow_id)
                .map(|(_, a)| a.clone()))
        }
    }

    #[async_trait]
    impl AttestationRepository for InMemoryStore {
        async fn store(
            &self,
            workflow_id: WorkflowId,
            attestation: &AttestationMetadata,
        ) -> Result<(), String> {
            let mut data = self.attestations.write().unwrap();
            data.insert(
                (attestation.payload.tenant_id, workflow_id),
                attestation.clone(),
            );
            Ok(())
        }

        async fn get(
            &self,
            tenant_id: TenantId,
            workflow_id: WorkflowId,
        ) -> Result<Option<AttestationMetadata>, String> {
            let data = self.attestations.read().unwrap();
            Ok(data.get(&(tenant_id, workflow_id)).cloned())
        }
    }
}
