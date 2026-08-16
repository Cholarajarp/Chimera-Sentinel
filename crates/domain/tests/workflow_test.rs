use time::OffsetDateTime;
use uuid::Uuid;

use sentinel_domain::{
    candidate::{AgentBillOfMaterials, CandidateRevision, RevisionDigest, RiskTier},
    ids::{AgentId, ApprovalId, PolicyPackId, PrincipalId, TenantId},
    provenance::Provenance,
    workflow::{GateDecision, Workflow, WorkflowCommand, WorkflowState},
};

#[test]
fn test_abom_revision_id_determinism() {
    let now = OffsetDateTime::now_utc();
    let abom = AgentBillOfMaterials {
        source_digest: RevisionDigest::sha256(b"source-1"),
        registry_resource: "projects/p/locations/l/agents/a".to_string(),
        runtime_resource: "projects/p/locations/l/runtimes/r".to_string(),
        agent_identity: PrincipalId::new("sa@project.iam"),
        model_ref: "gemini-3.5-flash".to_string(),
        prompt_config_digest: RevisionDigest::sha256(b"prompt-1"),
        tool_manifest_digest: RevisionDigest::sha256(b"tool-1"),
        memory_config_digest: RevisionDigest::sha256(b"mem-1"),
        gateway_policy_digest: RevisionDigest::sha256(b"gw-1"),
        model_armor_config_digest: RevisionDigest::sha256(b"armor-1"),
        requested_capabilities: vec!["draft_invoice_payment".to_string()],
        data_classification: vec!["SYNTHETIC".to_string()],
        environment: "production".to_string(),
        owner: PrincipalId::new("dev@corp"),
        risk_tier: RiskTier::Medium,
        provenance: Provenance::Live,
        recorded_at: now,
    };

    let rev1 = CandidateRevision::compute_revision_id(&abom);
    let rev2 = CandidateRevision::compute_revision_id(&abom);
    assert_eq!(rev1, rev2);
    assert!(rev1.0.starts_with("sha256:"));
}

#[test]
fn test_workflow_state_machine_happy_path() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let rev_id = sentinel_domain::candidate::RevisionId::from_digest("sha256:1234");
    let mut wf = Workflow::new(tenant_id, rev_id, PolicyPackId::new("ap-v1"), "v1".to_string());

    assert_eq!(wf.state, WorkflowState::Registered);
    assert_eq!(wf.version, 1);

    // 1. Registered -> Queued
    let state = wf.apply(WorkflowCommand::Start, 1, now).unwrap();
    assert_eq!(state, WorkflowState::Queued);
    assert_eq!(wf.version, 2);

    // 2. Queued -> Running
    let state = wf.apply(WorkflowCommand::Claim { worker_id: "w1".to_string() }, 2, now).unwrap();
    assert_eq!(state, WorkflowState::Running);
    assert_eq!(wf.version, 3);

    // 3. Running -> EvidencePending
    let state = wf.apply(WorkflowCommand::EvidenceCollected, 3, now).unwrap();
    assert_eq!(state, WorkflowState::EvidencePending);

    // 4. EvidencePending -> ApprovalRequired
    let state = wf.apply(
        WorkflowCommand::EvaluationFinished {
            decision: GateDecision::ConstrainedApprovalRequired,
            explanation: "Reviewer approval needed".to_string(),
        },
        4,
        now,
    ).unwrap();
    assert_eq!(state, WorkflowState::ApprovalRequired);

    // 5. ApprovalRequired -> RetestRequired
    let app_id = ApprovalId::new();
    let state = wf.apply(
        WorkflowCommand::Approve {
            approval_id: app_id,
            constrained_capabilities: vec!["draft_invoice_payment".to_string()],
        },
        5,
        now,
    ).unwrap();
    assert_eq!(state, WorkflowState::RetestRequired);

    // 6. RetestRequired -> Running (retest dispatch)
    let state = wf.apply(WorkflowCommand::DispatchRetest, 6, now).unwrap();
    assert_eq!(state, WorkflowState::Running);

    // 7. Running -> Attesting (retest passed)
    let state = wf.apply(WorkflowCommand::RetestFinished { passed: true }, 7, now).unwrap();
    assert_eq!(state, WorkflowState::Attesting);

    // 8. Attesting -> Certified
    let state = wf.apply(
        WorkflowCommand::AttestationSigned {
            attestation_digest: "sha256:digest".to_string(),
            signature: "sig123".to_string(),
            key_version: "1".to_string(),
        },
        8,
        now,
    ).unwrap();
    assert_eq!(state, WorkflowState::Certified);
    assert!(wf.state.is_terminal());
}

#[test]
fn test_workflow_version_conflict_rejection() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let rev_id = sentinel_domain::candidate::RevisionId::from_digest("sha256:1234");
    let mut wf = Workflow::new(tenant_id, rev_id, PolicyPackId::new("ap-v1"), "v1".to_string());

    // Expected version is 1, pass wrong version 99
    let err = wf.apply(WorkflowCommand::Start, 99, now);
    assert!(err.is_err());
}
