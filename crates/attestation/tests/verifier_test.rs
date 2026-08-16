use time::OffsetDateTime;

use sentinel_attestation::{build_attestation_payload, sign_attestation, verify_attestation};
use sentinel_domain::{
    attestation::KeyReference,
    ids::{PolicyPackId, TenantId, WorkflowId},
    policy::GateDecision,
};
use sentinel_test_support::test_candidate_revision;

#[tokio::test]
async fn test_attestation_signing_and_verification_happy_path() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(tenant_id, vec!["draft_invoice_payment".to_string()]);
    let workflow_id = WorkflowId::new();

    let payload = build_attestation_payload(
        tenant_id,
        "production",
        workflow_id,
        &candidate,
        PolicyPackId::new("ap-agent-v1"),
        "1.0.0",
        "sha256:pack-digest",
        "v1.0.0",
        "sha256:corpus-digest",
        "sha256:eval-digest",
        "sha256:manifest-digest",
        vec!["draft_invoice_payment".to_string()],
        None,
        GateDecision::Certifiable,
        now,
        time::Duration::days(90),
        "sentinel-signer@project.iam",
    );

    let key_ref = KeyReference {
        key_resource: "projects/test/locations/us-central1/keyRings/kr/cryptoKeys/k".to_string(),
        key_version: "1".to_string(),
        algorithm: "RSA_SIGN_PSS_2048_SHA256".to_string(),
    };

    let attestation = sign_attestation(&payload, &key_ref, None).await.unwrap();

    let result = verify_attestation(
        &attestation,
        &tenant_id,
        "production",
        Some(&candidate.revision_id.0),
        None,
        now,
    );

    assert!(result.valid);
    assert!(result.checks.signature_valid);
    assert!(result.checks.not_expired);
    assert!(result.checks.revision_matches);
}

#[tokio::test]
async fn test_attestation_1_byte_tamper_rejection() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(tenant_id, vec!["draft_invoice_payment".to_string()]);
    let workflow_id = WorkflowId::new();

    let payload = build_attestation_payload(
        tenant_id,
        "production",
        workflow_id,
        &candidate,
        PolicyPackId::new("ap-agent-v1"),
        "1.0.0",
        "sha256:pack-digest",
        "v1.0.0",
        "sha256:corpus-digest",
        "sha256:eval-digest",
        "sha256:manifest-digest",
        vec!["draft_invoice_payment".to_string()],
        None,
        GateDecision::Certifiable,
        now,
        time::Duration::days(90),
        "sentinel-signer@project.iam",
    );

    let key_ref = KeyReference {
        key_resource: "projects/test/locations/us-central1/keyRings/kr/cryptoKeys/k".to_string(),
        key_version: "1".to_string(),
        algorithm: "RSA_SIGN_PSS_2048_SHA256".to_string(),
    };

    let mut attestation = sign_attestation(&payload, &key_ref, None).await.unwrap();

    // Tamper with 1 byte in the payload
    attestation.payload.model_ref = "vertex-ai:gemini-tampered-model".to_string();

    let result = verify_attestation(
        &attestation,
        &tenant_id,
        "production",
        Some(&candidate.revision_id.0),
        None,
        now,
    );

    // Must fail signature verification!
    assert!(!result.valid);
    assert!(!result.checks.signature_valid);
    assert!(result.error.unwrap().contains("Cryptographic signature verification failed"));
}

#[tokio::test]
async fn test_attestation_expired_rejection() {
    let now = OffsetDateTime::now_utc();
    let tenant_id = TenantId::new();
    let candidate = test_candidate_revision(tenant_id, vec!["draft_invoice_payment".to_string()]);
    let workflow_id = WorkflowId::new();

    let payload = build_attestation_payload(
        tenant_id,
        "production",
        workflow_id,
        &candidate,
        PolicyPackId::new("ap-agent-v1"),
        "1.0.0",
        "sha256:pack-digest",
        "v1.0.0",
        "sha256:corpus-digest",
        "sha256:eval-digest",
        "sha256:manifest-digest",
        vec!["draft_invoice_payment".to_string()],
        None,
        GateDecision::Certifiable,
        now - time::Duration::days(100),
        time::Duration::days(30), // Expired 70 days ago
        "sentinel-signer@project.iam",
    );

    let key_ref = KeyReference {
        key_resource: "projects/test/locations/us-central1/keyRings/kr/cryptoKeys/k".to_string(),
        key_version: "1".to_string(),
        algorithm: "RSA_SIGN_PSS_2048_SHA256".to_string(),
    };

    let attestation = sign_attestation(&payload, &key_ref, None).await.unwrap();

    let result = verify_attestation(
        &attestation,
        &tenant_id,
        "production",
        Some(&candidate.revision_id.0),
        None,
        now,
    );

    assert!(!result.valid);
    assert!(!result.checks.not_expired);
}
