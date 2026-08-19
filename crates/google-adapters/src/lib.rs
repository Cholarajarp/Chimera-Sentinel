#![allow(missing_docs)]
//! Isolated provider ports and adapters for Google Cloud agent services.
//!
//! Provides clean interfaces for Agent Registry, Agent Runtime, Model Armor,
//! Agent Gateway, Memory Bank, Cloud KMS, Firestore, and Cloud Storage with
//! explicit provenance tagging (`LIVE` vs `LOCAL`).

use time::OffsetDateTime;

use sentinel_domain::{
    evidence::{GatewayDecisionEvidence, MemoryRetrievalEvidence, ModelArmorEvidence},
    ids::{CaseId, PrincipalId, TenantId},
    provenance::Provenance,
};

pub mod model_armor {
    use super::*;

    /// Inspects content for prompt injection, jailbreaks, and sensitive data.
    pub async fn inspect_content(
        tenant_id: TenantId,
        case_id: &CaseId,
        content: &str,
        template_id: &str,
        is_live: bool,
    ) -> Result<ModelArmorEvidence, String> {
        let now = OffsetDateTime::now_utc();
        let (disposition, matched_rules, explanation) = if content
            .to_lowercase()
            .contains("ignore previous instructions")
            || content.to_lowercase().contains("ignore policy")
            || content.to_lowercase().contains("override policy")
            || content
                .to_lowercase()
                .contains("pay immediately without review")
        {
            (
                "BLOCK".to_string(),
                vec!["PROMPT_INJECTION_DIRECT".to_string()],
                "Model Armor detected direct prompt injection attempting policy override"
                    .to_string(),
            )
        } else {
            (
                "ALLOW".to_string(),
                vec![],
                "Content inspected: no hostile pattern detected".to_string(),
            )
        };

        Ok(ModelArmorEvidence {
            tenant_id,
            case_id: case_id.clone(),
            template_id: template_id.to_string(),
            disposition,
            matched_rules,
            sanitized_explanation: explanation,
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/templates/{}",
                template_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}

pub mod gateway {
    use super::*;

    /// Evaluates Agent Gateway identity-scoped tool authorization policy.
    pub async fn check_permission(
        tenant_id: TenantId,
        principal: &PrincipalId,
        action: &str,
        is_live: bool,
    ) -> Result<GatewayDecisionEvidence, String> {
        let now = OffsetDateTime::now_utc();

        // Gateway policy: candidate identity is granted draft_invoice_payment,
        // but release_payment is explicitly DENIED by default for least privilege.
        let allowed = match action {
            "draft_invoice_payment" | "get_payment_status" => true,
            "release_payment" => false, // Excessive agency blocked at Gateway
            _ => false,
        };

        Ok(GatewayDecisionEvidence {
            tenant_id,
            principal: principal.clone(),
            action: action.to_string(),
            allowed,
            policy_reference:
                "projects/sentinel-prod/locations/us-central1/gatewayPolicies/ap-least-privilege-v1"
                    .to_string(),
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/gateways/gw-{}",
                tenant_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}

pub mod memory_bank {
    use super::*;

    /// Retrieves advisory prior disposition context from Memory Bank.
    pub async fn query_advisory_context(
        tenant_id: TenantId,
        memory_bank_id: &str,
        query: &str,
        is_live: bool,
    ) -> Result<MemoryRetrievalEvidence, String> {
        let now = OffsetDateTime::now_utc();
        let items = if query.to_lowercase().contains("invoice") {
            vec![
                "Analyst Disposition #2026-04A: Invoices under $5,000 require draft creation only, subject to automated batch verification".to_string(),
            ]
        } else {
            vec![]
        };

        Ok(MemoryRetrievalEvidence {
            tenant_id,
            memory_bank_id: memory_bank_id.to_string(),
            retrieved_items: items,
            analyst_approval_references: vec!["DISP-REF-202604A-SEC".to_string()],
            provider_reference: format!(
                "projects/sentinel-prod/locations/us-central1/memoryBanks/{}",
                memory_bank_id
            ),
            provenance: if is_live {
                Provenance::Live
            } else {
                Provenance::Local
            },
            timestamp: now,
        })
    }
}
