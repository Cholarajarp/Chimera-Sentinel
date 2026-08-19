//! Policy evaluation tests
use time::OffsetDateTime;

use sentinel_domain::{
    evidence::Approval,
    ids::{ApprovalId, PrincipalId, TenantId, WorkflowId},
    policy::GateDecision,
};
use sentinel_policy::{evaluate, RetestResult};
use sentinel_test_support::{test_candidate_revision, test_evidence_bundle, test_policy_pack};

#[test]
fn test_policy_evaluation_excessive_agency_requires_approval() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(
        tenant_id,
        vec![
            "draft_invoice_payment".to_string(),
            "release_payment".to_string(),
        ],
    );

    let (bundle, ledger) = test_evidence_bundle(tenant_id, candidate);
    let pack = test_policy_pack();

    let evaluation = evaluate(&pack, &bundle, &ledger, now);
    assert_eq!(
        evaluation.gate_decision,
        GateDecision::ConstrainedApprovalRequired
    );

    // Verify capability diff was generated
    let agency_rule = evaluation
        .rule_results
        .iter()
        .find(|r| r.rule_ref.0 == "RULE-005-LEAST-PRIVILEGE-AGENCY")
        .unwrap();
    assert!(!agency_rule.passed);
    assert!(agency_rule.recommended_capability_diff.is_some());
    let diff = agency_rule.recommended_capability_diff.as_ref().unwrap();
    assert_eq!(
        diff.removed_capabilities,
        vec!["release_payment".to_string()]
    );
}

#[test]
fn test_policy_evaluation_unauthorized_payment_release_blocks_promotion() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(tenant_id, vec!["draft_invoice_payment".to_string()]);

    let (bundle, mut ledger) = test_evidence_bundle(tenant_id, candidate);
    // Invariant violation: unauthorized payment occurred
    ledger.zero_unauthorized_releases = false;

    let pack = test_policy_pack();
    let evaluation = evaluate(&pack, &bundle, &ledger, now);

    assert_eq!(evaluation.gate_decision, GateDecision::Blocked);
    let ledger_rule = evaluation
        .rule_results
        .iter()
        .find(|r| r.rule_ref.0 == "RULE-002-LEDGER-INVARIANTS")
        .unwrap();
    assert!(!ledger_rule.passed);
}

#[test]
fn test_policy_evaluation_wrong_required_case_outcome_blocks_promotion() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(tenant_id, vec!["draft_invoice_payment".to_string()]);
    let (mut bundle, ledger) = test_evidence_bundle(tenant_id, candidate);
    let required_case = bundle
        .cases
        .iter_mut()
        .find(|case| case.case_id.0 == "case-safe-draft-001")
        .expect("required safe case exists");
    required_case.observed_outcome = "PREVENTED_AT_GATEWAY".to_string();

    let evaluation = evaluate(&test_policy_pack(), &bundle, &ledger, now);

    assert_eq!(evaluation.gate_decision, GateDecision::Blocked);
    let coverage_rule = evaluation
        .rule_results
        .iter()
        .find(|result| result.rule_ref.0 == "RULE-004-CASE-COVERAGE")
        .expect("coverage rule evaluated");
    assert!(!coverage_rule.passed);
    assert!(coverage_rule.explanation.contains("outcomes mismatched"));
}

#[test]
fn test_policy_evaluation_separation_of_duties_blocks_self_approval() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(
        tenant_id,
        vec![
            "draft_invoice_payment".to_string(),
            "release_payment".to_string(),
        ],
    );

    let (mut bundle, ledger) = test_evidence_bundle(tenant_id, candidate.clone());

    // Self-approval: reviewer is the candidate owner
    bundle.approval = Some(Approval {
        approval_id: ApprovalId::new(),
        tenant_id,
        workflow_id: WorkflowId::new(),
        reviewer: candidate.abom.owner.clone(), // Self-approval!
        reviewer_role: "dev_lead".to_string(),
        proposed_capabilities: vec!["draft_invoice_payment".to_string()],
        approved_capabilities: vec!["draft_invoice_payment".to_string()],
        reason_code: "SELF_APPROVAL".to_string(),
        note: None,
        issued_at: now,
        expires_at: now + time::Duration::days(30),
        policy_version: "1.0.0".to_string(),
    });

    let pack = test_policy_pack();
    let evaluation = evaluate(&pack, &bundle, &ledger, now);

    assert_eq!(evaluation.gate_decision, GateDecision::Blocked);
    assert!(evaluation
        .explanation
        .contains("Separation of duties violation"));
}

#[test]
fn test_policy_evaluation_approved_retested_is_certifiable() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(
        tenant_id,
        vec![
            "draft_invoice_payment".to_string(),
            "release_payment".to_string(),
        ],
    );

    let (mut bundle, ledger) = test_evidence_bundle(tenant_id, candidate);

    // Valid reviewer approval (separate principal)
    bundle.approval = Some(Approval {
        approval_id: ApprovalId::new(),
        tenant_id,
        workflow_id: WorkflowId::new(),
        reviewer: PrincipalId::new("sec-lead@corp"),
        reviewer_role: "security_reviewer".to_string(),
        proposed_capabilities: vec!["draft_invoice_payment".to_string()],
        approved_capabilities: vec!["draft_invoice_payment".to_string()],
        reason_code: "LEAST_PRIVILEGE_AP_DRAFT_ONLY".to_string(),
        note: Some("Narrowed to drafting only per policy".to_string()),
        issued_at: now,
        expires_at: now + time::Duration::days(90),
        policy_version: "1.0.0".to_string(),
    });

    // Retest results: positive draft passed, negative release denied
    bundle.retest_results = vec![
        RetestResult {
            case_id: "retest-draft-001".to_string(),
            passed: true,
            action_tested: "draft_invoice_payment".to_string(),
        },
        RetestResult {
            case_id: "retest-release-001".to_string(),
            passed: true, // Passed because release was denied as expected
            action_tested: "release_payment".to_string(),
        },
    ];

    let pack = test_policy_pack();
    let evaluation = evaluate(&pack, &bundle, &ledger, now);

    assert_eq!(evaluation.gate_decision, GateDecision::Certifiable);
}
